pub mod commands;
pub mod models;
pub mod storage;
pub mod sync;

use commands::{config as cmd_config, event as cmd_event, icon as cmd_icon,
    launcher as cmd_launcher, logger as cmd_logger, misc as cmd_misc, open as cmd_open,
    project as cmd_project, sync as cmd_sync};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            // config
            cmd_config::load_config,
            cmd_config::save_config,
            cmd_config::detect_tool_path,
            // launcher
            cmd_launcher::list_launchers,
            cmd_launcher::add_launcher,
            cmd_launcher::scan_dev_utils,
            cmd_launcher::remove_launcher,
            cmd_launcher::toggle_launcher_star,
            cmd_launcher::run_launcher,
            // project
            cmd_project::list_projects,
            cmd_project::add_project,
            cmd_project::remove_project,
            cmd_project::toggle_project_star,
            cmd_project::rename_project,
            cmd_project::scan_project,
            cmd_project::rescan_all_projects,
            cmd_project::select_cbp,
            cmd_project::select_dcf,
            cmd_project::duplicate_project,
            cmd_project::check_projects,
            // open
            cmd_open::open_target,
            // event（事件进展模块）
            cmd_event::list_events,
            cmd_event::add_event,
            cmd_event::update_event,
            cmd_event::set_event_category,
            cmd_event::remove_event,
            cmd_event::toggle_event_star,
            cmd_event::toggle_event_status,
            cmd_event::add_step,
            cmd_event::cycle_step_state,
            cmd_event::remove_step,
            // logger
            cmd_logger::append_log,
            // misc
            cmd_misc::open_logs_dir,
            cmd_misc::clear_all_data,
            cmd_misc::reveal_in_explorer,
            cmd_misc::get_autostart,
            cmd_misc::set_autostart,
            // icon
            cmd_icon::get_launcher_icon,
            // sync（坚果云 WebDAV）
            cmd_sync::get_sync_credential,
            cmd_sync::set_sync_credential,
            cmd_sync::diagnose_sync,
            cmd_sync::get_device_info,
            cmd_sync::push_events,
            cmd_sync::pull_events,
            cmd_sync::list_remote_archives,
            cmd_sync::get_sync_settings,
            cmd_sync::set_sync_auto_push,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
