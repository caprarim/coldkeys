use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Bind {
    pub id: String,
    pub name: String,
    pub command: String,
    pub accelerator: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub gnome_key: Option<String>,
}

fn default_true() -> bool {
    true
}

pub fn config_path(dir: &PathBuf) -> PathBuf {
    dir.join("binds.json")
}

pub fn load(dir: &PathBuf) -> Vec<Bind> {
    let path = config_path(dir);
    match fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub fn save(dir: &PathBuf, binds: &[Bind]) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let text = serde_json::to_string_pretty(binds).map_err(|e| e.to_string())?;
    fs::write(config_path(dir), text).map_err(|e| e.to_string())
}
