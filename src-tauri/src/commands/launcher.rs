// launchers.json：快捷应用列表
use std::path::Path;
use std::process::Command;

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
    let p = Path::new(&path);
    if !p.exists() {
        return Err("路径不存在".into());
    }
    let mut list = load_all()?;
    let final_name = if name.trim().is_empty() {
        p.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("App")
            .to_string()
    } else {
        name.trim().to_string()
    };
    let item = Launcher {
        id: Uuid::new_v4().to_string(),
        name: final_name,
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
    let item = list
        .iter()
        .find(|l| l.id == id)
        .cloned()
        .ok_or("启动器不存在")?;

    let result: Result<(), String> = (|| {
        if !Path::new(&item.path).exists() {
            return Err(format!("目标不存在: {}", item.path));
        }
        Command::new(&item.path)
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    })();

    let status = match &result {
        Ok(_) => "OK".to_string(),
        Err(e) => format!("FAIL: {}", e),
    };
    let _ = log_line(&format!(
        "run_launcher name=\"{}\" path=\"{}\" result={}",
        item.name, item.path, status
    ));
    result
}
