# 工作助手 (Forge Studio)

> 一款 Windows 桌面便携绿色软件：双击 `.exe` 即用、无需安装、数据随身。
> 用于统一管理常用应用启动、STM32/嵌入式项目的快速打开、编译工程 (`.cbp`) 与烧录文件 (`.dcf`) 的一键调用。

---

## 一、项目状态

- 阶段：**框架雏形搭建中**
- 版本：`0.1.0-alpha`
- 更新日期：2026-07-30

后续每次功能落地或结构调整，都会同步更新本 README。

---

## 二、技术选型

| 层级 | 选型 | 说明 |
| --- | --- | --- |
| 桌面壳 | **Tauri v2** | 最新稳定版，权限体系升级为 permissions/capabilities |
| 前端框架 | **Vue 3 + TypeScript** | `<script setup>` 组合式 API，业务逻辑极简 |
| 构建工具 | **Vite** | 官方 Tauri 模板默认使用 |
| UI 库 | **Naive UI** | 纯亮色、轻量、TS 原生支持 |
| 后端语言 | **Rust** | 通过 `tauri::command` 暴露给前端 |
| 数据存储 | **本地 JSON 文件** | 全部放在 `.exe` 同级 `Data/` 目录 |

打包输出：**便携版单文件 `.exe`**（绿色免安装）。

---

## 三、便携存储方案

所有用户数据存放在 **可执行文件同目录** 的 `Data/` 文件夹内：

```
你的工作助手.exe
Data/
├── config.json          # 全局配置（三个工具的路径）
├── launchers.json       # 快捷应用列表
├── projects.json        # 项目列表（核心数据）
└── logs/                # 按日期切割的日志
    └── 2026-07-30.log
```

开发期（`npm run tauri dev`）时，`Data/` 目录会落在 `src-tauri/target/debug/Data/`，便于本地调试；打包后自动位于 `.exe` 旁边。

---

## 四、数据结构（JSON Schema）

### 1. `config.json`

```json
{
  "vscode_path": "C:/Users/xxx/AppData/Local/Programs/Microsoft VS Code/Code.exe",
  "codeblocks_path": "C:/Program Files/CodeBlocks/codeblocks.exe",
  "burn_tool_path": "D:/Tools/BurnTool.exe"
}
```

### 2. `launchers.json`

```json
[
  { "id": "uuid1", "name": "计算器", "path": "C:/Windows/System32/calc.exe", "starred": true }
]
```

### 3. `projects.json`

```json
[
  {
    "id": "proj_uuid1",
    "name": "STM32_Project_A",
    "path": "D:/Work/STM32_Project_A",
    "starred": true,
    "last_accessed": 1700000000000,
    "cbp_files": ["D:/Work/STM32_Project_A/main.cbp"],
    "dcf_files": ["D:/Work/STM32_Project_A/output.dcf"],
    "selected_cbp": "D:/Work/STM32_Project_A/main.cbp",
    "selected_dcf": "D:/Work/STM32_Project_A/output.dcf"
  }
]
```

---

## 五、后端 Rust Commands 清单

| 模块 | Command | 功能 |
| --- | --- | --- |
| 配置读写 | `load_config` / `save_config` | 加载 / 保存 `config.json` |
| 启动器管理 | `list_launchers` / `add_launcher` / `remove_launcher` / `toggle_launcher_star` | 增删改星标 |
| 项目管理 | `list_projects` / `add_project` / `remove_project` / `toggle_project_star` | 增删改星标 |
| 双击重命名 | `rename_project` | 物理重命名文件夹 + 自动重扫 + 更新缓存 |
| 智能扫描 | `scan_project` | 递归遍历（限 1000 文件），筛选 `.cbp` / `.dcf` |
| 执行打开 | `open_target` | 按类型调用对应 `.exe` |
| 路径批量检测 | `check_projects` | 检查所有 `path` 是否存在 |
| 复制副本 | `duplicate_project` | 异步 + 120s 超时的目录复制 |
| 日志记录 | `append_log` | 追加写入当天日志 |
| 打开数据目录 | `open_logs_dir` | 系统资源管理器打开 `Data/logs/` |
| 清空数据 | `clear_all_data` | 二次确认后清空三份 JSON |

---

## 六、前端界面布局

```
+----------------------------------------------------------+
|  🛠️ 工作助手                          ⚙️ 设置 (齿轮图标)   |
+----------------------------------------------------------+
|  快捷应用区 (横向平铺/滚动)                                |
|  [⭐计算器] [记事本] [微信]  [+ 添加应用]                  |
+----------------------------------------------------------+
|  🔍 [搜索框]   (输入实时过滤项目名)                        |
+----------------------------------------------------------+
|  项目卡片列表 (星标⭐置顶 > 最近访问降序)                  |
|  ⭐ 项目A         [📁][✍️][🔧][🔥][📋][🔄]                |
|     项目B         [📁][✍️][🔧][🔥][📋][🔄]                |
|     项目C (失效)  [所有按钮置灰]                          |
|  [+ 添加项目]                                             |
+----------------------------------------------------------+
```

按钮说明：`📁` 打开文件夹 / `✍️` VSCode / `🔧` CodeBlocks / `🔥` 烧录 / `📋` 复制副本 / `🔄` 重扫。

---

## 七、核心业务流程（关键点）

1. **添加项目**：拖拽或浏览文件夹 → 后端取目录名 → 立即智能扫描 → 默认选中数组第 1 项。
2. **智能扫描 + 红点下拉**：`cbp_files.length > 1` 或 `dcf_files.length > 1` 时显示小红点，选择后立即持久化 `selected_cbp` / `selected_dcf`，重启后仍生效。
3. **双击重命名**：过滤非法字符 → 冲突检测 → `std::fs::rename` → 清空缓存 → 自动重扫 → 更新 `name` / `path` / `last_accessed`。
4. **失效灰显**：启动 & 手动刷新时批量检测 `path`，失效项目名灰色斜体 + `[路径失效]` 标签，所有按钮禁用。
5. **复制副本**：`tokio::task::spawn_blocking` + `tokio::time::timeout(120s)`，命名冲突自动加 `_copy_N` 后缀，完成后作为新项目添加并自动扫描。
6. **搜索 + 排序**：前端 `filter` 实时匹配（不区分大小写）；排序：`starred` 置顶 → `last_accessed` 降序。
7. **打开操作**：无论成功/失败都写日志（时间戳 | 项目名 | 操作类型 | 路径 | 结果），并刷新 `last_accessed`。
8. **手动刷新**：重扫后必须清空 `selected_cbp` / `selected_dcf`（旧文件可能已删）。

---

## 八、设置面板

- VSCode 路径（浏览）
- CodeBlocks 路径（浏览）
- 烧录工具路径（浏览）
- 【打开日志目录】按钮
- 【清空所有数据】按钮（二次确认）

---

## 九、目录结构

```
forge-studio/
├── README.md
├── package.json
├── vite.config.ts
├── tsconfig.json
├── tsconfig.node.json
├── index.html
├── .gitignore
├── src/                      # 前端 Vue 3 + TS
│   ├── main.ts
│   ├── App.vue
│   ├── style.css
│   ├── env.d.ts              # Vite / .vue 类型声明
│   ├── types/
│   │   └── index.ts          # 前后端共享类型
│   ├── api/
│   │   └── index.ts          # invoke 包装
│   └── components/
│       ├── LauncherBar.vue
│       ├── ProjectList.vue
│       ├── ProjectCard.vue
│       ├── SettingsDialog.vue
│       └── AddProjectButton.vue
└── src-tauri/                # Rust 后端
    ├── Cargo.toml
    ├── tauri.conf.json
    ├── build.rs
    ├── capabilities/
    │   └── default.json
    ├── icons/                # 打包用图标（占位）
    └── src/
        ├── main.rs
        ├── lib.rs
        ├── models.rs         # 数据结构
        ├── storage.rs        # Data/ 目录 & JSON 读写
        └── commands/
            ├── mod.rs
            ├── config.rs
            ├── launcher.rs
            ├── project.rs
            ├── scan.rs
            ├── open.rs
            ├── logger.rs
            └── misc.rs
```

---

## 十、开发 & 构建

### 环境要求

- Node.js ≥ 18
- Rust ≥ 1.77（`rustup default stable`）
- Windows：需要 [Microsoft Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) 和 [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/)
- Tauri CLI：随 `npm i` 一起装到 devDependencies，无需全局安装

### 常用命令

```powershell
npm install          # 安装前端依赖（首次会拉取 tauri-cli，稍慢）
npm run dev          # 仅启动 Vite（浏览器调试用）
npm run tauri dev    # 启动 Tauri 桌面壳 + 热更新（推荐）
npm run build        # 前端产物构建
npm run tauri build  # 打包便携 exe（输出在 src-tauri/target/release/）
```

### 便携打包提示

- `tauri.conf.json` 已配置 `bundle.targets` 为 `nsis`（也可切 `msi`），最终产物会输出 `.exe`。
- 如需真正的**单文件绿色版**（无安装器），可将 `bundle.active` 设为 `false`，直接把 `target/release/forge-studio.exe` + 同级 `Data/` 拷走即可。

---

## 十一、更新日志

- `2026-07-30`：初始化项目骨架，编写 README、Tauri v2 + Vue 3 + TS + Naive UI 脚手架、Rust Commands 占位。
