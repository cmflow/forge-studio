// 其他 Command：打开日志目录 & 清空所有数据
use crate::storage::{config_path, launchers_path, logs_dir, projects_path};

#[tauri::command]
pub fn open_logs_dir() -> Result<(), String> {
    let dir = logs_dir();
    // Windows 上使用 explorer 打开目录（跨平台可后续替换为 tauri_plugin_opener）
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&dir)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        // 非 Windows：占位，避免编译错误
        let _ = dir;
    }
    Ok(())
}

#[tauri::command]
pub fn clear_all_data() -> Result<(), String> {
    for p in [config_path(), launchers_path(), projects_path()] {
        if p.exists() {
            std::fs::remove_file(&p).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
