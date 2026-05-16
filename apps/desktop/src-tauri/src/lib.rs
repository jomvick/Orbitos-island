mod daemon_client;

use tauri::Emitter;
use tauri::Manager;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

#[tauri::command]
fn set_ignore_cursor(app: tauri::AppHandle, ignore: bool) {
    let window = app.get_webview_window("main").unwrap();
    let _ = window.set_ignore_cursor_events(ignore);
}

fn build_tray_menu(app: &tauri::AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let show = MenuItem::with_id(app, "show", "Show AgentOS", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let settings = MenuItem::with_id(app, "settings", "Preferences…", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit AgentOS", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show, &separator, &settings, &separator, &quit])?;
    Ok(menu)
}

fn handle_tray_event(tray: &tauri::tray::TrayIcon, event: TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        if let Some(window) = tray.app_handle().get_webview_window("main") {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

fn handle_menu_event(app: &tauri::AppHandle, id: &str) {
    match id {
        "show" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "settings" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.emit("navigate", "/settings");
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "quit" => {
            std::process::exit(0);
        }
        _ => {}
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let menu = build_tray_menu(&handle)?;
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| {
                    handle_menu_event(app, event.id().as_ref());
                })
                .on_tray_icon_event(handle_tray_event)
                .build(&handle)?;

            tauri::async_runtime::spawn(async move {
                match daemon_client::connect_and_listen(handle).await {
                    Ok(_) => tracing::info!("daemon listener finished"),
                    Err(e) => tracing::error!("daemon connection failed: {}", e),
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            daemon_client::get_sessions,
            daemon_client::get_session,
            daemon_client::get_session_stats,
            daemon_client::get_agent_analytics,
            daemon_client::get_timeline,
            daemon_client::search_sessions,
            daemon_client::resolve_permission,
            daemon_client::answer_question,
            daemon_client::jump_to_session,
            daemon_client::stop_agent,
            daemon_client::shutdown,
            daemon_client::ping,
            set_ignore_cursor
        ])
        .run(tauri::generate_context!())
        .expect("error while running agentos desktop");
}
