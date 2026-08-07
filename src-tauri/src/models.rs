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
    /// 云端存放目录，如 apps/forge-studio
    #[serde(default)]
    pub sync_remote_dir: String,
    /// 是否启用云同步
    #[serde(default)]
    pub sync_enabled: bool,
    /// 是否每 10 分钟自动上传本机存档
    #[serde(default)]
    pub sync_auto_push: bool,
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

/// 事件状态。序列化为 "open" | "done"，与旧数据格式一致，可直接读旧存档
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EventStatus {
    #[default]
    Open,
    Done,
}

/// 进展节点状态。序列化为 "pending" | "doing" | "done"，与旧数据格式一致
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StepState {
    Pending,
    #[default]
    Doing,
    Done,
}

/// 事件进展中的单个节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressStep {
    pub id: String,
    /// 节点描述，例如「已联系厂商确认参数」
    pub text: String,
    #[serde(default)]
    pub state: StepState,
    #[serde(default)]
    pub created_at: i64,
}

/// 一个「事件」，由若干进展节点串成处理流程
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEvent {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub note: String,
    /// 分类名，空字符串表示「未分类」
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub status: EventStatus,
    /// 归档时间戳，0 表示未归档。独立存储，避免被 updated_at 覆盖
    #[serde(default)]
    pub archived_at: i64,
    #[serde(default)]
    pub starred: bool,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub updated_at: i64,
    #[serde(default)]
    pub steps: Vec<ProgressStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OpenKind {
    Folder,
    Vscode,
    Codeblocks,
    Burn,
}
