// 共享凭据：%USERPROFILE%\.cloudsync\credential.json
// 多个项目共用同一份坚果云账号信息，改密码只需改这一个文件。
use std::path::PathBuf;

use crate::storage::{read_json, write_json};
use crate::sync::webdav::WebdavCredential;

/// 跨项目共享的凭据目录（独立于 .forge-studio，便于其它项目复用）
fn cloudsync_dir() -> PathBuf {
    let home = std::env::var("USERPROFILE")
        .ok()
        .map(PathBuf::from)
        .or_else(|| std::env::var("HOME").ok().map(PathBuf::from))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let dir = home.join(".cloudsync");
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }
    dir
}

pub fn credential_path() -> PathBuf {
    cloudsync_dir().join("credential.json")
}

pub fn load_credential() -> Result<WebdavCredential, String> {
    read_json::<WebdavCredential>(&credential_path())
}

pub fn save_credential(cred: &WebdavCredential) -> Result<(), String> {
    write_json(&credential_path(), cred)
}
