// 便携存储：所有 JSON 都放在 .exe 同级 Data/ 目录下
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::de::DeserializeOwned;
use serde::Serialize;

/// 获取 Data 目录（.exe 同级 / 开发期为 target/debug 同级）
pub fn data_dir() -> PathBuf {
    let exe = std::env::current_exe().expect("current_exe failed");
    let base = exe.parent().expect("exe parent missing").to_path_buf();
    let dir = base.join("Data");
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }
    let logs = dir.join("logs");
    if !logs.exists() {
        let _ = std::fs::create_dir_all(&logs);
    }
    dir
}

pub fn config_path() -> PathBuf {
    data_dir().join("config.json")
}
pub fn launchers_path() -> PathBuf {
    data_dir().join("launchers.json")
}
pub fn projects_path() -> PathBuf {
    data_dir().join("projects.json")
}
pub fn logs_dir() -> PathBuf {
    data_dir().join("logs")
}

/// 全局写锁：防止并发写坏 JSON 文件
pub static IO_LOCK: Mutex<()> = Mutex::new(());

pub fn read_json<T: DeserializeOwned + Default>(path: &Path) -> Result<T, String> {
    if !path.exists() {
        return Ok(T::default());
    }
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    if content.trim().is_empty() {
        return Ok(T::default());
    }
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let _guard = IO_LOCK.lock().map_err(|e| e.to_string())?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let s = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    std::fs::write(path, s).map_err(|e| e.to_string())?;
    Ok(())
}
