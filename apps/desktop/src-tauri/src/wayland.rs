use std::process::Command;

/// GTK/Tauri uses the app identifier as the Wayland app_id.
pub const APP_ID: &str = "com.orbitos.desktop";

#[derive(Debug, Clone, PartialEq)]
pub enum Compositor {
    Sway,
    Hyprland,
    River,
    Kde,
    Gnome,
    Wlroots,
    Unknown,
}

impl Compositor {
    pub fn detect() -> Self {
        if std::env::var("SWAYSOCK").is_ok() {
            return Self::Sway;
        }
        if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
            return Self::Hyprland;
        }
        if std::env::var("RIVER_SOCKET").is_ok() {
            return Self::River;
        }
        if std::env::var("KDE_FULL_SESSION").is_ok()
            || std::env::var("XDG_CURRENT_DESKTOP")
                .map(|d| d.to_lowercase().contains("kde"))
                .unwrap_or(false)
        {
            return Self::Kde;
        }
        if std::env::var("XDG_CURRENT_DESKTOP")
            .map(|d| d.to_lowercase().contains("gnome"))
            .unwrap_or(false)
        {
            return Self::Gnome;
        }
        if std::env::var("XDG_CURRENT_DESKTOP")
            .map(|d| d.to_lowercase().contains("sway"))
            .unwrap_or(false)
        {
            return Self::Wlroots;
        }
        Self::Unknown
    }

    pub fn configure_overlay_rules(&self, app_id: &str) {
        match self {
            Self::Sway => {
                // Float, sticky, no focus on hover for overlay behavior
                let rules = format!(
                    "for_window [app_id=\"{app_id}\"] floating enable, sticky enable, focus_on_window_activation none"
                );
                let _ = Command::new("swaymsg")
                    .args([&rules])
                    .output();
            }
            Self::Hyprland => {
                // Window rule: float, no border, pin, no focus on activate
                let rule = format!(
                    "windowrulev2 = float, class:({app_id})\n\
                     windowrulev2 = noborder, class:({app_id})\n\
                     windowrulev2 = pin, class:({app_id})\n\
                     windowrulev2 = nofocus, class:({app_id})"
                );
                let _ = Command::new("hyprctl")
                    .args(["keyword", &rule])
                    .output();
            }
            Self::River => {
                let _ = Command::new("riverctl")
                    .args([
                        "rule-add",
                        &format!("app-id=\"{app_id}\""),
                        "float",
                    ])
                    .output();
                let _ = Command::new("riverctl")
                    .args([
                        "rule-add",
                        &format!("app-id=\"{app_id}\""),
                        "csd",
                    ])
                    .output();
            }
            Self::Kde => {
                let kwrite = if Command::new("kwriteconfig6").arg("--version").output().is_ok() {
                    "kwriteconfig6"
                } else {
                    "kwriteconfig5"
                };
                let _ = Command::new(kwrite)
                    .args([
                        "--file",
                        "kwinrulesrc",
                        "--group",
                        "1",
                        "--key",
                        "Description",
                        "Orbitos Island Overlay",
                    ])
                    .output();
                let _ = Command::new(kwrite)
                    .args([
                        "--file",
                        "kwinrulesrc",
                        "--group",
                        "1",
                        "--key",
                        "wmclass",
                        app_id,
                    ])
                    .output();
                let _ = Command::new(kwrite)
                    .args([
                        "--file",
                        "kwinrulesrc",
                        "--group",
                        "1",
                        "--key",
                        "types",
                        "1",
                    ])
                    .output();
            }
            Self::Gnome => {
                // GNOME: use gsettings/mutter overrides (limited effect, best effort)
                let _ = Command::new("gsettings")
                    .args([
                        "set",
                        "org.gnome.mutter",
                        "attach-modal-dialogs",
                        "false",
                    ])
                    .output();
            }
            Self::Wlroots | Self::Unknown => {}
        }
    }

    pub fn configure_input_ignore(&self, window_id: u64) {
        match self {
            Self::Sway | Self::Hyprland | Self::Wlroots => {
                // Use wlrctl for input region manipulation on wlroots compositors
                let _ = Command::new("wlrctl")
                    .args(["toplevel", "set-input-region", "0x0+0+0"])
                    .output();
            }
            _ => {}
        }
    }

    pub fn configure_input_accept(&self, _window_id: u64) {
        // Restore default input region (entire window)
        let _ = Command::new("wlrctl")
            .args(["toplevel", "set-input-region", "0"])
            .output();
    }

    /// Returns recommended Sway config snippet for true layer-shell overlay.
    /// This can't be applied programmatically from a client — the user
    /// must add it to their sway config (~/.config/sway/config).
    pub fn sway_layer_shell_config(app_id: &str) -> String {
        format!(
            "for_window [app_id=\"{app_id}\"] floating enable, sticky enable, border none\n\
             # For full layer-shell, install wlr-layer-shell-unstable-v1 support\n\
             # and add: layer=overlay, exclusive_zone=-1, anchor=top"
        )
    }
}

pub struct WaylandState {
    pub active: bool,
    pub compositor: Compositor,
}

pub fn detect_wayland() -> WaylandState {
    let active = std::env::var("WAYLAND_DISPLAY").is_ok();
    let compositor = if active {
        Compositor::detect()
    } else {
        Compositor::Unknown
    };
    WaylandState { active, compositor }
}