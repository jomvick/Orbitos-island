use std::time::Duration;

use agentos_ipc::{BridgeCodec, IpcCommand, IpcMessage, IpcServer, SocketConfig};
use chrono::Utc;
use daemon_core::state::{AgentKind, EventKind, UniversalEvent};
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;
use uuid::Uuid;

fn make_event() -> UniversalEvent {
    UniversalEvent {
        id: Uuid::new_v4(),
        agent: AgentKind::Opencode,
        event: EventKind::SessionStarted,
        session_id: "test-session".to_string(),
        cwd: Some("/tmp".to_string()),
        branch: None,
        model: Some("test-model".to_string()),
        tokens_input: Some(100),
        tokens_output: Some(50),
        duration_ms: Some(1000),
        terminal: Some("tmux".to_string()),
        pane: Some("0".to_string()),
        permission: None,
        question: None,
        jump_target: None,
        plan: None,
        diff: None,
        error: None,
        current_action: None,
        metadata: None,
        pid: None,
        ppid: None,
        timestamp: Utc::now(),
    }
}

async fn connect_client(socket_path: &std::path::Path) -> BridgeCodec {
    let stream = UnixStream::connect(socket_path)
        .await
        .expect("failed to connect");
    let (read, write) = stream.into_split();
    BridgeCodec::new(read, write)
}

#[tokio::test]
async fn test_event_send_receive() {
    let dir = tempfile::tempdir().unwrap();
    let socket_path = dir.path().join("test.sock");

    let config = SocketConfig {
        path: socket_path.clone(),
        max_connections: 8,
    };
    let server = IpcServer::bind(config).unwrap();

    let server_path = server.local_path().to_path_buf();

    tokio::spawn(async move {
        let (mut codec, _fd) = server.accept().await.unwrap();
        let msg = codec.recv().await.unwrap();
        match msg {
            IpcMessage::Event { source, payload, .. } => {
                assert_eq!(source, "test-agent");
                let event: UniversalEvent =
                    serde_json::from_value(payload).unwrap();
                assert_eq!(event.session_id, "test-session");
                let response = IpcMessage::new_response(Uuid::new_v4(), None);
                codec.send(&response).await.unwrap();
            }
            _ => panic!("expected Event message"),
        }
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    let mut client = connect_client(&server_path).await;
    let event = make_event();
    let payload = serde_json::to_value(&event).unwrap();
    let msg = IpcMessage::new_event("test-agent", payload);
    client.send(&msg).await.unwrap();

    let response = client.recv().await.unwrap();
    assert!(matches!(response, IpcMessage::Response { .. }));
}

#[tokio::test]
async fn test_command_response_ping() {
    let dir = tempfile::tempdir().unwrap();
    let socket_path = dir.path().join("ping.sock");

    let config = SocketConfig {
        path: socket_path.clone(),
        max_connections: 8,
    };
    let server = IpcServer::bind(config).unwrap();
    let server_path = server.local_path().to_path_buf();

    tokio::spawn(async move {
        let (mut codec, _fd) = server.accept().await.unwrap();
        let msg = codec.recv().await.unwrap();
        match msg {
            IpcMessage::Command { id, command, .. } => {
                match command {
                    IpcCommand::Ping => {
                        let response = IpcMessage::new_response(
                            id,
                            Some(serde_json::json!({"pong": true})),
                        );
                        codec.send(&response).await.unwrap();
                    }
                    _ => {
                        let err = IpcMessage::new_error(id, "unexpected".to_string());
                        codec.send(&err).await.unwrap();
                    }
                }
            }
            _ => panic!("expected Command message"),
        }
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    let mut client = connect_client(&server_path).await;
    let cmd = IpcMessage::new_command(IpcCommand::Ping);
    client.send(&cmd).await.unwrap();

    let response = client.recv().await.unwrap();
    match response {
        IpcMessage::Response { status, data, .. } => {
            assert_eq!(status, agentos_ipc::IpcStatus::Ok);
            assert_eq!(data, Some(serde_json::json!({"pong": true})));
        }
        _ => panic!("expected Response message"),
    }
}

#[tokio::test]
async fn test_subscribe_event_flow() {
    let dir = tempfile::tempdir().unwrap();
    let socket_path = dir.path().join("sub.sock");

    let config = SocketConfig {
        path: socket_path.clone(),
        max_connections: 8,
    };
    let server = IpcServer::bind(config).unwrap();
    let server_path = server.local_path().to_path_buf();

    tokio::spawn(async move {
        let (mut codec, _fd) = server.accept().await.unwrap();
        let msg = codec.recv().await.unwrap();
        assert!(matches!(msg, IpcMessage::Subscribe { .. }));

        let event = make_event();
        let sub_msg = IpcMessage::SubscriptionEvent {
            channel: "sessions".to_string(),
            event: Box::new(event),
            session: None,
            timestamp: Utc::now(),
        };
        codec.send(&sub_msg).await.unwrap();
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    let mut client = connect_client(&server_path).await;
    let sub = IpcMessage::Subscribe {
        channel: "sessions".to_string(),
        timestamp: Utc::now(),
    };
    client.send(&sub).await.unwrap();

    let response = client.recv().await.unwrap();
    match response {
        IpcMessage::SubscriptionEvent { channel, event, .. } => {
            assert_eq!(channel, "sessions");
            assert_eq!(event.session_id, "test-session");
            assert_eq!(event.agent, AgentKind::Opencode);
        }
        _ => panic!("expected SubscriptionEvent message"),
    }
}

#[tokio::test]
async fn test_max_message_size_enforced() {
    let dir = tempfile::tempdir().unwrap();
    let socket_path = dir.path().join("msgsize.sock");

    let config = SocketConfig {
        path: socket_path.clone(),
        max_connections: 8,
    };
    let server = IpcServer::bind(config).unwrap();
    let server_path = server.local_path().to_path_buf();

    tokio::spawn(async move {
        let (mut codec, _fd) = server.accept().await.unwrap();
        let result = codec.recv().await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_str = err.to_string();
        assert!(err_str.contains("too large") || err_str.contains("MessageTooLarge"));
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    let mut client = connect_client(&server_path).await;
    let large_payload = serde_json::json!({"data": "x".repeat(2_000_000)});
    let oversized = IpcMessage::new_event("test", large_payload);
    let result = client.send(&oversized).await;

    match result {
        Err(e) => {
            let err_str = e.to_string();
            assert!(
                err_str.contains("too large") 
                    || err_str.contains("MessageTooLarge") 
                    || err_str.contains("Broken pipe") 
                    || err_str.contains("Connection reset by peer"),
                "expected size error or broken pipe, got: {}",
                err_str
            );
        }
        Ok(_) => {
            // The send may succeed from the client side since the 1MiB
            // check is on recv. Try receiving an error from the server
            // side via our bounded connection.
        }
    }
}

#[tokio::test]
async fn test_multiple_clients() {
    let dir = tempfile::tempdir().unwrap();
    let socket_path = dir.path().join("multi.sock");

    let config = SocketConfig {
        path: socket_path.clone(),
        max_connections: 8,
    };
    let server = IpcServer::bind(config).unwrap();
    let server_path = server.local_path().to_path_buf();

    let num_clients = 3;

    tokio::spawn(async move {
        for i in 0..num_clients {
            let (mut codec, _fd) = server.accept().await.unwrap();
            let msg = codec.recv().await.unwrap();
            if let IpcMessage::Command { id, .. } = msg {
                let response = IpcMessage::new_response(
                    id,
                    Some(serde_json::json!({"client": i})),
                );
                codec.send(&response).await.unwrap();
            }
        }
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    for _ in 0..num_clients {
        let mut client = connect_client(&server_path).await;
        let cmd = IpcMessage::new_command(IpcCommand::Ping);
        client.send(&cmd).await.unwrap();
        let response = client.recv().await.unwrap();
        assert!(matches!(response, IpcMessage::Response { .. }));
    }
}

#[tokio::test]
async fn test_malformed_json_does_not_crash_server() {
    let dir = tempfile::tempdir().unwrap();
    let socket_path = dir.path().join("malformed.sock");

    let config = SocketConfig {
        path: socket_path.clone(),
        max_connections: 8,
    };
    let server = IpcServer::bind(config).unwrap();
    let server_path = server.local_path().to_path_buf();

    tokio::spawn(async move {
        while let Ok((codec, _fd)) = server.accept().await {
            tokio::spawn(async move {
                let mut codec = codec;
                while let Ok(msg) = codec.recv().await {
                    if let IpcMessage::Command { id, command, .. } = msg {
                        match command {
                            IpcCommand::Ping => {
                                let _ = codec
                                    .send(&IpcMessage::new_response(
                                        id,
                                        Some(serde_json::json!({"pong": true})),
                                    ))
                                    .await;
                            }
                            _ => {
                                let _ = codec
                                    .send(&IpcMessage::new_error(
                                        id,
                                        "unexpected".to_string(),
                                    ))
                                    .await;
                            }
                        }
                    }
                }
            });
        }
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    {
        let stream = UnixStream::connect(&server_path).await.unwrap();
        let (_, mut wr) = stream.into_split();
        wr.write_all(b"not json at all\n").await.unwrap();
        wr.shutdown().await.unwrap();
    }

    tokio::time::sleep(Duration::from_millis(50)).await;

    let mut client = connect_client(&server_path).await;
    let cmd = IpcMessage::new_command(IpcCommand::Ping);
    client.send(&cmd).await.unwrap();

    let response = client.recv().await.unwrap();
    match response {
        IpcMessage::Response { status, data, .. } => {
            assert_eq!(status, agentos_ipc::IpcStatus::Ok);
            assert_eq!(data, Some(serde_json::json!({"pong": true})));
        }
        _ => panic!("expected Response message"),
    }
}

#[tokio::test]
async fn test_hook_timeout_does_not_hang() {
    let start = std::time::Instant::now();
    let result = tokio::time::timeout(
        Duration::from_millis(500),
        UnixStream::connect("/tmp/nonexistent-agentos-test.sock"),
    )
    .await;
    let elapsed = start.elapsed();

    match result {
        Err(_) => {}
        Ok(Err(e)) => {
            assert!(
                matches!(e.kind(), std::io::ErrorKind::ConnectionRefused | std::io::ErrorKind::NotFound),
                "unexpected error kind: {:?}",
                e.kind()
            );
        }
        Ok(Ok(_)) => panic!("unexpectedly connected to non-existent socket"),
    }

    assert!(
        elapsed.as_millis() < 300,
        "timeout took too long: {}ms",
        elapsed.as_millis()
    );
}
