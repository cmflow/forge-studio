// projects.json：项目管理
use std::path::{Path, PathBuf};
use std::time::Duration;

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

/// 非法字符集：Windows 文件名禁用 \\ / : * ? " < > |
const INVALID_CHARS: &[char] = &['\\', '/', ':', '*', '?', '"', '<', '>', '|'];

fn validate_name(name: &str) -> Result<(), String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("名称不能为空".into());
    }
    if trimmed.chars().any(|c| INVALID_CHARS.contains(&c)) {
        return Err(format!("名称含有非法字符 {:?}", INVALID_CHARS));
    }
    Ok(())
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

/// 物理重命名文件夹 + 清空缓存 + 自动重扫
#[tauri::command]
pub fn rename_project(id: String, new_name: String) -> Result<Project, String> {
    validate_name(&new_name)?;
    let new_name = new_name.trim().to_string();

    let mut list = load_all()?;
    let idx = list
        .iter()
        .position(|p| p.id == id)
        .ok_or("项目不存在")?;

    let old_path = PathBuf::from(&list[idx].path);
    if !old_path.exists() {
        return Err("原路径不存在，无法重命名".into());
    }

    // 同名直接返回
    if list[idx].name == new_name {
        return Ok(list[idx].clone());
    }

    let parent = old_path
        .parent()
        .ok_or("原路径没有父目录")?
        .to_path_buf();
    let new_path = parent.join(&new_name);

    if new_path.exists() {
        return Err(format!("目标已存在: {}", new_path.display()));
    }

    std::fs::rename(&old_path, &new_path).map_err(|e| format!("重命名失败: {}", e))?;

    // 更新字段 + 清空 selected + 重扫
    let (cbp, dcf) = scan_dir(&new_path);
    let selected_cbp = cbp.first().cloned();
    let selected_dcf = dcf.first().cloned();

    let p = &mut list[idx];
    p.name = new_name;
    p.path = new_path.to_string_lossy().to_string();
    p.cbp_files = cbp;
    p.dcf_files = dcf;
    p.selected_cbp = selected_cbp;
    p.selected_dcf = selected_dcf;
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
    // 重扫后重置 selected（旧文件可能已删）
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

/// 递归复制目录（阻塞）
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        std::fs::create_dir_all(dst)?;
    }
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else if ty.is_file() {
            std::fs::copy(&from, &to)?;
        }
        // symlink 及其他类型跳过
    }
    Ok(())
}

/// 生成不冲突的副本路径：original_copy / original_copy_1 / _2 ...
fn resolve_copy_path(src: &Path) -> Result<PathBuf, String> {
    let parent = src.parent().ok_or("原路径没有父目录")?;
    let name = src
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("原文件夹名无效")?;

    let base = format!("{}_copy", name);
    let mut candidate = parent.join(&base);
    let mut i = 1u32;
    while candidate.exists() {
        candidate = parent.join(format!("{}_{}", base, i));
        i += 1;
        if i > 9999 {
            return Err("副本命名达到上限".into());
        }
    }
    Ok(candidate)
}

/// 复制副本：spawn_blocking + timeout(120s)
#[tauri::command]
pub async fn duplicate_project(id: String) -> Result<Project, String> {
    let list = load_all()?;
    let src_project = list
        .iter()
        .find(|p| p.id == id)
        .cloned()
        .ok_or("项目不存在")?;

    let src_path = PathBuf::from(&src_project.path);
    if !src_path.exists() {
        return Err("原路径不存在".into());
    }
    let dst_path = resolve_copy_path(&src_path)?;

    let src_clone = src_path.clone();
    let dst_clone = dst_path.clone();

    let copy_handle =
        tokio::task::spawn_blocking(move || copy_dir_recursive(&src_clone, &dst_clone));

    match tokio::time::timeout(Duration::from_secs(120), copy_handle).await {
        Ok(join_res) => match join_res {
            Ok(io_res) => io_res.map_err(|e| format!("复制失败: {}", e))?,
            Err(join_err) => return Err(format!("复制任务异常: {}", join_err)),
        },
        Err(_) => {
            return Err("复制超时（120 秒），后台线程仍在继续，请稍后手动检查".into());
        }
    };

    // 复制成功：作为新项目添加并扫描
    let (cbp, dcf) = scan_dir(&dst_path);
    let selected_cbp = cbp.first().cloned();
    let selected_dcf = dcf.first().cloned();
    let new_name = dst_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("Unnamed")
        .to_string();

    let new_proj = Project {
        id: Uuid::new_v4().to_string(),
        name: new_name,
        path: dst_path.to_string_lossy().to_string(),
        starred: false,
        last_accessed: now_ms(),
        cbp_files: cbp,
        dcf_files: dcf,
        selected_cbp,
        selected_dcf,
    };

    let mut list = load_all()?;
    list.push(new_proj.clone());
    save_all(&list)?;
    Ok(new_proj)
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
