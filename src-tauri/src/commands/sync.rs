// 同步相关 Command：凭据读写 + 连通性自检 + 每设备独立存档的上传/下载
//
// 设计要点：
// 1. 每台设备在云端有独立文件 events-<设备名>.json，彼此不覆盖，从根本上避免冲突
// 2. 上传前校验自己那份存档的指纹，被外部改动过就拒绝自动上传并提示
// 3. 下载可任选一份设备存档，下载后立即以本机名义另存一份，保证本机存档始终存在
// 4. 所有网络请求都放到 spawn_blocking，避免阻塞式 reqwest 卡住 UI
use serde::{Deserialize, Serialize};

use crate::commands::config::load_config;
use crate::commands::event::{load_all, now_ms, save_all};
use crate::storage::{config_path, write_json};
use crate::models::AppConfig;
use crate::sync::credential::{load_credential, save_credential};
use crate::sync::envelope::{
    archive_filename, device_from_filename, device_id, device_name, SyncEnvelope,
};
use crate::sync::webdav::{WebdavClient, WebdavCredential};

/// 自检结果：逐步返回，便于定位到底哪一环出问题
#[derive(Debug, Clone, Serialize)]
pub struct SyncDiagnostic {
    pub steps: Vec<DiagStep>,
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiagStep {
    pub name: String,
    pub ok: bool,
    pub detail: String,
}

impl DiagStep {
    fn pass(name: &str, detail: &str) -> Self {
        Self { name: name.into(), ok: true, detail: detail.into() }
    }
    fn fail(name: &str, detail: String) -> Self {
        Self { name: name.into(), ok: false, detail }
    }
}

/// 云端一份设备存档的摘要，用于下载前挑选
#[derive(Debug, Clone, Serialize)]
pub struct RemoteArchive {
    pub device_id: String,
    pub device_name: String,
    pub updated_at: i64,
    pub event_count: usize,
    /// 是否为当前这台电脑的存档
    pub is_self: bool,
    /// 指纹校验是否通过；false 表示该文件被外部改动过
    pub intact: bool,
}

/// 上传结果
#[derive(Debug, Clone, Serialize)]
pub struct PushOutcome {
    pub ok: bool,
    /// 被拒绝时说明原因，供前端提示
    pub message: String,
    /// 检测到远端存档被篡改
    pub tampered: bool,
    pub updated_at: i64,
}

// ---------- 基础配置 ----------

#[tauri::command]
pub fn get_sync_credential() -> Result<WebdavCredential, String> {
    load_credential()
}

#[tauri::command]
pub fn set_sync_credential(credential: WebdavCredential) -> Result<(), String> {
    save_credential(&credential)
}

/// 当前设备标识，前端展示用
#[tauri::command]
pub fn get_device_info() -> (String, String) {
    (device_id(), device_name())
}

/// 同步相关设置，独立于主配置弹窗单独读写
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSettings {
    pub remote_dir: String,
    pub enabled: bool,
    pub auto_push: bool,
}

#[tauri::command]
pub fn get_sync_settings() -> Result<SyncSettings, String> {
    let cfg = load_config()?;
    Ok(SyncSettings {
        remote_dir: remote_dir_of(&cfg),
        enabled: cfg.sync_enabled,
        auto_push: cfg.sync_auto_push,
    })
}

/// 只改「自动上传」开关。读-改-写，不碰其它字段，
/// 避免用旧快照把设置弹窗刚保存的 remote_dir / enabled 冲掉。
#[tauri::command]
pub fn set_sync_auto_push(enabled: bool) -> Result<(), String> {
    let mut cfg = load_config()?;
    cfg.sync_auto_push = enabled;
    write_json(&config_path(), &cfg)
}

/// 读取配置里的云端目录，未配置则回退到默认值
fn remote_dir_of(cfg: &AppConfig) -> String {
    let d = cfg.sync_remote_dir.trim().trim_matches('/');
    if d.is_empty() {
        "apps/forge-studio".to_string()
    } else {
        d.to_string()
    }
}

/// 建立客户端并返回 (客户端, 云端目录)
fn prepare() -> Result<(WebdavClient, String), String> {
    let cred = load_credential()?;
    if !cred.is_filled() {
        return Err("尚未配置坚果云凭据，请先在设置中填写并测试连接".into());
    }
    let cfg = load_config()?;
    let client = WebdavClient::new(&cred)?;
    Ok((client, remote_dir_of(&cfg)))
}

// ---------- 上传 ----------

/// 上传本机存档。force = false 时，若远端本机存档被外部改动过则拒绝上传。
#[tauri::command]
pub async fn push_events(force: bool) -> Result<PushOutcome, String> {
    tauri::async_runtime::spawn_blocking(move || push_blocking(force))
        .await
        .map_err(|e| format!("任务执行失败：{}", e))?
}

fn push_blocking(force: bool) -> Result<PushOutcome, String> {
    let (client, dir) = prepare()?;
    let dev = device_id();
    let path = format!("{}/{}", dir, archive_filename(&dev));

    // 篡改检测：先看远端自己那份是否自洽
    if !force {
        if let Ok(Some(body)) = client.get_text(&path) {
            if let Ok(remote) = serde_json::from_str::<SyncEnvelope>(&body) {
                if !remote.is_intact() {
                    return Ok(PushOutcome {
                        ok: false,
                        message: "云端本机存档的内容与指纹不一致，可能被外部修改过。\
                                  已停止自动上传，请确认后手动强制上传。"
                            .into(),
                        tampered: true,
                        updated_at: remote.updated_at,
                    });
                }
                // 设备标识被改写，同样视为异常
                if !remote.device_id.is_empty() && remote.device_id != dev {
                    return Ok(PushOutcome {
                        ok: false,
                        message: format!(
                            "云端本机存档的设备标识为「{}」，与当前设备「{}」不一致，已停止上传。",
                            remote.device_id, dev
                        ),
                        tampered: true,
                        updated_at: remote.updated_at,
                    });
                }
            }
        }
    }

    let events = load_all()?;
    let env = SyncEnvelope::build(dev, device_name(), events);
    let body = serde_json::to_string_pretty(&env).map_err(|e| e.to_string())?;
    client.put_text(&path, body)?;

    Ok(PushOutcome {
        ok: true,
        message: format!("已上传 {} 条事件", env.event_count),
        tampered: false,
        updated_at: env.updated_at,
    })
}

// ---------- 列出云端存档 ----------

/// 列出云端所有设备存档，供用户选择下载哪一份
#[tauri::command]
pub async fn list_remote_archives() -> Result<Vec<RemoteArchive>, String> {
    tauri::async_runtime::spawn_blocking(list_blocking)
        .await
        .map_err(|e| format!("任务执行失败：{}", e))?
}

fn list_blocking() -> Result<Vec<RemoteArchive>, String> {
    let (client, dir) = prepare()?;
    let self_dev = device_id();
    let entries = client.list_dir(&dir)?;

    let mut out = Vec::new();
    for e in entries {
        if e.is_dir {
            continue;
        }
        let Some(dev) = device_from_filename(&e.name) else {
            continue;
        };
        // 逐个下载解析。存档是小 JSON，设备数量也有限，成本可接受
        let body = match client.get_text(&format!("{}/{}", dir, e.name)) {
            Ok(Some(b)) => b,
            _ => continue,
        };
        let env: SyncEnvelope = match serde_json::from_str(&body) {
            Ok(v) => v,
            Err(_) => continue,
        };
        out.push(RemoteArchive {
            is_self: dev == self_dev,
            device_id: dev,
            device_name: if env.device_name.is_empty() {
                env.device_id.clone()
            } else {
                env.device_name.clone()
            },
            updated_at: env.updated_at,
            event_count: env.event_count,
            intact: env.is_intact(),
        });
    }
    // 最近更新的排前面
    out.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(out)
}

// ---------- 下载 ----------

/// 下载指定设备的存档覆盖本地，随后立刻以本机名义另存一份，
/// 保证「下载完本机在云端也有一份一模一样的存档」。
#[tauri::command]
pub async fn pull_events(device: String) -> Result<PushOutcome, String> {
    tauri::async_runtime::spawn_blocking(move || pull_blocking(device))
        .await
        .map_err(|e| format!("任务执行失败：{}", e))?
}

fn pull_blocking(device: String) -> Result<PushOutcome, String> {
    let (client, dir) = prepare()?;
    let path = format!("{}/{}", dir, archive_filename(&device));

    let body = client
        .get_text(&path)?
        .ok_or_else(|| format!("云端不存在设备「{}」的存档", device))?;
    let env: SyncEnvelope = serde_json::from_str(&body).map_err(|e| format!("存档解析失败：{}", e))?;

    // 内容与指纹不自洽说明被外部改动过，恢复会用不可信数据覆盖本机，直接拒绝
    if !env.is_intact() {
        return Err(
            "该存档内容与指纹不一致，可能被外部修改过。为保护本机数据，已取消恢复。".into(),
        );
    }

    // 覆盖本地
    save_all(&env.events)?;

    // 以本机名义另存一份，使本机存档与刚下载的内容一致
    let self_dev = device_id();
    let mine = SyncEnvelope::build(self_dev.clone(), device_name(), env.events.clone());
    let mine_body = serde_json::to_string_pretty(&mine).map_err(|e| e.to_string())?;
    let mine_path = format!("{}/{}", dir, archive_filename(&self_dev));
    // 另存失败不影响本地已完成的下载，只在提示里说明
    let saved = client.put_text(&mine_path, mine_body).is_ok();

    Ok(PushOutcome {
        ok: true,
        message: if saved {
            format!(
                "已从「{}」恢复 {} 条事件，并同步为本机存档",
                env.device_name, env.event_count
            )
        } else {
            format!(
                "已从「{}」恢复 {} 条事件，但本机存档写入失败，请稍后手动上传",
                env.device_name, env.event_count
            )
        },
        tampered: false,
        updated_at: now_ms(),
    })
}

// ---------- 连通性自检 ----------

/// 连通性自检：依次验证 鉴权 → 建目录 → 写 → 读 → 清理。
/// 同时把云端目录与启用状态写入配置。
#[tauri::command]
pub async fn diagnose_sync(remote_dir: String) -> Result<SyncDiagnostic, String> {
    tauri::async_runtime::spawn_blocking(move || diagnose_blocking(remote_dir))
        .await
        .map_err(|e| format!("任务执行失败：{}", e))?
}

fn diagnose_blocking(remote_dir: String) -> Result<SyncDiagnostic, String> {
    let cred = load_credential()?;
    let mut steps: Vec<DiagStep> = Vec::new();

    if !cred.is_filled() {
        steps.push(DiagStep::fail(
            "凭据检查",
            "服务器地址 / 账号 / 应用密码 有未填写项".into(),
        ));
        return Ok(SyncDiagnostic { steps, ok: false });
    }
    steps.push(DiagStep::pass("凭据检查", "已填写"));

    let client = match WebdavClient::new(&cred) {
        Ok(c) => c,
        Err(e) => {
            steps.push(DiagStep::fail("初始化客户端", e));
            return Ok(SyncDiagnostic { steps, ok: false });
        }
    };

    match client.check() {
        Ok(_) => steps.push(DiagStep::pass("连接与鉴权", "PROPFIND 根目录成功")),
        Err(e) => {
            steps.push(DiagStep::fail("连接与鉴权", e));
            return Ok(SyncDiagnostic { steps, ok: false });
        }
    }

    let dir = remote_dir.trim().trim_matches('/').to_string();
    if dir.is_empty() {
        steps.push(DiagStep::fail("远程目录", "远程目录不能为空".into()));
        return Ok(SyncDiagnostic { steps, ok: false });
    }

    match client.ensure_dir(&dir) {
        Ok(_) => steps.push(DiagStep::pass("创建目录", &format!("{} 可用", dir))),
        Err(e) => {
            steps.push(DiagStep::fail("创建目录", e));
            steps.push(DiagStep::fail(
                "提示",
                "若报父目录不存在，请先在坚果云网页端手动创建第一层文件夹".into(),
            ));
            return Ok(SyncDiagnostic { steps, ok: false });
        }
    }

    // 探针文件不用点号开头，避免网页端隐藏导致残留难以发现
    let probe = format!("{}/_probe-forge-studio.json", dir);
    let payload = r#"{"probe":"forge-studio","ok":true}"#.to_string();
    match client.put_text(&probe, payload.clone()) {
        Ok(_) => steps.push(DiagStep::pass("上传测试", "探针文件写入成功")),
        Err(e) => {
            steps.push(DiagStep::fail("上传测试", e));
            return Ok(SyncDiagnostic { steps, ok: false });
        }
    }

    match client.get_text(&probe) {
        Ok(Some(body)) if body.trim() == payload.trim() => {
            steps.push(DiagStep::pass("下载测试", "内容与上传一致"))
        }
        Ok(Some(_)) => steps.push(DiagStep::fail("下载测试", "读回内容与上传不一致".into())),
        Ok(None) => steps.push(DiagStep::fail("下载测试", "刚上传的文件读不到".into())),
        Err(e) => steps.push(DiagStep::fail("下载测试", e)),
    }

    match client.delete(&probe) {
        Ok(_) => steps.push(DiagStep::pass("清理", "已删除探针文件")),
        Err(e) => steps.push(DiagStep::fail("清理", e)),
    }

    let ok = steps.iter().all(|s| s.ok);

    // 自检通过则记住目录并启用同步，省去用户再点一次保存
    if ok {
        if let Ok(mut cfg) = load_config() {
            cfg.sync_remote_dir = dir;
            cfg.sync_enabled = true;
            let _ = write_json(&config_path(), &cfg);
        }
        steps.push(DiagStep::pass("保存配置", "已记录云端目录并启用同步"));
    }

    Ok(SyncDiagnostic { steps, ok })
}
