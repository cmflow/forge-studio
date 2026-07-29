// 执行打开：按 kind 调用对应 exe / 打开文件夹
// 无论成功失败都写日志；同时刷新项目的 last_accessed
use std::path::Path;
use std::process::Command;

use crate::commands::logger::log_line;
use crate::commands::project::{load_all, save_all};
use crate::models::{AppConfig, OpenKind};
use crate::storage::{config_path, read_json};

fn touch_project_access(id: &str) {
    if let Ok(mut list) = load_all() {
        let now = chrono::Utc::now().timestamp_millis();
        for p in list.iter_mut() {
            if p.id == id {
                p.last_accessed = now;
            }
        }
        let _ = save_all(&list);
    }
}

fn spawn_exe(exe: &str, args: &[&str]) -> Result<(), String> {
    if exe.trim().is_empty() {
        return Err("未配置该工具的路径，请到『设置』填写".into());
    }
    if !Path::new(exe).exists() {
        return Err(format!("目标程序不存在: {}", exe));
    }
    Command::new(exe)
        .args(args)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn open_folder(path: &str) -> Result<(), String> {
    if !Path::new(path).exists() {
        return Err(format!("路径不存在: {}", path));
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(path)
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = path;
        Err("非 Windows 平台暂未支持".into())
    }
}

#[tauri::command]
pub fn open_target(kind: OpenKind, project_id: String) -> Result<(), String> {
    let list = load_all()?;
    let proj = list
        .iter()
        .find(|p| p.id == project_id)
        .ok_or("项目不存在")?
        .clone();

    let cfg: AppConfig = read_json(&config_path()).unwrap_or_default();

    let kind_str = match kind {
        OpenKind::Folder => "folder",
        OpenKind::Vscode => "vscode",
        OpenKind::Codeblocks => "codeblocks",
        OpenKind::Burn => "burn",
    };

    let result: Result<(), String> = match kind {
        OpenKind::Folder => open_folder(&proj.path),
        OpenKind::Vscode => spawn_exe(&cfg.vscode_path, &[proj.path.as_str()]),
        OpenKind::Codeblocks => {
            let target = proj
                .selected_cbp
                .clone()
                .or_else(|| proj.cbp_files.first().cloned());
            match target {
                Some(cbp) => spawn_exe(&cfg.codeblocks_path, &[cbp.as_str()]),
                None => Err("该项目下未找到 .cbp 文件".into()),
            }
        }
        OpenKind::Burn => {
            let target = proj
                .selected_dcf
                .clone()
                .or_else(|| proj.dcf_files.first().cloned());
            match target {
                Some(dcf) => spawn_exe(&cfg.burn_tool_path, &[dcf.as_str()]),
                None => Err("该项目下未找到 .dcf 文件".into()),
            }
        }
    };

    // 无论成败：写日志 + 刷新 last_accessed
    let status = match &result {
        Ok(_) => "OK".to_string(),
        Err(e) => format!("FAIL: {}", e),
    };
    let _ = log_line(&format!(
        "open_target project=\"{}\" kind={} path=\"{}\" result={}",
        proj.name, kind_str, proj.path, status
    ));
    touch_project_access(&project_id);

    result
}
