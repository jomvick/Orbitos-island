mod daemon_client;
mod sounds;

use tauri::Emitter;
use tauri::Manager;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

#[tauri::command]
fn set_ignore_cursor(app: tauri::AppHandle, ignore: bool) {
    // No-op to preserve always-interactive window behavior under dynamic sizing
}

#[tauri::command]
async fn update_window_size(
    width: f64,
    height: f64,
    center: Option<bool>,
    window: tauri::WebviewWindow,
) -> Result<(), String> {
    if width <= 0.0 || height <= 0.0 {
        return Err(format!("Invalid window dimensions: {}x{}", width, height));
    }

    let scale_factor = window.current_monitor()
        .map_err(|e| e.to_string())?
        .map(|m| m.scale_factor())
        .unwrap_or(1.0);

    // Apply new size
    window.set_size(tauri::Size::Logical(tauri::LogicalSize { width, height }))
        .map_err(|e| e.to_string())?;

    if center.unwrap_or(false) {
        // Re-center horizontally on the current monitor
        if let Ok(Some(monitor)) = window.current_monitor() {
            let monitor_size = monitor.size();
            let monitor_width = monitor_size.width as f64 / scale_factor;
            let monitor_height = monitor_size.height as f64 / scale_factor;
            let new_x = (monitor_width - width) / 2.0;
            let new_y = (monitor_height - height) / 2.0;
            window.set_position(tauri::Position::Logical(tauri::LogicalPosition {
                x: new_x,
                y: new_y,
            })).map_err(|e| e.to_string())?;
        }
    } else {
        // Capture current window size and position to preserve screen centering
        let current_size = window.outer_size().map_err(|e| e.to_string())?;
        let current_pos = window.outer_position().map_err(|e| e.to_string())?;

        let current_width_logical = current_size.width as f64 / scale_factor;
        let current_pos_x_logical = current_pos.x as f64 / scale_factor;

        // Calculate new x coordinate to keep the horizontal center stable
        let dx = width - current_width_logical;
        let new_x = current_pos_x_logical - (dx / 2.0);

        // Apply new position to maintain center alignment
        window.set_position(tauri::Position::Logical(tauri::LogicalPosition {
            x: new_x,
            y: current_pos.y as f64 / scale_factor,
        })).map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
async fn start_drag(window: tauri::WebviewWindow) -> Result<(), String> {
    window.start_dragging().map_err(|e| e.to_string())
}

#[tauri::command]
async fn ensure_always_on_top(window: tauri::WebviewWindow) -> Result<(), String> {
    window.set_always_on_top(true).map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}


fn build_tray_menu(app: &tauri::AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let show = MenuItem::with_id(app, "show", "Show Orbitos Island", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let settings = MenuItem::with_id(app, "settings", "Preferences…", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Orbitos Island", true, None::<&str>)?;

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
            let window = app.get_webview_window("main").unwrap();
            let _ = window.set_decorations(false);
            let _ = window.set_always_on_top(true);
            let _ = window.set_resizable(true);
            let _ = window.set_skip_taskbar(true);
            let _ = window.set_ignore_cursor_events(false);

            // Enforce initial position top-center
            if let Ok(Some(monitor)) = window.current_monitor() {
                let monitor_size = monitor.size();
                let scale_factor = monitor.scale_factor();
                let monitor_width = monitor_size.width as f64 / scale_factor;
                
                let window_width = 364.0;
                let window_height = 78.0;
                let x = (monitor_width - window_width) / 2.0;
                let y = 48.0; // Float beautifully below Gnome's top bar
                
                let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize {
                    width: window_width,
                    height: window_height,
                }));
                let _ = window.set_position(tauri::Position::Logical(tauri::LogicalPosition { x, y }));
            }

            // Linux/Wayland always-on-top stabilization
            let window_clone = window.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(500));
                let _ = window_clone.set_always_on_top(true);
                let _ = window_clone.set_focus();
            });

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
            daemon_client::discover_agents,
            daemon_client::ping,
            set_ignore_cursor,
            update_window_size,
            start_drag,
            ensure_always_on_top,
            sounds::play_sound
        ])
        .run(tauri::generate_context!())
        .expect("error while running agentos desktop");
}
