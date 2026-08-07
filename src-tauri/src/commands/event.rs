// events.json：事件进展（独立模块，与 project / launcher 无耦合）
use uuid::Uuid;

use crate::models::{EventStatus, ProgressEvent, ProgressStep, StepState};
use crate::storage::{events_path, read_json, write_json};

pub fn load_all() -> Result<Vec<ProgressEvent>, String> {
    read_json::<Vec<ProgressEvent>>(&events_path())
}

pub fn save_all(list: &[ProgressEvent]) -> Result<(), String> {
    write_json(&events_path(), list)
}

pub fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

/// 取出可变引用，找不到就报错，避免各处重复写 ok_or
fn find_mut<'a>(
    list: &'a mut [ProgressEvent],
    id: &str,
) -> Result<&'a mut ProgressEvent, String> {
    list.iter_mut().find(|e| e.id == id).ok_or("事件不存在".into())
}

/// 所有「修改单个事件」命令的统一入口：
/// 读 → 定位 → 变更 → 刷新 updated_at → 落盘 → 返回最新事件。
/// 强制每次修改都刷新 updated_at，杜绝「改完忘更新排序时间」这类 bug。
fn mutate_event(
    id: &str,
    f: impl FnOnce(&mut ProgressEvent) -> Result<(), String>,
) -> Result<ProgressEvent, String> {
    let mut list = load_all()?;
    let ev = find_mut(&mut list, id)?;
    f(ev)?;
    ev.updated_at = now_ms();
    let updated = ev.clone();
    save_all(&list)?;
    Ok(updated)
}

#[tauri::command]
pub fn list_events() -> Result<Vec<ProgressEvent>, String> {
    load_all()
}

#[tauri::command]
pub fn add_event(
    title: String,
    note: Option<String>,
    category: Option<String>,
) -> Result<ProgressEvent, String> {
    let title = title.trim().to_string();
    if title.is_empty() {
        return Err("事件标题不能为空".into());
    }
    let ts = now_ms();
    let item = ProgressEvent {
        id: Uuid::new_v4().to_string(),
        title,
        note: note.unwrap_or_default().trim().to_string(),
        category: category.unwrap_or_default().trim().to_string(),
        status: EventStatus::Open,
        archived_at: 0,
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
    mutate_event(&id, |ev| {
        ev.category = category.trim().to_string();
        Ok(())
    })
}

#[tauri::command]
pub fn update_event(
    id: String,
    title: Option<String>,
    note: Option<String>,
) -> Result<ProgressEvent, String> {
    // 校验放在闭包外：标题为空直接拒绝，不产生任何写入
    if let Some(t) = &title {
        if t.trim().is_empty() {
            return Err("事件标题不能为空".into());
        }
    }
    mutate_event(&id, |ev| {
        if let Some(t) = title {
            ev.title = t.trim().to_string();
        }
        if let Some(n) = note {
            ev.note = n.trim().to_string();
        }
        Ok(())
    })
}

#[tauri::command]
pub fn remove_event(id: String) -> Result<(), String> {
    let mut list = load_all()?;
    list.retain(|e| e.id != id);
    save_all(&list)
}

#[tauri::command]
pub fn toggle_event_star(id: String) -> Result<ProgressEvent, String> {
    mutate_event(&id, |ev| {
        ev.starred = !ev.starred;
        Ok(())
    })
}

/// 归档 / 取消归档：done <-> open。归档时记录归档时间，取消归档则清零
#[tauri::command]
pub fn toggle_event_status(id: String) -> Result<ProgressEvent, String> {
    mutate_event(&id, |ev| {
        if ev.status == EventStatus::Done {
            ev.status = EventStatus::Open;
            ev.archived_at = 0;
        } else {
            ev.status = EventStatus::Done;
            ev.archived_at = now_ms();
        }
        Ok(())
    })
}

/// 追加一个进展节点，默认 doing，并把上一个 doing 节点收敛为 done，
/// 从而天然形成「一步步推进」的流程线。
#[tauri::command]
pub fn add_step(event_id: String, text: String) -> Result<ProgressEvent, String> {
    let text = text.trim().to_string();
    if text.is_empty() {
        return Err("进展内容不能为空".into());
    }
    mutate_event(&event_id, |ev| {
        for s in ev.steps.iter_mut() {
            if s.state == StepState::Doing {
                s.state = StepState::Done;
            }
        }
        ev.steps.push(ProgressStep {
            id: Uuid::new_v4().to_string(),
            text,
            state: StepState::Doing,
            created_at: now_ms(),
        });
        Ok(())
    })
}

/// 切换节点状态：doing -> done -> pending -> doing
/// 用枚举穷举匹配，状态机全在编译期保证，不存在的状态编译不过
#[tauri::command]
pub fn cycle_step_state(event_id: String, step_id: String) -> Result<ProgressEvent, String> {
    mutate_event(&event_id, |ev| {
        let step = ev
            .steps
            .iter_mut()
            .find(|s| s.id == step_id)
            .ok_or("进展节点不存在")?;
        step.state = match step.state {
            StepState::Doing => StepState::Done,
            StepState::Done => StepState::Pending,
            StepState::Pending => StepState::Doing,
        };
        Ok(())
    })
}

#[tauri::command]
pub fn remove_step(event_id: String, step_id: String) -> Result<ProgressEvent, String> {
    mutate_event(&event_id, |ev| {
        ev.steps.retain(|s| s.id != step_id);
        Ok(())
    })
}
