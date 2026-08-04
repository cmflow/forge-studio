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

/// 在文件资源管理器中定位并高亮指定文件/目录。
/// Windows: `explorer /select,<path>`，直接跳转到 dcf/cbp 所在目录并选中该文件。
#[tauri::command]
pub fn reveal_in_explorer(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Err(format!("路径不存在: {}", path));
    }
    #[cfg(target_os = "windows")]
    {
        // 注意：/select, 后必须紧跟路径，不能有空格；用单参数拼接最稳
        std::process::Command::new("explorer")
            .arg(format!("/select,{}", path))
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = p;
        return Err("非 Windows 平台暂未支持".into());
    }
    Ok(())
}

// ---------- 开机自启（HKCU\...\Run，便携软件不需要管理员权限） ----------

#[cfg(target_os = "windows")]
const AUTOSTART_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
#[cfg(target_os = "windows")]
const AUTOSTART_NAME: &str = "ForgeStudio";

/// 查询当前是否已设置开机自启
#[tauri::command]
pub fn get_autostart() -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        let out = std::process::Command::new("reg")
            .args(["query", AUTOSTART_KEY, "/v", AUTOSTART_NAME])
            .output()
            .map_err(|e| e.to_string())?;
        Ok(out.status.success())
    }
    #[cfg(not(target_os = "windows"))]
    Ok(false)
}

/// 开启/关闭开机自启
#[tauri::command]
pub fn set_autostart(enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        if enabled {
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            // 加引号，兼容路径含空格
            let value = format!("\"{}\"", exe.to_string_lossy());
            let out = std::process::Command::new("reg")
                .args([
                    "add",
                    AUTOSTART_KEY,
                    "/v",
                    AUTOSTART_NAME,
                    "/t",
                    "REG_SZ",
                    "/d",
                    &value,
                    "/f",
                ])
                .output()
                .map_err(|e| e.to_string())?;
            if !out.status.success() {
                return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
            }
        } else {
            let out = std::process::Command::new("reg")
                .args(["delete", AUTOSTART_KEY, "/v", AUTOSTART_NAME, "/f"])
                .output()
                .map_err(|e| e.to_string())?;
            // 键本来不存在时 reg delete 会失败，视为已关闭
            if !out.status.success() && get_autostart()? {
                return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = enabled;
        return Err("非 Windows 平台暂未支持".into());
    }
    Ok(())
}
