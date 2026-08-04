// 智能扫描：定位到 `projects` 目录后，只在其中递归查 .cbp / .dcf
// 规则（针对 STM32 / 嵌入式 SDK 工程）：
//   1. `.cbp` / `.dcf` 一定位于名为 `projects` 的目录内（层级不定）
//   2. `.dcf` 一定位于 `projects/**/bin/` 之下
//   3. 需要排除 obj/.git/node_modules 等产物目录；但 bin 不排除（dcf 在里面）
use std::path::{Path, PathBuf};

use walkdir::{DirEntry, WalkDir};

const MAX_FILES: usize = 50000;
const MAX_PROJECTS_DIR_DEPTH: usize = 6; // 从根目录查找 projects 的最大深度

/// 需要跳过的目录名（不区分大小写）。注意：`bin` 保留，因为 dcf 就在 bin 里。
const SKIP_DIRS: &[&str] = &[
    ".git",
    ".svn",
    ".hg",
    ".vs",
    ".vscode",
    ".idea",
    "node_modules",
    "target",
    "build",
    "out",
    "dist",
    "obj",
    "debug",
    "release",
    "cmake-build-debug",
    "cmake-build-release",
    "__pycache__",
];

fn is_skipped_dir(entry: &DirEntry) -> bool {
    if !entry.file_type().is_dir() {
        return false;
    }
    // 起始目录本身不跳过（walkdir 中 depth==0 表示 walker 起点）
    if entry.depth() == 0 {
        return false;
    }
    let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
    SKIP_DIRS.iter().any(|d| *d == name)
}

/// 在 root 下查找所有名为 `projects` 的目录（大小写不敏感），限深度避免爬穿。
fn find_projects_dirs(root: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let walker = WalkDir::new(root)
        .max_depth(MAX_PROJECTS_DIR_DEPTH)
        .into_iter()
        .filter_entry(|e| !is_skipped_dir(e));
    for entry in walker.filter_map(|e| e.ok()) {
        if !entry.file_type().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
        if name == "projects" {
            result.push(entry.path().to_path_buf());
        }
    }
    // 若整个工程本身就叫 projects（用户直接把 projects 添加为项目）也支持
    if result.is_empty() {
        let self_name = root
            .file_name()
            .map(|s| s.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default();
        if self_name == "projects" {
            result.push(root.to_path_buf());
        }
    }
    result
}

pub fn scan_dir(root: &Path) -> (Vec<String>, Vec<String>) {
    let mut cbp = Vec::new();
    let mut dcf = Vec::new();
    if !root.exists() {
        return (cbp, dcf);
    }

    let projects_dirs = find_projects_dirs(root);
    if projects_dirs.is_empty() {
        return (cbp, dcf);
    }

    let mut file_count = 0usize;
    'outer: for pdir in &projects_dirs {
        let walker = WalkDir::new(pdir)
            .into_iter()
            .filter_entry(|e| !is_skipped_dir(e));
        for entry in walker.filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            file_count += 1;
            if file_count > MAX_FILES {
                break 'outer;
            }
            let path = entry.path();
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            match ext.to_ascii_lowercase().as_str() {
                "cbp" => cbp.push(path.to_string_lossy().to_string()),
                // .dcf 必须在名为 bin 的目录之下
                "dcf" if path
                    .parent()
                    .and_then(|p| p.file_name())
                    .map(|n| n.to_string_lossy().eq_ignore_ascii_case("bin"))
                    .unwrap_or(false) =>
                {
                    dcf.push(path.to_string_lossy().to_string());
                }
                _ => {}
            }
        }
    }

    (cbp, dcf)
}
