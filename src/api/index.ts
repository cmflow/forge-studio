// 统一封装 Tauri invoke 调用
import { invoke } from "@tauri-apps/api/core";
import type {
  AppConfig,
  Launcher,
  OpenKind,
  ProgressEvent,
  Project,
  ProjectStatus,
  PushOutcome,
  RemoteArchive,
  SyncDiagnostic,
  SyncSettings,
  WebdavCredential,
} from "../types";

// ---------- Config ----------
export const loadConfig = () => invoke<AppConfig>("load_config");
export const saveConfig = (config: AppConfig) =>
  invoke<void>("save_config", { config });
export const detectToolPath = (
  kind: "vscode" | "codeblocks" | "burn" | "trae",
) => invoke<string | null>("detect_tool_path", { kind });

/** 扫描指定根目录下的工具子目录（每子目录取第一个 .exe），已存在则跳过；返回本次新增的列表 */
export const scanDevUtils = (root: string) =>
  invoke<Launcher[]>("scan_dev_utils", { root });

// ---------- Launcher ----------
export const listLaunchers = () => invoke<Launcher[]>("list_launchers");
export const addLauncher = (name: string, path: string) =>
  invoke<Launcher>("add_launcher", { name, path });
export const removeLauncher = (id: string) =>
  invoke<void>("remove_launcher", { id });
export const toggleLauncherStar = (id: string) =>
  invoke<void>("toggle_launcher_star", { id });
export const runLauncher = (id: string) => invoke<void>("run_launcher", { id });

// ---------- Project ----------
export const listProjects = () => invoke<Project[]>("list_projects");
export const addProject = (path: string) =>
  invoke<Project>("add_project", { path });
export const removeProject = (id: string) =>
  invoke<void>("remove_project", { id });
export const toggleProjectStar = (id: string) =>
  invoke<void>("toggle_project_star", { id });
export const renameProject = (id: string, newName: string) =>
  invoke<Project>("rename_project", { id, newName });
export const scanProject = (id: string) =>
  invoke<Project>("scan_project", { id });
export const selectCbp = (id: string, path: string) =>
  invoke<void>("select_cbp", { id, path });
export const selectDcf = (id: string, path: string) =>
  invoke<void>("select_dcf", { id, path });
export const duplicateProject = (id: string) =>
  invoke<Project>("duplicate_project", { id });
export const checkProjects = () => invoke<ProjectStatus[]>("check_projects");

// ---------- Open ----------
export const openTarget = (kind: OpenKind, projectId: string) =>
  invoke<void>("open_target", { kind, projectId });

// ---------- Event（事件进展） ----------
export const listEvents = () => invoke<ProgressEvent[]>("list_events");
export const addEvent = (title: string, note?: string, category?: string) =>
  invoke<ProgressEvent>("add_event", { title, note, category });
export const updateEvent = (
  id: string,
  payload: { title?: string; note?: string },
) => invoke<ProgressEvent>("update_event", { id, ...payload });
export const removeEvent = (id: string) =>
  invoke<void>("remove_event", { id });
/** 设置分类，传空字符串归回「未分类」 */
export const setEventCategory = (id: string, category: string) =>
  invoke<ProgressEvent>("set_event_category", { id, category });
export const toggleEventStar = (id: string) =>
  invoke<ProgressEvent>("toggle_event_star", { id });
export const toggleEventStatus = (id: string) =>
  invoke<ProgressEvent>("toggle_event_status", { id });
export const addStep = (eventId: string, text: string) =>
  invoke<ProgressEvent>("add_step", { eventId, text });
export const cycleStepState = (eventId: string, stepId: string) =>
  invoke<ProgressEvent>("cycle_step_state", { eventId, stepId });
export const removeStep = (eventId: string, stepId: string) =>
  invoke<ProgressEvent>("remove_step", { eventId, stepId });

// ---------- Sync（坚果云 WebDAV） ----------
export const getSyncCredential = () =>
  invoke<WebdavCredential>("get_sync_credential");
export const setSyncCredential = (credential: WebdavCredential) =>
  invoke<void>("set_sync_credential", { credential });
/** 连通性自检：鉴权 → 建目录 → 上传 → 下载 → 清理 */
export const diagnoseSync = (remoteDir: string) =>
  invoke<SyncDiagnostic>("diagnose_sync", { remoteDir });
/** 当前设备标识与显示名 [device_id, device_name] */
export const getDeviceInfo = () => invoke<[string, string]>("get_device_info");
/** 上传本机存档。force=true 跳过篡改检测强制覆盖 */
export const pushEvents = (force = false) =>
  invoke<PushOutcome>("push_events", { force });
/** 下载指定设备的存档并覆盖本地，同时另存为本机存档 */
export const pullEvents = (device: string) =>
  invoke<PushOutcome>("pull_events", { device });
/** 列出云端所有设备存档 */
export const listRemoteArchives = () =>
  invoke<RemoteArchive[]>("list_remote_archives");
export const getSyncSettings = () => invoke<SyncSettings>("get_sync_settings");
/** 只改自动上传开关，不碰其它同步设置 */
export const setSyncAutoPush = (enabled: boolean) =>
  invoke<void>("set_sync_auto_push", { enabled });

// ---------- Misc ----------
export const openLogsDir = () => invoke<void>("open_logs_dir");
export const clearAllData = () => invoke<void>("clear_all_data");
export const revealInExplorer = (path: string) =>
  invoke<void>("reveal_in_explorer", { path });

export const getAutostart = () => invoke<boolean>("get_autostart");

export const setAutostart = (enabled: boolean) =>
  invoke<void>("set_autostart", { enabled });

// ---------- Icon ----------
export const getLauncherIcon = (path: string, size = 32) =>
  invoke<string>("get_launcher_icon", { path, size });
