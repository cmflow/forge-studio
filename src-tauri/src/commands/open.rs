// 执行打开：按 kind 调用对应 exe（骨架：暂只记录日志，实际启动由后续阶段实现）
use crate::commands::logger::log_line;
use crate::models::OpenKind;

#[tauri::command]
pub fn open_target(kind: OpenKind, project_id: String) -> Result<(), String> {
    let kind_str = match kind {
        OpenKind::Folder => "folder",
        OpenKind::Vscode => "vscode",
        OpenKind::Codeblocks => "codeblocks",
        OpenKind::Burn => "burn",
    };
    let _ = log_line(&format!("open_target kind={} project_id={}", kind_str, project_id));
    // TODO: 通过 tauri_plugin_shell / std::process::Command 启动对应工具
    Ok(())
}
