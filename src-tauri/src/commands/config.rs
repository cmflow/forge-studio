// config.json 读写
use std::path::PathBuf;

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

/// 展开环境变量占位（如 %LOCALAPPDATA%）后判断是否是存在的文件
fn resolve_env(path: &str) -> Option<PathBuf> {
    let mut s = path.to_string();
    // 处理 %VAR% 形式（Windows 风格）
    while let Some(l) = s.find('%') {
        let rest = &s[l + 1..];
        if let Some(r_rel) = rest.find('%') {
            let var_name = &rest[..r_rel];
            let value = std::env::var(var_name).ok()?;
            s = format!("{}{}{}", &s[..l], value, &rest[r_rel + 1..]);
        } else {
            break;
        }
    }
    let p = PathBuf::from(s);
    if p.exists() { Some(p) } else { None }
}

/// 常见候选安装路径（按优先级降序）
fn candidates(kind: &str) -> Vec<&'static str> {
    match kind {
        "vscode" => vec![
            r"%LOCALAPPDATA%\Programs\Microsoft VS Code\Code.exe",
            r"C:\Program Files\Microsoft VS Code\Code.exe",
            r"C:\Program Files (x86)\Microsoft VS Code\Code.exe",
            r"D:\Program Files\Microsoft VS Code\Code.exe",
            r"D:\Program Files (x86)\Microsoft VS Code\Code.exe",
            r"D:\Microsoft VS Code\Code.exe",
            r"D:\VSCode\Code.exe",
            r"%LOCALAPPDATA%\Programs\Microsoft VS Code Insiders\Code - Insiders.exe",
        ],
        "codeblocks" => vec![
            r"C:\Program Files\CodeBlocks\codeblocks.exe",
            r"C:\Program Files (x86)\CodeBlocks\codeblocks.exe",
            r"D:\Program Files\CodeBlocks\codeblocks.exe",
            r"D:\Program Files (x86)\CodeBlocks\codeblocks.exe",
            r"D:\CodeBlocks\codeblocks.exe",
            r"C:\CodeBlocks\codeblocks.exe",
        ],
        "trae" => vec![
            // Trae CN 与国际版；user-install 与 system-install 都覆盖
            r"%LOCALAPPDATA%\Programs\Trae CN\Trae CN.exe",
            r"%LOCALAPPDATA%\Programs\Trae\Trae.exe",
            r"C:\Program Files\Trae CN\Trae CN.exe",
            r"C:\Program Files\Trae\Trae.exe",
            r"C:\Program Files (x86)\Trae CN\Trae CN.exe",
            r"C:\Program Files (x86)\Trae\Trae.exe",
            r"D:\Program Files\Trae CN\Trae CN.exe",
            r"D:\Program Files\Trae\Trae.exe",
            r"D:\Trae CN\Trae CN.exe",
            r"D:\Trae\Trae.exe",
        ],
        "burn" => vec![
            r"C:\dev_utils\downloader_v3.5.0\Downloader.exe",
            r"D:\dev_utils\downloader_v3.5.0\Downloader.exe",
            r"C:\downloader_v3.5.0\Downloader.exe",
            r"D:\downloader_v3.5.0\Downloader.exe",
        ],
        _ => vec![],
    }
}

/// 自动识别本地已安装的工具路径；失败返回 Ok(None)，交由前端提示手动选择。
#[tauri::command]
pub fn detect_tool_path(kind: String) -> Result<Option<String>, String> {
    for c in candidates(&kind) {
        if let Some(p) = resolve_env(c) {
            return Ok(Some(p.to_string_lossy().to_string()));
        }
    }
    Ok(None)
}
