// 便携存储：所有 JSON 都放在 %USERPROFILE%\.forge-studio\ 目录下
// 这样无论是 dev (target\debug\...) 还是 release (target\release\...)，甚至拷贝到别处，
// 共享同一份用户数据。目录在用户主目录下"用户级"固定，不污染系统。
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::de::DeserializeOwned;
use serde::Serialize;

/// 获取 Data 目录（%USERPROFILE%/.forge-studio/，开发/release 共享）
pub fn data_dir() -> PathBuf {
    // 优先读 USERPROFILE；PowerShell / 普通登录都有这个变量
    let home = std::env::var("USERPROFILE")
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            // 兜底：HOME（极少数情况，比如 service 账户）
            std::env::var("HOME").ok().map(PathBuf::from)
        })
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let dir = home.join(".forge-studio");
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
pub fn events_path() -> PathBuf {
    data_dir().join("events.json")
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
