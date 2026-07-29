pub mod commands;
pub mod models;
pub mod storage;

use commands::{config as cmd_config, launcher as cmd_launcher, logger as cmd_logger,
    misc as cmd_misc, open as cmd_open, project as cmd_project};

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
            // launcher
            cmd_launcher::list_launchers,
            cmd_launcher::add_launcher,
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
            cmd_project::select_cbp,
            cmd_project::select_dcf,
            cmd_project::duplicate_project,
            cmd_project::check_projects,
            // open
            cmd_open::open_target,
            // logger
            cmd_logger::append_log,
            // misc
            cmd_misc::open_logs_dir,
            cmd_misc::clear_all_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
