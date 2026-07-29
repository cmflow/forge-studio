// projects.json：项目管理（骨架实现，业务细节后续补齐）
use std::path::Path;

use uuid::Uuid;

use crate::commands::scan::scan_dir;
use crate::models::{Project, ProjectStatus};
use crate::storage::{projects_path, read_json, write_json};

pub fn load_all() -> Result<Vec<Project>, String> {
    read_json::<Vec<Project>>(&projects_path())
}

pub fn save_all(list: &[Project]) -> Result<(), String> {
    write_json(&projects_path(), &list.to_vec())
}

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

#[tauri::command]
pub fn list_projects() -> Result<Vec<Project>, String> {
    load_all()
}

#[tauri::command]
pub fn add_project(path: String) -> Result<Project, String> {
    let p = Path::new(&path);
    if !p.exists() || !p.is_dir() {
        return Err("路径不存在或不是目录".into());
    }
    let name = p
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("Unnamed")
        .to_string();

    let (cbp_files, dcf_files) = scan_dir(p);
    let selected_cbp = cbp_files.first().cloned();
    let selected_dcf = dcf_files.first().cloned();

    let project = Project {
        id: Uuid::new_v4().to_string(),
        name,
        path: path.clone(),
        starred: false,
        last_accessed: now_ms(),
        cbp_files,
        dcf_files,
        selected_cbp,
        selected_dcf,
    };

    let mut list = load_all()?;
    list.push(project.clone());
    save_all(&list)?;
    Ok(project)
}

#[tauri::command]
pub fn remove_project(id: String) -> Result<(), String> {
    let mut list = load_all()?;
    list.retain(|p| p.id != id);
    save_all(&list)
}

#[tauri::command]
pub fn toggle_project_star(id: String) -> Result<(), String> {
    let mut list = load_all()?;
    for p in list.iter_mut() {
        if p.id == id {
            p.starred = !p.starred;
        }
    }
    save_all(&list)
}

#[tauri::command]
pub fn rename_project(id: String, new_name: String) -> Result<Project, String> {
    // TODO: 物理重命名 + 清空缓存 + 重扫（后续阶段实现）
    let mut list = load_all()?;
    let p = list
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or("项目不存在")?;
    p.name = new_name;
    p.last_accessed = now_ms();
    let result = p.clone();
    save_all(&list)?;
    Ok(result)
}

#[tauri::command]
pub fn scan_project(id: String) -> Result<Project, String> {
    let mut list = load_all()?;
    let p = list
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or("项目不存在")?;
    let (cbp, dcf) = scan_dir(Path::new(&p.path));
    p.cbp_files = cbp;
    p.dcf_files = dcf;
    // 重扫后重置 selected（按需求：旧文件可能已删）
    p.selected_cbp = None;
    p.selected_dcf = None;
    let result = p.clone();
    save_all(&list)?;
    Ok(result)
}

#[tauri::command]
pub fn select_cbp(id: String, path: String) -> Result<(), String> {
    let mut list = load_all()?;
    for p in list.iter_mut() {
        if p.id == id {
            p.selected_cbp = Some(path.clone());
        }
    }
    save_all(&list)
}

#[tauri::command]
pub fn select_dcf(id: String, path: String) -> Result<(), String> {
    let mut list = load_all()?;
    for p in list.iter_mut() {
        if p.id == id {
            p.selected_dcf = Some(path.clone());
        }
    }
    save_all(&list)
}

#[tauri::command]
pub async fn duplicate_project(id: String) -> Result<Project, String> {
    // TODO: 使用 tokio::task::spawn_blocking + tokio::time::timeout(120s)
    // 此处仅占位返回错误，避免误用
    let _ = id;
    Err("复制副本功能尚未实现".into())
}

#[tauri::command]
pub fn check_projects() -> Result<Vec<ProjectStatus>, String> {
    let list = load_all()?;
    Ok(list
        .iter()
        .map(|p| ProjectStatus {
            id: p.id.clone(),
            exists: Path::new(&p.path).exists(),
        })
        .collect())
}
