use std::path::PathBuf;

use super::Result;

const MAX_OPTIONS_SIZE: u64 = 1024 * 1024;

/// Copies an `options.txt` chosen by the user into an instance, so it can become the base of
/// the shared game settings.
#[tauri::command]
pub async fn import_game_options_file(
    instance_id: String,
    path: PathBuf,
) -> Result<()> {
    let metadata = tokio::fs::metadata(&path).await?;
    if !metadata.is_file() || metadata.len() > MAX_OPTIONS_SIZE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "options.txt is missing or too large",
        )
        .into());
    }
    let contents = tokio::fs::read(&path).await?;
    let text = String::from_utf8_lossy(&contents);
    let looks_like_options = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .take(20)
        .all(|line| line.contains(':'));
    if !looks_like_options || !text.lines().any(|line| line.contains(':')) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "This file is not a Minecraft options.txt",
        )
        .into());
    }
    let instance_dir = theseus::instance::get_full_path(&instance_id).await?;
    tokio::fs::create_dir_all(&instance_dir).await?;
    let target = instance_dir.join("options.txt");
    let staging = instance_dir.join("options.txt.import");
    tokio::fs::write(&staging, &contents).await?;
    tokio::fs::rename(&staging, &target).await?;
    Ok(())
}
