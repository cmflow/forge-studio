// events.json：事件进展（独立模块，与 project / launcher 无耦合）
use uuid::Uuid;

use crate::models::{ProgressEvent, ProgressStep};
use crate::storage::{events_path, read_json, write_json};

fn load_all() -> Result<Vec<ProgressEvent>, String> {
    read_json::<Vec<ProgressEvent>>(&events_path())
}

fn save_all(list: &[ProgressEvent]) -> Result<(), String> {
    write_json(&events_path(), &list.to_vec())
}

fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

/// 取出可变引用，找不到就报错，避免各处重复写 ok_or
fn find_mut<'a>(
    list: &'a mut [ProgressEvent],
    id: &str,
) -> Result<&'a mut ProgressEvent, String> {
    list.iter_mut().find(|e| e.id == id).ok_or("事件不存在".into())
}

#[tauri::command]
pub fn list_events() -> Result<Vec<ProgressEvent>, String> {
    load_all()
}

#[tauri::command]
pub fn add_event(title: String, note: Option<String>) -> Result<ProgressEvent, String> {
    let title = title.trim().to_string();
    if title.is_empty() {
        return Err("事件标题不能为空".into());
    }
    let ts = now_ms();
    let item = ProgressEvent {
        id: Uuid::new_v4().to_string(),
        title,
        note: note.unwrap_or_default().trim().to_string(),
        category: String::new(),
        status: "open".into(),
        starred: false,
        created_at: ts,
        updated_at: ts,
        steps: Vec::new(),
    };
    let mut list = load_all()?;
    list.push(item.clone());
    save_all(&list)?;
    Ok(item)
}

/// 设置事件分类，传空字符串即归回「未分类」
#[tauri::command]
pub fn set_event_category(id: String, category: String) -> Result<ProgressEvent, String> {
    let mut list = load_all()?;
    let ev = find_mut(&mut list, &id)?;
    ev.category = category.trim().to_string();
    ev.updated_at = now_ms();
    let updated = ev.clone();
    save_all(&list)?;
    Ok(updated)
}

#[tauri::command]
pub fn update_event(
    id: String,
    title: Option<String>,
    note: Option<String>,
) -> Result<ProgressEvent, String> {
    let mut list = load_all()?;
    let ev = find_mut(&mut list, &id)?;
    if let Some(t) = title {
        let t = t.trim().to_string();
        if t.is_empty() {
            return Err("事件标题不能为空".into());
        }
        ev.title = t;
    }
    if let Some(n) = note {
        ev.note = n.trim().to_string();
    }
    ev.updated_at = now_ms();
    let updated = ev.clone();
    save_all(&list)?;
    Ok(updated)
}

#[tauri::command]
pub fn remove_event(id: String) -> Result<(), String> {
    let mut list = load_all()?;
    list.retain(|e| e.id != id);
    save_all(&list)
}

#[tauri::command]
pub fn toggle_event_star(id: String) -> Result<(), String> {
    let mut list = load_all()?;
    let ev = find_mut(&mut list, &id)?;
    ev.starred = !ev.starred;
    ev.updated_at = now_ms();
    save_all(&list)
}

/// 归档 / 取消归档：done <-> open
#[tauri::command]
pub fn toggle_event_status(id: String) -> Result<ProgressEvent, String> {
    let mut list = load_all()?;
    let ev = find_mut(&mut list, &id)?;
    ev.status = if ev.status == "done" { "open".into() } else { "done".into() };
    ev.updated_at = now_ms();
    let updated = ev.clone();
    save_all(&list)?;
    Ok(updated)
}

/// 追加一个进展节点，默认 state = "doing"，并把上一个 doing 节点收敛为 done，
/// 从而天然形成「一步步推进」的流程线。
#[tauri::command]
pub fn add_step(event_id: String, text: String) -> Result<ProgressEvent, String> {
    let text = text.trim().to_string();
    if text.is_empty() {
        return Err("进展内容不能为空".into());
    }
    let mut list = load_all()?;
    let ev = find_mut(&mut list, &event_id)?;
    for s in ev.steps.iter_mut() {
        if s.state == "doing" {
            s.state = "done".into();
        }
    }
    ev.steps.push(ProgressStep {
        id: Uuid::new_v4().to_string(),
        text,
        state: "doing".into(),
        created_at: now_ms(),
    });
    ev.updated_at = now_ms();
    let updated = ev.clone();
    save_all(&list)?;
    Ok(updated)
}

/// 切换节点状态：doing -> done -> pending -> doing
#[tauri::command]
pub fn cycle_step_state(event_id: String, step_id: String) -> Result<ProgressEvent, String> {
    let mut list = load_all()?;
    let ev = find_mut(&mut list, &event_id)?;
    let step = ev
        .steps
        .iter_mut()
        .find(|s| s.id == step_id)
        .ok_or("进展节点不存在")?;
    step.state = match step.state.as_str() {
        "doing" => "done".into(),
        "done" => "pending".into(),
        _ => "doing".into(),
    };
    ev.updated_at = now_ms();
    let updated = ev.clone();
    save_all(&list)?;
    Ok(updated)
}

#[tauri::command]
pub fn remove_step(event_id: String, step_id: String) -> Result<ProgressEvent, String> {
    let mut list = load_all()?;
    let ev = find_mut(&mut list, &event_id)?;
    ev.steps.retain(|s| s.id != step_id);
    ev.updated_at = now_ms();
    let updated = ev.clone();
    save_all(&list)?;
    Ok(updated)
}
