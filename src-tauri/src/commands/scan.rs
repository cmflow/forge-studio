// 智能扫描：递归 project 目录（限 1000 文件），筛选 .cbp / .dcf
use std::path::Path;

use walkdir::WalkDir;

const MAX_FILES: usize = 1000;

pub fn scan_dir(root: &Path) -> (Vec<String>, Vec<String>) {
    let mut cbp = Vec::new();
    let mut dcf = Vec::new();
    if !root.exists() {
        return (cbp, dcf);
    }
    let mut count = 0usize;
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        count += 1;
        if count > MAX_FILES {
            break;
        }
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        match ext.to_ascii_lowercase().as_str() {
            "cbp" => cbp.push(path.to_string_lossy().to_string()),
            "dcf" => dcf.push(path.to_string_lossy().to_string()),
            _ => {}
        }
    }
    (cbp, dcf)
}
