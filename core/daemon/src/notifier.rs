use daemon_core::events::EventBus;
use daemon_core::notifications::dispatcher::event_to_notification;
use daemon_core::notifications::{NotificationCategory, NotificationPriority};
use tracing::warn;

pub async fn start_notification_loop(event_bus: EventBus, app_name: &str) {
    let mut rx = event_bus.subscribe();
    let app = app_name.to_string();

    loop {
        match rx.recv().await {
            Ok(event) => {
                if let Some(notification) = event_to_notification(&event) {
                    if let Err(e) = send_desktop_notification(&notification, &app) {
                        warn!(error = %e, "failed to send notification");
                    }
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                warn!(count = %n, "notification loop lagged");
            }
            Err(_) => break,
        }
    }
}

fn send_desktop_notification(
    notification: &daemon_core::notifications::Notification,
    app_name: &str,
) -> Result<(), String> {
    let mut n = notify_rust::Notification::new();
    n.appname(app_name)
        .summary(&notification.title)
        .body(&notification.body);

    n.timeout(match notification.priority {
        NotificationPriority::Urgent => notify_rust::Timeout::Milliseconds(10000),
        NotificationPriority::High => notify_rust::Timeout::Milliseconds(7000),
        NotificationPriority::Normal => notify_rust::Timeout::Milliseconds(5000),
        NotificationPriority::Low => notify_rust::Timeout::Milliseconds(3000),
    });

    n.hint(notify_rust::Hint::Category(match notification.category {
        NotificationCategory::TaskComplete => "task_complete".to_string(),
        NotificationCategory::PermissionRequest => "permission".to_string(),
        NotificationCategory::Error => "error".to_string(),
        NotificationCategory::Warning => "warning".to_string(),
        NotificationCategory::Info => "info".to_string(),
        NotificationCategory::Progress => "progress".to_string(),
    }));

    if notification.priority == NotificationPriority::Urgent {
        n.urgency(notify_rust::Urgency::Critical);
    } else if notification.priority == NotificationPriority::High {
        n.urgency(notify_rust::Urgency::Normal);
    } else {
        n.urgency(notify_rust::Urgency::Low);
    }

    n.show().map_err(|e| e.to_string())?;
    Ok(())
}
