// config.json 读写
use crate::models::AppConfig;
use crate::storage::{config_path, read_json, write_json};

#[tauri::command]
pub fn load_config() -> Result<AppConfig, String> {
    read_json::<AppConfig>(&config_path())
}

#[tauri::command]
pub fn save_config(config: AppConfig) -> Result<(), String> {
    write_json(&config_path(), &config)
}
