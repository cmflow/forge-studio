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

/// 扫描指定根目录下的所有"工具"：每层子目录视为一个工具，里面找到的第一个 .exe 即视为该工具。
/// 把扫到但 launchers.json 中不存在的工具自动加入；已存在（同 exe 文件名比较）则跳过。
/// 返回本次新增的项目，便于前端提示。
#[tauri::command]
pub fn scan_dev_utils(root: String) -> Result<Vec<Launcher>, String> {
    let root_path = Path::new(&root);
    if !root_path.exists() || !root_path.is_dir() {
        return Ok(vec![]);
    }

    let entries = match std::fs::read_dir(root_path) {
        Ok(e) => e,
        Err(_) => return Ok(vec![]),
    };

    let mut list = load_all()?;
    // 已有 exe 文件名（小写）集合，规避大小写/空格差异
    let existing: std::collections::HashSet<String> = list
        .iter()
        .map(|l| {
            Path::new(&l.path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_ascii_lowercase()
        })
        .collect();

    let mut added: Vec<Launcher> = Vec::new();
    for entry in entries.flatten() {
        let p = entry.path();
        if !p.is_dir() {
            continue;
        }
        // 在子目录中按文件名顺序找第一个 .exe，找到就停
        let Ok(sub) = std::fs::read_dir(&p) else { continue };
        let mut found_exe: Option<std::path::PathBuf> = None;
        for sub_entry in sub.flatten() {
            let sp = sub_entry.path();
            if sp.is_file() {
                if sp.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("exe"))
                    == Some(true)
                {
                    found_exe = Some(sp);
                    break;
                }
            }
        }
        let Some(exe) = found_exe else { continue };

        // 同名判断：以 exe 文件名（小写）为键
        let key = exe
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if key.is_empty() || existing.contains(&key) {
            continue;
        }

        let name = exe
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("App")
            .to_string();
        let item = Launcher {
            id: Uuid::new_v4().to_string(),
            name,
            path: exe.to_string_lossy().to_string(),
            starred: false,
        };
        list.push(item.clone());
        added.push(item);
    }

    if !added.is_empty() {
        save_all(&list)?;
    }
    Ok(added)
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
        let exe = Path::new(&item.path);
        if !exe.exists() {
            return Err(format!("目标不存在: {}", item.path));
        }
        let mut cmd = Command::new(exe);
        // 关键：把工作目录切到 exe 所在目录，避免第三方软件用相对路径找不到自己身边的
        // DLL / 授权 / 配置文件而弹出"运行环境异常"（相当于模拟双击时的 CWD）。
        if let Some(dir) = exe.parent() {
            if !dir.as_os_str().is_empty() {
                cmd.current_dir(dir);
            }
        }
        cmd.spawn().map(|_| ()).map_err(|e| e.to_string())
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
