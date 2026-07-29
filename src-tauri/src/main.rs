// 禁用 Windows 上的控制台窗口
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    forge_studio_lib::run();
}
