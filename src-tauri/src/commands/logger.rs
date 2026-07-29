// 日志记录：按日期切割写入 Data/logs/YYYY-MM-DD.log
use std::fs::OpenOptions;
use std::io::Write;

use crate::storage::logs_dir;

/// 内部工具：任意 Rust 代码可调用
pub fn log_line(msg: &str) -> Result<(), String> {
    let now = chrono::Local::now();
    let filename = format!("{}.log", now.format("%Y-%m-%d"));
    let path = logs_dir().join(filename);
    let line = format!("[{}] {}\n", now.format("%H:%M:%S"), msg);

    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| e.to_string())?;
    f.write_all(line.as_bytes()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn append_log(message: String) -> Result<(), String> {
    log_line(&message)
}
