// 前后端共享的数据结构类型定义
// 与 src-tauri/src/models.rs 中的结构体保持一致

export interface AppConfig {
  vscode_path: string;
  codeblocks_path: string;
  burn_tool_path: string;
  trae_path: string;
  /** "vscode" | "trae"，为空按 vscode 处理 */
  default_ide: string;
  /** 自动扫描工具的根目录，每层子目录视为一个工具（取里面第一个 .exe） */
  dev_utils_root: string;
  /** 启动应用时自动扫描 dev_utils_root */
  scan_dev_utils_on_start: boolean;
  /** 坚果云云端目录（如 apps/forge-studio） */
  sync_remote_dir: string;
  /** 是否启用坚果云同步 */
  sync_enabled: boolean;
  /** 是否每 10 分钟自动上传本机存档 */
  sync_auto_push: boolean;
}

export interface Launcher {
  id: string;
  name: string;
  path: string;
  starred: boolean;
}

export interface Project {
  id: string;
  name: string;
  path: string;
  starred: boolean;
  last_accessed: number;
  cbp_files: string[];
  dcf_files: string[];
  selected_cbp: string | null;
  selected_dcf: string | null;
}

/** 项目路径存在性检测结果 */
export interface ProjectStatus {
  id: string;
  exists: boolean;
}

/** open_target 的目标类型 */
export type OpenKind = "folder" | "vscode" | "codeblocks" | "burn";

// ---------- 云同步（坚果云 WebDAV） ----------

/** 跨项目共享的 WebDAV 凭据，存在 %USERPROFILE%\.cloudsync\credential.json */
export interface WebdavCredential {
  server: string;
  account: string;
  app_password: string;
}

export interface DiagStep {
  name: string;
  ok: boolean;
  detail: string;
}

export interface SyncDiagnostic {
  steps: DiagStep[];
  ok: boolean;
}

// ---------- 事件进展模块（独立于项目管理） ----------

/** 进展节点状态：待办 / 进行中 / 已完成 */
export type StepState = "pending" | "doing" | "done";

/** 事件状态：open 进行中 | done 已归档 */
export type EventStatus = "open" | "done";

export interface ProgressStep {
  id: string;
  text: string;
  state: StepState;
  created_at: number;
}

export interface ProgressEvent {
  id: string;
  title: string;
  note: string;
  /** 分类名，空字符串表示「未分类」 */
  category: string;
  status: EventStatus;
  /** 归档时间戳，0 表示未归档 */
  archived_at: number;
  starred: boolean;
  created_at: number;
  updated_at: number;
  steps: ProgressStep[];
}

// ---------- 云同步（坚果云 WebDAV） ----------

/** 云端一份设备存档的摘要 */
export interface RemoteArchive {
  device_id: string;
  device_name: string;
  updated_at: number;
  event_count: number;
  /** 是否为当前这台电脑的存档 */
  is_self: boolean;
  /** 指纹校验是否通过，false 表示被外部改动过 */
  intact: boolean;
}

/** 上传 / 下载结果 */
export interface PushOutcome {
  ok: boolean;
  message: string;
  /** 检测到云端存档被篡改 */
  tampered: boolean;
  updated_at: number;
}

export interface SyncSettings {
  remote_dir: string;
  enabled: boolean;
  auto_push: boolean;
}
