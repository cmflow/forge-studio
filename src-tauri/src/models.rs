// 数据结构定义（与前端 src/types/index.ts 保持一致）
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub vscode_path: String,
    #[serde(default)]
    pub codeblocks_path: String,
    #[serde(default)]
    pub burn_tool_path: String,
    #[serde(default)]
    pub trae_path: String,
    /// 默认 IDE："vscode" | "trae"，为空时按 vscode 处理
    #[serde(default)]
    pub default_ide: String,
    /// 自动扫描工具的根目录
    #[serde(default)]
    pub dev_utils_root: String,
    /// 启动应用时自动扫描 dev_utils_root
    #[serde(default)]
    pub scan_dev_utils_on_start: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Launcher {
    pub id: String,
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub starred: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub starred: bool,
    #[serde(default)]
    pub last_accessed: i64,
    #[serde(default)]
    pub cbp_files: Vec<String>,
    #[serde(default)]
    pub dcf_files: Vec<String>,
    #[serde(default)]
    pub selected_cbp: Option<String>,
    #[serde(default)]
    pub selected_dcf: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStatus {
    pub id: String,
    pub exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OpenKind {
    Folder,
    Vscode,
    Codeblocks,
    Burn,
}
