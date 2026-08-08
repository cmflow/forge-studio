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
        use std::os::windows::process::CommandExt;
        // explorer.exe 的 /select, 解析很挑剔：
        // 1. 路径含空格时，若让 Rust 自动给整个参数加引号，命令行会变成
        //    explorer "/select,D:\My Folder\foo.dcf"，explorer 无法识别 /select，
        //    回退到默认的"文档"目录——这就是"经常跳到文档目录"的根因。
        // 2. 用 raw_arg 精确控制命令行为：explorer /select,"D:\My Folder\foo.dcf"
        //    引号只包住路径部分，explorer 才能正确识别 /select, 并定位含空格的路径。
        // 3. 顺带把正斜杠统一为反斜杠（explorer 对正斜杠支持不稳）。
        let win_path = path.replace('/', "\\");
        std::process::Command::new("explorer")
            .raw_arg(format!("/select,\"{}\"", win_path))
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

/// 创建 reg.exe 子进程时禁止弹出控制台黑窗口
#[cfg(target_os = "windows")]
fn reg_command() -> std::process::Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let mut cmd = std::process::Command::new("reg");
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

/// 查询当前是否已设置开机自启
#[tauri::command]
pub async fn get_autostart() -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        // reg.exe 子进程启动较慢（可达数百毫秒），放到 spawn_blocking
        // 避免阻塞 Tauri 命令线程池，导致 UI 卡顿
        let ok = tauri::async_runtime::spawn_blocking(|| -> Result<bool, String> {
            let out = reg_command()
                .args(["query", AUTOSTART_KEY, "/v", AUTOSTART_NAME])
                .output()
                .map_err(|e| e.to_string())?;
            Ok(out.status.success())
        })
        .await
        .map_err(|e| format!("任务执行失败：{}", e))??;
        Ok(ok)
    }
    #[cfg(not(target_os = "windows"))]
    Ok(false)
}

/// 开启/关闭开机自启
#[tauri::command]
pub async fn set_autostart(enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // reg.exe 子进程同样较慢，放到 spawn_blocking 避免阻塞 UI
        tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
            if enabled {
                let exe = std::env::current_exe().map_err(|e| e.to_string())?;
                let value = format!("\"{}\"", exe.to_string_lossy());
                let out = reg_command()
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
                let out = reg_command()
                    .args(["delete", AUTOSTART_KEY, "/v", AUTOSTART_NAME, "/f"])
                    .output()
                    .map_err(|e| e.to_string())?;
                // 键本来不存在时 reg delete 会失败，视为已关闭
                if !out.status.success() {
                    let still = reg_command()
                        .args(["query", AUTOSTART_KEY, "/v", AUTOSTART_NAME])
                        .output()
                        .map_err(|e| e.to_string())?;
                    if still.status.success() {
                        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
                    }
                }
            }
            Ok(())
        })
        .await
        .map_err(|e| format!("任务执行失败：{}", e))??;
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = enabled;
        return Err("非 Windows 平台暂未支持".into());
    }
}
