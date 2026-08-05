// 同步相关 Command：凭据读写 + 连通性自检
use serde::Serialize;

use crate::sync::credential::{load_credential, save_credential};
use crate::sync::webdav::{WebdavClient, WebdavCredential};

/// 自检结果：逐步返回，便于定位到底哪一环出问题
#[derive(Debug, Clone, Serialize)]
pub struct SyncDiagnostic {
    /// 每一步的名称与结果描述
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
        Self {
            name: name.into(),
            ok: true,
            detail: detail.into(),
        }
    }
    fn fail(name: &str, detail: String) -> Self {
        Self {
            name: name.into(),
            ok: false,
            detail,
        }
    }
}

#[tauri::command]
pub fn get_sync_credential() -> Result<WebdavCredential, String> {
    load_credential()
}

#[tauri::command]
pub fn set_sync_credential(credential: WebdavCredential) -> Result<(), String> {
    save_credential(&credential)
}

/// 连通性自检：依次验证 鉴权 → 建目录 → 写 → 读 → 清理。
/// 这是阶段 1 的核心，用来确认坚果云的实际目录规则。
#[tauri::command]
pub fn diagnose_sync(remote_dir: String) -> Result<SyncDiagnostic, String> {
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

    // 1. 鉴权 + 连通
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

    // 2. 逐级建目录（验证坚果云是否允许 MKCOL 建第一层）
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

    // 3. 写入探针文件
    let probe = format!("{}/.forge-studio-probe.json", dir);
    let payload = r#"{"probe":"forge-studio","ok":true}"#.to_string();
    match client.put_text(&probe, payload.clone()) {
        Ok(_) => steps.push(DiagStep::pass("上传测试", "探针文件写入成功")),
        Err(e) => {
            steps.push(DiagStep::fail("上传测试", e));
            return Ok(SyncDiagnostic { steps, ok: false });
        }
    }

    // 4. 读回并比对
    match client.get_text(&probe) {
        Ok(Some(body)) if body.trim() == payload.trim() => {
            steps.push(DiagStep::pass("下载测试", "内容与上传一致"))
        }
        Ok(Some(_)) => steps.push(DiagStep::fail(
            "下载测试",
            "读回内容与上传不一致".into(),
        )),
        Ok(None) => steps.push(DiagStep::fail("下载测试", "刚上传的文件读不到".into())),
        Err(e) => steps.push(DiagStep::fail("下载测试", e)),
    }

    // 5. 清理探针（失败不影响整体结论）
    match client.delete(&probe) {
        Ok(_) => steps.push(DiagStep::pass("清理", "已删除探针文件")),
        Err(e) => steps.push(DiagStep::fail("清理", e)),
    }

    let ok = steps.iter().all(|s| s.ok);
    Ok(SyncDiagnostic { steps, ok })
}
