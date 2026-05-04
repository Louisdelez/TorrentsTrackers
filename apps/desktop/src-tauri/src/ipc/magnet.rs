use std::process::Command;

#[tauri::command]
pub fn open_magnet(magnet: String) -> Result<(), String> {
    let status = if cfg!(target_os = "linux") {
        Command::new("xdg-open").arg(&magnet).status()
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(&magnet).status()
    } else if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "start", "", &magnet])
            .status()
    } else {
        return Err("unsupported OS".into());
    };
    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => Err(format!("opener exited with {s}")),
        Err(e) => Err(format!("failed to spawn opener: {e}")),
    }
}
