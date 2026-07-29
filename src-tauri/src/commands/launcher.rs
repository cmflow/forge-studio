// launchers.json：快捷应用列表（占位实现，仅落地基础 CRUD）
use uuid::Uuid;

use crate::commands::logger::log_line;
use crate::models::Launcher;
use crate::storage::{launchers_path, read_json, write_json};

fn load_all() -> Result<Vec<Launcher>, String> {
    read_json::<Vec<Launcher>>(&launchers_path())
}

fn save_all(list: &[Launcher]) -> Result<(), String> {
    write_json(&launchers_path(), &list.to_vec())
}

#[tauri::command]
pub fn list_launchers() -> Result<Vec<Launcher>, String> {
    load_all()
}

#[tauri::command]
pub fn add_launcher(name: String, path: String) -> Result<Launcher, String> {
    let mut list = load_all()?;
    let item = Launcher {
        id: Uuid::new_v4().to_string(),
        name,
        path,
        starred: false,
    };
    list.push(item.clone());
    save_all(&list)?;
    Ok(item)
}

#[tauri::command]
pub fn remove_launcher(id: String) -> Result<(), String> {
    let mut list = load_all()?;
    list.retain(|l| l.id != id);
    save_all(&list)
}

#[tauri::command]
pub fn toggle_launcher_star(id: String) -> Result<(), String> {
    let mut list = load_all()?;
    for l in list.iter_mut() {
        if l.id == id {
            l.starred = !l.starred;
        }
    }
    save_all(&list)
}

#[tauri::command]
pub fn run_launcher(id: String) -> Result<(), String> {
    let list = load_all()?;
    let item = list.iter().find(|l| l.id == id).ok_or("启动器不存在")?;
    // TODO: 使用 tauri_plugin_shell 打开；此处先记录日志占位
    let _ = log_line(&format!("run_launcher name={} path={}", item.name, item.path));
    Ok(())
}
