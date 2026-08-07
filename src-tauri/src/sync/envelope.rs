// 同步信封：每台设备在云端有独立存档文件，互不覆盖。
//
// 篡改检测原理：写入时记录 fingerprint（对 events 内容做哈希）。
// 下次自动上传前先拉取自己那份存档，若其 fingerprint 与内容不匹配，
// 或 device_id 被改成别人，说明文件被外部改动过，此时停止自动上传，交由用户确认。
use serde::{Deserialize, Serialize};

use crate::models::ProgressEvent;

/// 云端单个设备存档的文件内容
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncEnvelope {
    #[serde(default)]
    pub version: u32,
    /// 设备唯一标识（计算机名），同时决定云端文件名
    #[serde(default)]
    pub device_id: String,
    /// 展示用设备名
    #[serde(default)]
    pub device_name: String,
    /// 本次写入时间（毫秒）
    #[serde(default)]
    pub updated_at: i64,
    /// 事件条数，便于列表展示时无需解析全部内容
    #[serde(default)]
    pub event_count: usize,
    /// 对 events 序列化结果的指纹，用于篡改检测
    #[serde(default)]
    pub fingerprint: String,
    #[serde(default)]
    pub events: Vec<ProgressEvent>,
}

impl SyncEnvelope {
    pub fn build(device_id: String, device_name: String, events: Vec<ProgressEvent>) -> Self {
        let fingerprint = fingerprint_of(&events);
        Self {
            version: 1,
            device_id,
            device_name,
            updated_at: chrono::Utc::now().timestamp_millis(),
            event_count: events.len(),
            fingerprint,
            events,
        }
    }

    /// 校验内容与指纹是否自洽。false 表示文件被外部改动过。
    pub fn is_intact(&self) -> bool {
        // 老版本或手工创建的文件没有指纹，不视为篡改，只是无法校验
        if self.fingerprint.is_empty() {
            return true;
        }
        // 兼容旧算法：切换为排序 key 之前上传的存档用的是「结构体字段序」指纹，
        // 两者都认，避免升级后所有云端旧存档被误判为篡改。
        self.fingerprint == fingerprint_of(&self.events)
            || self.fingerprint == fingerprint_of_legacy(&self.events)
    }
}

/// 旧版指纹算法（按结构体字段声明顺序序列化）。仅用于兼容历史存档的校验，
/// 新写入一律用 fingerprint_of。将来确认云端已无旧存档后可删除。
fn fingerprint_of_legacy(events: &[ProgressEvent]) -> String {
    let json = match serde_json::to_string(events) {
        Ok(s) => s,
        Err(_) => return String::new(),
    };
    fnv1a(&json)
}

/// 对事件列表算一个稳定指纹。用 FNV-1a，够快且无需额外依赖；
/// 目的是检测意外改动，不是防恶意伪造。
///
/// 序列化时先 to_value 再 to_string：serde_json 默认的 Map 是 BTreeMap，
/// key 天然按字典序排列，所以指纹与 ProgressEvent 的字段声明顺序无关。
/// 否则将来调整字段顺序会让所有云端旧存档集体误判为「被篡改」。
pub fn fingerprint_of(events: &[ProgressEvent]) -> String {
    let value = match serde_json::to_value(events) {
        Ok(v) => v,
        Err(_) => return String::new(),
    };
    let json = match serde_json::to_string(&value) {
        Ok(s) => s,
        Err(_) => return String::new(),
    };
    fnv1a(&json)
}

/// FNV-1a 64 位哈希，输出 16 位十六进制
fn fnv1a(s: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    format!("{:016x}", hash)
}

/// 设备标识：优先计算机名，取不到则用固定占位符。
/// 作为云端文件名的一部分，所以需要过滤非法字符。
pub fn device_id() -> String {
    let raw = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown-device".to_string());
    sanitize(&raw)
}

/// 计算机名原文，用于界面展示
pub fn device_name() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "未知设备".to_string())
}

/// 过滤文件名非法字符，避免拼出无效的 WebDAV 路径
fn sanitize(s: &str) -> String {
    let cleaned: String = s
        .trim()
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => c,
            _ => '-',
        })
        .collect();
    let trimmed = cleaned.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "unknown-device".to_string()
    } else {
        trimmed
    }
}

/// 云端存档文件名：events-<device>.json
pub fn archive_filename(device: &str) -> String {
    format!("events-{}.json", device)
}

/// 从文件名反解设备标识，非本项目的文件返回 None
pub fn device_from_filename(name: &str) -> Option<String> {
    name.strip_prefix("events-")
        .and_then(|s| s.strip_suffix(".json"))
        .map(|s| s.to_string())
}
