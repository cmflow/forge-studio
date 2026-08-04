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
