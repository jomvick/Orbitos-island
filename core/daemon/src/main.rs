mod cli;
mod discover;
mod notifier;
mod plugin_loader;
mod server;
mod handlers;
mod watcher;

use std::path::Path;
use std::sync::Arc;

use clap::Parser;
use tracing::{error, info, warn};
use tracing_subscriber::EnvFilter;

use tokio::sync::Mutex;

use agentos_ipc::{IpcServer, SocketConfig};
use agentos_storage::Database;

use crate::cli::CliArgs;
use crate::server::{handle_client, DaemonState};

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();

    if args.discover {
        let result = discover::discover_agents();
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
        return;
    }



    let filter = if args.verbose {
        EnvFilter::new("agentosd=debug,daemon_core=debug,agentos_ipc=debug,agentos_storage=debug")
    } else {
        EnvFilter::new("agentosd=info,daemon_core=info")
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .init();

    info!("agentosd v{} starting", env!("CARGO_PKG_VERSION"));

    let db = match setup_database(&args).await {
        Ok(db) => {
            info!("database initialized");
            db
        }
        Err(e) => {
            error!(error = %e, "failed to initialize database");
            std::process::exit(1);
        }
    };

    let plugin_registry = plugin_loader::load_default_plugins();

    // Install transparent shell wrappers for all detected agents.
    let wrapper_results = discover::install_shell_wrappers();
    let installed: Vec<_> = wrapper_results.iter().filter(|r| r.installed).collect();
    if !installed.is_empty() {
        info!(count = %installed.len(), "shell wrappers installed/updated");
    }

    let db_for_events: Arc<Mutex<Database>> = Arc::new(Mutex::new(db));

    let state = Arc::new(DaemonState::new(
        plugin_registry,
        Some(db_for_events.clone()),
    ));

    // Restore active sessions from database into in-memory state
    // so that sessions started before this daemon instance are visible.
    {
        let db = db_for_events.lock().await;
        match db.get_active_sessions() {
            Ok(stored_sessions) => {
                let restored = stored_sessions.len();
                let mut session_state = state.session_state.write().await;
                for stored in stored_sessions {
                    if let Ok(session) = stored.to_domain() {
                        session_state.sessions.insert(session.id.clone(), session);
                    }
                }
                info!(count = %restored, "restored active sessions from database");
            }
            Err(e) => {
                error!(error = %e, "failed to restore active sessions from database");
            }
        }
    }

    {
        let session_state = state.session_state.read().await;
        let count = session_state.total_count();
        info!(count = %count, "sessions in memory");
    }

    let state_for_events = state.clone();

    tokio::spawn(async move {
        persist_events_loop(state_for_events, db_for_events).await;
    });

    let notif_event_bus = state.event_bus.clone();
    tokio::spawn(async move {
        notifier::start_notification_loop(notif_event_bus, "agentosd").await;
    });

    // Spawn the process watcher — detects agent crashes and synthesizes
    // session_completed events for sessions whose PIDs have vanished.
    {
        let watcher_state = state.clone();
        tokio::spawn(async move {
            watcher::start_process_watcher(watcher_state).await;
        });
    }

    // Stale session watchdog — marks running sessions as orphaned
    // if they haven't sent a heartbeat in 30 seconds.
    {
        let watchdog_state = state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
            loop {
                interval.tick().await;
                let mut session_state = watchdog_state.session_state.write().await;
                let stale_threshold = chrono::Duration::seconds(30);
                let mut orphaned = 0;
                for (_, session) in session_state.sessions.iter_mut() {
                    if session.is_active() && session.is_stale(&stale_threshold) {
                        session.phase = daemon_core::state::SessionPhase::Orphaned;
                        orphaned += 1;
                    }
                }
                if orphaned > 0 {
                    tracing::info!(count = %orphaned, "orphaned stale sessions");
                }
            }
        });
    }

    // Export agent traces/metrics via OpenTelemetry (if --otlp-endpoint is set)
    if let Some(ref otlp_endpoint) = args.otlp_endpoint {
        let rx = state.event_bus.subscribe();
        let endpoint = otlp_endpoint.clone();
        tokio::spawn(async move {
            agentos_exporter::OtlpExporter::new(endpoint).start(rx).await;
        });
        info!(endpoint = %otlp_endpoint, "OTLP exporter initialized");
    }

    let mut socket_path = args
        .socket_path
        .clone()
        .unwrap_or_else(agentos_ipc::get_default_socket_path);

    if !socket_path.is_absolute() {
        let home = std::env::var("HOME").unwrap_or_else(|_| {
            warn!("HOME env var not set, falling back to /tmp");
            "/tmp".to_string()
        });
        socket_path = std::path::Path::new(&home).join(&socket_path);
    }

    let socket_config = SocketConfig {
        path: socket_path.clone(),
        max_connections: args.max_connections,
    };

    let server = match IpcServer::bind(socket_config) {
        Ok(s) => {
            info!(path = %socket_path.display(), "IPC server listening");
            s
        }
        Err(e) => {
            error!(error = %e, "failed to bind IPC server");
            std::process::exit(1);
        }
    };

    tokio::spawn(async move {
        loop {
            match server.accept().await {
                Ok((codec, _fd)) => {
                    let state = state.clone();
                    tokio::spawn(async move {
                        handle_client(codec, state).await;
                    });
                }
                Err(e) => {
                    error!(error = %e, "failed to accept connection");
                }
            }
        }
    });

    info!("agentosd ready");

    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for signal");
    info!("shutting down");
}

async fn setup_database(args: &CliArgs) -> Result<Database, Box<dyn std::error::Error>> {
    if args.db_in_memory {
        return Ok(Database::open_in_memory()?);
    }

    let db_path = if args.db_path.starts_with("~") {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        Path::new(&home).join(args.db_path.strip_prefix("~/").unwrap_or(&args.db_path))
    } else {
        args.db_path.clone()
    };

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let db = Database::open(&db_path)?;

    match db.integrity_check() {
        Ok(result) if result == "ok" => info!("database integrity: ok"),
        Ok(result) => warn!("database integrity: {}", result),
        Err(e) => error!("integrity check failed: {}", e),
    }

    Ok(db)
}

async fn persist_events_loop(state: Arc<DaemonState>, db: Arc<Mutex<Database>>) {
    let mut rx = state.event_bus.subscribe();
    use tokio::sync::broadcast::error::RecvError;

    let mut prune_interval = tokio::time::interval(std::time::Duration::from_secs(300));

    loop {
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Ok(event) => {
                        let session_id = event.session_id.clone();
                        {
                            let db = db.lock().await;
                            let _ = db.insert_event(&event);
                        }
                        let session = {
                            let session_state = state.session_state.read().await;
                            session_state.sessions.get(&session_id).cloned()
                        };
                        if let Some(session) = session {
                            let db = db.lock().await;
                            let _ = db.upsert_session(&session);
                        }
                    }
                    Err(RecvError::Lagged(n)) => {
                        warn!(count = %n, "persistence lagged");
                    }
                    Err(RecvError::Closed) => break,
                }
            }
            _ = prune_interval.tick() => {
                let mut session_state = state.session_state.write().await;
                let before = session_state.total_count();
                session_state.prune_orphaned(chrono::Duration::hours(1));
                let after = session_state.total_count();
                if before != after {
                    info!(before = %before, after = %after, "pruned completed sessions");
                }
            }
        }
    }
}
