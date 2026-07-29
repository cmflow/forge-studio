// 统一封装 Tauri invoke 调用
import { invoke } from "@tauri-apps/api/core";
import type {
  AppConfig,
  Launcher,
  OpenKind,
  Project,
  ProjectStatus,
} from "../types";

// ---------- Config ----------
export const loadConfig = () => invoke<AppConfig>("load_config");
export const saveConfig = (config: AppConfig) =>
  invoke<void>("save_config", { config });

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

// ---------- Misc ----------
export const openLogsDir = () => invoke<void>("open_logs_dir");
export const clearAllData = () => invoke<void>("clear_all_data");
