use std::process::Stdio;

static PERMISSION_REQUEST_WAV: &[u8] = include_bytes!("../assets/sounds/permission_request.wav");
static TASK_ERROR_WAV: &[u8] = include_bytes!("../assets/sounds/task_error.wav");
static TASK_COMPLETED_WAV: &[u8] = include_bytes!("../assets/sounds/task_completed.wav");

fn pick_player() -> Option<&'static str> {
    for cmd in &["pw-play", "paplay", "aplay"] {
        if which::which(cmd).is_ok() {
            return Some(cmd);
        }
    }
    None
}

fn get_sound_bytes(sound: &str) -> Option<&'static [u8]> {
    match sound {
        "permission_request" => Some(PERMISSION_REQUEST_WAV),
        "task_error" => Some(TASK_ERROR_WAV),
        "task_completed" => Some(TASK_COMPLETED_WAV),
        _ => None,
    }
}

#[tauri::command]
pub fn play_sound(sound: String) -> Result<(), String> {
    let data = get_sound_bytes(&sound).ok_or_else(|| format!("unknown sound: {}", sound))?;
    let player = pick_player().ok_or_else(|| "no system audio player found (try: pw-play, paplay, aplay)".to_string())?;

    let tmp = std::env::temp_dir().join(format!("agentos_{}.wav", sound));
    std::fs::write(&tmp, data).map_err(|e| format!("failed to write temp sound: {}", e))?;

    let status = std::process::Command::new(player)
        .arg(&tmp)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| format!("failed to execute {}: {}", player, e))?;

    let _ = std::fs::remove_file(&tmp);

    if status.success() {
        Ok(())
    } else {
        Err(format!("{} exited with code {:?}", player, status.code()))
    }
}
