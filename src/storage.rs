use crate::model::PlayerState;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn ensure_data_layout(data_dir: &Path) -> Result<(), String> {
    fs::create_dir_all(data_dir.join("players")).map_err(|e| e.to_string())
}

pub fn seed_file(data_dir: &Path, name: &str, contents: &str) -> Result<PathBuf, String> {
    let path = data_dir.join(name);
    if !path.exists() {
        fs::write(&path, contents)
            .map_err(|e| format!("failed to create {}: {e}", path.display()))?;
    }
    Ok(path)
}

pub fn load_player(data_dir: &Path, id: &str) -> PlayerState {
    let path = data_dir.join("players").join(format!("{id}.json"));
    fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

pub fn save_player(data_dir: &Path, id: &str, state: &PlayerState) -> Result<(), String> {
    let path = data_dir.join("players").join(format!("{id}.json"));
    let temp = data_dir.join("players").join(format!("{id}.json.tmp"));
    let bytes = serde_json::to_vec_pretty(state).map_err(|e| e.to_string())?;
    fs::write(&temp, bytes).map_err(|e| e.to_string())?;
    fs::rename(&temp, &path).map_err(|e| e.to_string())
}
