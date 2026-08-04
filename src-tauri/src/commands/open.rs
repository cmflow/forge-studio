// 执行打开：按 kind 调用对应 exe / 打开文件夹
// 无论成功失败都写日志；同时刷新项目的 last_accessed
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::commands::logger::log_line;
use crate::commands::project::{load_all, save_all};
use crate::models::{AppConfig, OpenKind};
use crate::storage::{config_path, read_json};

fn touch_project_access(id: &str) {
    if let Ok(mut list) = load_all() {
        let now = chrono::Utc::now().timestamp_millis();
        for p in list.iter_mut() {
            if p.id == id {
                p.last_accessed = now;
            }
        }
        let _ = save_all(&list);
    }
}

fn spawn_exe(exe: &str, args: &[&str]) -> Result<(), String> {
    if exe.trim().is_empty() {
        return Err("未配置该工具的路径，请到『设置』填写".into());
    }
    let exe_path = Path::new(exe);
    if !exe_path.exists() {
        return Err(format!("目标程序不存在: {}", exe));
    }
    let mut cmd = Command::new(exe_path);
    cmd.args(args);
    // 关键：切换工作目录到 exe 所在目录，避免部分软件（如烧录工具）用相对路径找不到
    // 自己身边的 DLL / license / 配置资源而弹出"运行环境异常"。
    if let Some(dir) = exe_path.parent() {
        if !dir.as_os_str().is_empty() {
            cmd.current_dir(dir);
        }
    }
    cmd.spawn().map(|_| ()).map_err(|e| e.to_string())
}

fn open_folder(path: &str) -> Result<(), String> {
    if !Path::new(path).exists() {
        return Err(format!("路径不存在: {}", path));
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(path)
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = path;
        Err("非 Windows 平台暂未支持".into())
    }
}

/// 定位烧录工具旁边的 .NET 配置文件（`Downloader.exe.config` 或 `Downloader.config`）。
fn find_burn_config(exe_path: &Path) -> Option<PathBuf> {
    let dir = exe_path.parent()?;
    let stem = exe_path.file_stem()?.to_str()?;
    let candidates = [
        dir.join(format!("{}.exe.config", stem)),
        dir.join(format!("{}.config", stem)),
    ];
    candidates.into_iter().find(|p| p.exists())
}

/// XML 属性值转义（Windows 路径通常不含这些字符，保险起见做基本转义）
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// 把 `<add key="DownFile" value="..." />` 中的 value 替换为新 dcf 路径。
/// 只改主键 DownFile，不动最近记录 DownFile1..DownFileN。
/// 返回 true 表示替换成功；false 表示 config 里没有 DownFile 项（应回退到命令行方式）。
fn patch_burn_config(config_path: &Path, new_dcf: &str) -> Result<bool, String> {
    let text = std::fs::read_to_string(config_path)
        .map_err(|e| format!("读取烧录工具配置失败: {}", e))?;

    // 定位 <add key="DownFile" ... />（注意：不能匹配到 DownFile1/DownFile2...）
    // 因为 XML 属性顺序理论上可变，这里稳妥用两步：先找到 key="DownFile" 边界，再找它属于的 <add ... /> 整段。
    let needle = "key=\"DownFile\"";
    let key_pos = match text.find(needle) {
        Some(p) => p,
        None => return Ok(false),
    };

    // 从 key_pos 往前找最近的 `<add `
    let tag_start = match text[..key_pos].rfind("<add") {
        Some(p) => p,
        None => return Ok(false),
    };
    // 从 tag_start 往后找 `/>` 或 `</add>` 结束
    let after = &text[tag_start..];
    let tag_end_rel = match after.find("/>").or_else(|| after.find("</add>")) {
        Some(p) => p,
        None => return Ok(false),
    };
    let tag_end = tag_start + tag_end_rel; // 指向 `/>` 或 `</add>` 起点

    // 在这一整个 <add .../> 片段里替换 value="..."
    let segment = &text[tag_start..tag_end];
    let value_key = "value=\"";
    let v_start_rel = match segment.find(value_key) {
        Some(p) => p + value_key.len(),
        None => return Ok(false),
    };
    let rest_after_v = &segment[v_start_rel..];
    let v_end_rel = match rest_after_v.find('"') {
        Some(p) => p,
        None => return Ok(false),
    };

    let abs_v_start = tag_start + v_start_rel;
    let abs_v_end = abs_v_start + v_end_rel;
    let mut new_text = String::with_capacity(text.len() + new_dcf.len());
    new_text.push_str(&text[..abs_v_start]);
    new_text.push_str(&xml_escape(new_dcf));
    new_text.push_str(&text[abs_v_end..]);

    std::fs::write(config_path, new_text)
        .map_err(|e| format!("写入烧录工具配置失败: {}", e))?;
    Ok(true)
}

/// 启动烧录工具打开指定 dcf。
/// 策略：先尝试改 `<exe>.config` 里的 DownFile → 启动 exe（不传参）。
/// 若配置不存在或没有 DownFile 项，退回到 `exe "<dcf>"` 命令行方式。
fn spawn_burn(exe: &str, dcf: &str) -> Result<(), String> {
    if exe.trim().is_empty() {
        return Err("未配置烧录工具路径，请到『设置』填写".into());
    }
    let exe_path = Path::new(exe);
    if !exe_path.exists() {
        return Err(format!("烧录工具不存在: {}", exe));
    }
    if !Path::new(dcf).exists() {
        return Err(format!("烧录文件不存在: {}", dcf));
    }

    // 优先方案：改 config
    if let Some(cfg) = find_burn_config(exe_path) {
        match patch_burn_config(&cfg, dcf) {
            Ok(true) => return spawn_exe(exe, &[]),
            Ok(false) => {
                // 配置里没有 DownFile 项，说明不是这个 Downloader 品牌 → 退回命令行方式
            }
            Err(e) => {
                // 改配置失败：写日志但继续尝试命令行方式，避免完全失败
                let _ = log_line(&format!("patch_burn_config FAIL: {}", e));
            }
        }
    }

    // 兜底方案：直接把 dcf 当参数传（若程序不吃参数，用户至少还能看到工具启动，然后再手动拖）
    spawn_exe(exe, &[dcf])
}

#[tauri::command]
pub fn open_target(kind: OpenKind, project_id: String) -> Result<(), String> {
    let list = load_all()?;
    let proj = list
        .iter()
        .find(|p| p.id == project_id)
        .ok_or("项目不存在")?
        .clone();

    let cfg: AppConfig = read_json(&config_path()).unwrap_or_default();

    let kind_str = match kind {
        OpenKind::Folder => "folder",
        OpenKind::Vscode => "ide", // 现在语义是"用默认 IDE 打开"
        OpenKind::Codeblocks => "codeblocks",
        OpenKind::Burn => "burn",
    };

    let result: Result<(), String> = match kind {
        OpenKind::Folder => open_folder(&proj.path),
        OpenKind::Vscode => {
            // 根据 default_ide 决定用哪个 IDE，缺省 vscode
            let ide = cfg.default_ide.trim().to_ascii_lowercase();
            let (label, exe) = if ide == "trae" {
                ("Trae", cfg.trae_path.as_str())
            } else {
                ("VSCode", cfg.vscode_path.as_str())
            };
            if exe.trim().is_empty() {
                Err(format!("未配置 {} 路径，请到『设置』填写", label))
            } else {
                spawn_exe(exe, &[proj.path.as_str()])
            }
        }
        OpenKind::Codeblocks => {
            let target = proj
                .selected_cbp
                .clone()
                .or_else(|| proj.cbp_files.first().cloned());
            match target {
                Some(cbp) => spawn_exe(&cfg.codeblocks_path, &[cbp.as_str()]),
                None => Err("该项目下未找到 .cbp 文件".into()),
            }
        }
        OpenKind::Burn => {
            let target = proj
                .selected_dcf
                .clone()
                .or_else(|| proj.dcf_files.first().cloned());
            match target {
                Some(dcf) => spawn_burn(&cfg.burn_tool_path, &dcf),
                None => Err("该项目下未找到 .dcf 文件".into()),
            }
        }
    };

    // 无论成败：写日志 + 刷新 last_accessed
    let status = match &result {
        Ok(_) => "OK".to_string(),
        Err(e) => format!("FAIL: {}", e),
    };
    let _ = log_line(&format!(
        "open_target project=\"{}\" kind={} path=\"{}\" result={}",
        proj.name, kind_str, proj.path, status
    ));
    touch_project_access(&project_id);

    result
}
