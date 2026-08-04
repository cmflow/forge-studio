# 工作助手 (Forge Studio)

> 一款 Windows 桌面便携绿色软件：双击 `.exe` 即用、无需安装、数据随身。
> 用于统一管理常用应用启动、STM32/嵌入式项目的快速打开、编译工程 (`.cbp`) 与烧录文件 (`.dcf`) 的一键调用。

---

## 一、项目状态

- 阶段：**核心功能全部落地，可日常使用**
- 版本：`0.1.0`
- 更新日期：2026-08-03

后续每次功能落地或结构调整，都会同步更新本 README。

### 功能清单（已实现 ✅）

**项目管理**
- ✅ 添加项目：右下角 ＋ 选择文件夹 · 拖拽文件夹到窗口批量添加
- ✅ 智能扫描：递归遍历自动识别 `.cbp` / `.dcf`（支持 `projects/` 下的多层子目录）
- ✅ 双击项目名物理重命名：非法字符校验 + 冲突检测 + `std::fs::rename` + 清缓存 + 重扫
- ✅ 一键打开：文件夹 / IDE（VSCode 或 Trae）/ CodeBlocks / 烧录工具
- ✅ 多 `.cbp` / `.dcf` 时按钮红点 + 右键下拉选择（当前项带 ✔）+ 立即持久化
- ✅ 右键 `.dcf` 按钮可"在资源管理器中定位"该文件
- ✅ 复制副本：`spawn_blocking + timeout(120s)` + 全屏遮罩 + `_copy_N` 命名 + 自动加为新项目
- ✅ 移除项目（二次确认）· 星标切换
- ✅ 路径失效：启动/刷新时批量检测，失效项灰色斜体 + `[路径失效]` 标签 + 按钮禁用
- ✅ 搜索：实时不区分大小写过滤
- ✅ 排序：星标置顶 → `last_accessed` 降序，操作后自动前移
- ✅ 操作后静默刷新，列表不闪烁

**快捷应用**
- ✅ ＋ 添加应用（`.exe/.bat/.cmd`）· 拖拽 exe 到快捷应用区
- ✅ 自动提取并缓存 exe 真实图标（支持 24bpp + AND 掩码的老式图标透明还原）
- ✅ 左键启动（自动切 CWD 到 exe 目录，避免"运行环境异常"）· 右键菜单（星标 / 移除）
- ✅ 星标置顶
- ✅ 工具目录自动扫描：指定根目录后，每个子目录取第一个 `.exe` 自动加入（同名跳过）

**设置面板**
- ✅ 开机自启开关（写入 `HKCU\...\Run`，无需管理员权限，即点即生效）
- ✅ 默认 IDE 选择：VSCode / Trae
- ✅ VSCode / Trae / CodeBlocks / 烧录工具四条路径：自动识别 + 浏览…
- ✅ 工具目录：路径选择 + 立即扫描 + 启动时自动扫描开关（保存时也会立即扫一遍）
- ✅ 打开日志目录（explorer）
- ✅ 清空所有数据（二次确认）

**日志与容错**
- ✅ 每次打开操作按日期切割写入 `logs/YYYY-MM-DD.log`：`[时间] operation project="…" kind=… result=OK/FAIL: …`
- ✅ 所有失败通过 Naive UI Toast 提示，不闪退不卡死
- ✅ 复制副本按钮防连点
- ✅ 屏蔽 WebView 自带右键菜单（输入框内保留，便于粘贴）

---

## 二、技术选型

| 层级 | 选型 | 说明 |
| --- | --- | --- |
| 桌面壳 | **Tauri v2** | 最新稳定版，权限体系升级为 permissions/capabilities |
| 前端框架 | **Vue 3 + TypeScript** | `<script setup>` 组合式 API，业务逻辑极简 |
| 构建工具 | **Vite** | 官方 Tauri 模板默认使用 |
| UI 库 | **Naive UI** | 纯亮色、轻量、TS 原生支持 |
| 后端语言 | **Rust** | 通过 `tauri::command` 暴露给前端 |
| 数据存储 | **本地 JSON 文件** | 统一放在 `%USERPROFILE%\.forge-studio\` |

打包输出：**便携版单文件 `.exe`**（绿色免安装，约 4.7 MB）。

---

## 三、数据存储方案

所有用户数据存放在 **用户主目录下的 `.forge-studio/`** 文件夹内：

```
C:\Users\<你的用户名>\.forge-studio\
├── config.json          # 全局配置（工具路径、默认 IDE、扫描设置）
├── launchers.json       # 快捷应用列表
├── projects.json        # 项目列表（核心数据）
└── logs/                # 按日期切割的日志
    └── 2026-08-03.log
```

**为什么不放 exe 同级**：开发期 exe 在 `target/debug/`，release 在 `target/release/`，两者数据会分裂；放用户目录后，`npm run tauri dev` 与打包后的 exe **共享同一份数据**，切换无感。

---

## 四、数据结构（JSON Schema）

### 1. `config.json`

```json
{
  "vscode_path": "C:/Users/xxx/AppData/Local/Programs/Microsoft VS Code/Code.exe",
  "trae_path": "C:/Users/xxx/AppData/Local/Programs/Trae CN/Trae CN.exe",
  "codeblocks_path": "C:/Program Files/CodeBlocks/codeblocks.exe",
  "burn_tool_path": "C:/dev_utils/downloader_v3.5.0/Downloader.exe",
  "default_ide": "vscode",
  "dev_utils_root": "C:/dev_utils",
  "scan_dev_utils_on_start": true
}
```

- `default_ide`：`"vscode"` | `"trae"`，决定卡片上 *IDE* 按钮调用哪个
- `dev_utils_root`：工具目录根路径，扫描时每个子目录取第一个 `.exe`
- `scan_dev_utils_on_start`：应用启动时是否自动扫描该目录

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
| 工具自动识别 | `detect_tool_path` | 按 `vscode` / `trae` / `codeblocks` / `burn` 扫常见安装路径 |
| 启动器管理 | `list_launchers` / `add_launcher` / `remove_launcher` / `toggle_launcher_star` | 增删改星标 |
| 启动器扫描 | `scan_dev_utils` | 扫工具根目录，每子目录取第一个 `.exe`，同名跳过 |
| 启动器运行 | `run_launcher` | 启动 exe（CWD 切到 exe 所在目录） |
| 启动器图标 | `get_launcher_icon` | 提取 exe 图标为 PNG DataURL（内存缓存 + 透明还原） |
| 项目管理 | `list_projects` / `add_project` / `remove_project` / `toggle_project_star` | 增删改星标 |
| 双击重命名 | `rename_project` | 物理重命名文件夹 + 自动重扫 + 更新缓存 |
| 智能扫描 | `scan_project` | 递归遍历，筛选 `.cbp` / `.dcf` |
| 选择文件 | `select_cbp` / `select_dcf` | 持久化 `selected_cbp` / `selected_dcf` |
| 执行打开 | `open_target` | 按类型调用对应 `.exe`（`vscode` 会按 `default_ide` 分派） |
| 资源管理器定位 | `reveal_in_explorer` | `explorer /select,<path>` 定位并高亮文件 |
| 路径批量检测 | `check_projects` | 检查所有 `path` 是否存在 |
| 复制副本 | `duplicate_project` | 异步 + 120s 超时的目录复制 |
| 开机自启 | `get_autostart` / `set_autostart` | 读写 `HKCU\...\Run` 下的 `ForgeStudio` 键 |
| 日志记录 | `append_log` | 追加写入当天日志 |
| 打开数据目录 | `open_logs_dir` | 系统资源管理器打开 `logs/` |
| 清空数据 | `clear_all_data` | 二次确认后清空三份 JSON |

---

## 六、前端界面布局

```
+----------------------------------------------------------+
|  🛠️ 工作助手                          ⚙️ 设置 (齿轮图标)   |
+----------------------------------------------------------+
|  快捷应用区 (横向平铺 · 固定不随列表滚动)                  |
|  [⭐计算器] [记事本] [BLEDebug]  [+ 添加应用]              |
+----------------------------------------------------------+
|  🔍 [搜索框]   (输入实时过滤项目名 · 固定不滚动)           |
+----------------------------------------------------------+
|  项目卡片列表 (可滚动 · 星标⭐置顶 > 最近访问降序)         |
|  ⭐ 项目A         [📁][IDE][.cbp][.dcf][copy][✕]          |
|     项目B         [📁][IDE][.cbp][.dcf][copy][✕]          |
|     项目C (失效)  [所有按钮置灰]                          |
|  [+ 添加项目]                                             |
+----------------------------------------------------------+
```

按钮说明：

| 按钮 | 左键 | 右键 |
| --- | --- | --- |
| `📁` | 打开文件夹 | — |
| *IDE* | 用默认 IDE 打开（VSCode / Trae，可在设置切换） | — |
| *.cbp* | 用 CodeBlocks 打开当前 `.cbp` | 多个时弹菜单切换（当前项带 ✔） |
| *.dcf* | 用烧录工具打开当前 `.dcf` | 在资源管理器中定位 / 多个时切换 |
| *copy* | 复制副本 | — |
| `✕` | 移除项目（二次确认，不删硬盘文件） | — |

- 文字型按钮采用 **斜体 + 粗体**，比 emoji 更易分辨
- 多 `.cbp` / `.dcf` 时按钮右上角有红点提示
- 悬浮 tooltip 会显示当前生效的文件名，不用右键也能确认

---

## 七、核心业务流程（关键点）

1. **添加项目**：拖拽或浏览文件夹 → 后端取目录名 → 立即智能扫描 → 默认选中数组第 1 项。
2. **智能扫描 + 红点下拉**：`cbp_files.length > 1` 或 `dcf_files.length > 1` 时显示小红点，右键弹菜单选择后立即持久化 `selected_cbp` / `selected_dcf`，重启后仍生效。
3. **双击重命名**：过滤非法字符 → 冲突检测 → `std::fs::rename` → 清空缓存 → 自动重扫 → 更新 `name` / `path` / `last_accessed`。
4. **失效灰显**：启动 & 刷新时批量检测 `path`，失效项目名灰色斜体 + `[路径失效]` 标签，所有按钮禁用。
5. **复制副本**：`tokio::task::spawn_blocking` + `tokio::time::timeout(120s)`，命名冲突自动加 `_copy_N` 后缀，完成后作为新项目添加并自动扫描。
6. **搜索 + 排序**：前端 `filter` 实时匹配（不区分大小写）；排序：`starred` 置顶 → `last_accessed` 降序。
7. **打开操作**：无论成功/失败都写日志（时间戳 | 项目名 | 操作类型 | 路径 | 结果），并刷新 `last_accessed`。
8. **静默刷新**：操作完成后只更新数据不重建 DOM（靠 `:key="p.id"` 做差量），避免列表闪烁。
9. **烧录工具**：打开前会把 `selected_dcf` 写进 `Downloader.config` 的 `DownFile` 字段，然后以 exe 所在目录为 CWD 启动，模拟"双击时的最近记录"。
10. **工具目录扫描**：遍历 `dev_utils_root` 的每个子目录，取第一个 `.exe`，以 exe 文件名（忽略大小写）判重，已存在则跳过。
11. **开机自启**：写 `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` 的 `ForgeStudio` 值（无需管理员权限），路径自动加引号兼容空格。

---

## 八、设置面板

从上到下：

1. **开机自启** 开关（即点即生效，不需要按保存）
2. **默认 IDE**：VSCode / Trae 单选
3. **VSCode 路径**（自动识别 / 浏览…）
4. **Trae 路径**（自动识别 / 浏览…，支持 Trae CN 与国际版）
5. **CodeBlocks 路径**（自动识别 / 浏览…）
6. **工具目录**（浏览… / 立即扫描 / 启动时自动扫描开关）
7. **烧录工具路径**（自动识别 / 浏览…）
8. 【打开日志目录】按钮
9. 【清空所有数据】按钮（二次确认）

**自动识别**会扫这些常见位置：

| 工具 | 扫描路径 |
| --- | --- |
| VSCode | `%LOCALAPPDATA%\Programs\Microsoft VS Code\` · `C:/D: Program Files\` · Insiders |
| Trae | `%LOCALAPPDATA%\Programs\Trae CN\` · `Trae\` · `C:/D: Program Files\` · D 盘根目录 |
| CodeBlocks | `C:/D: Program Files (x86)\CodeBlocks\` · 盘根目录 |
| 烧录工具 | `C:/D:\dev_utils\downloader_v3.5.0\` · `C:/D:\downloader_v3.5.0\` |

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
    ├── icons/                # 打包用图标（透明底橘子）
    └── src/
        ├── main.rs
        ├── lib.rs
        ├── models.rs         # 数据结构
        ├── storage.rs        # %USERPROFILE%/.forge-studio 目录 & JSON 读写
        └── commands/
            ├── mod.rs
            ├── config.rs     # 配置读写 + 工具路径自动识别
            ├── launcher.rs   # 快捷应用 + 工具目录扫描
            ├── icon.rs       # exe 图标提取（Win32 API + PNG 编码 + 缓存）
            ├── project.rs
            ├── scan.rs
            ├── open.rs       # 打开分派（含烧录工具 config 改写）
            ├── logger.rs
            └── misc.rs       # 日志目录 / 清空数据 / 定位文件 / 开机自启
```

---

## 十、开发 & 构建

### 环境要求

- Node.js ≥ 18
- Rust ≥ 1.77（`rustup default stable`，实测 1.97 通过）
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

- `tauri.conf.json` 的 `bundle.targets` 已设为 `[]`，`npm run tauri build` **只产出单个 exe**，不生成安装器。
- 产物位置：`src-tauri/target/release/forge-studio.exe`（约 4.7 MB），拷到任何目录双击即用。
- 数据在 `%USERPROFILE%\.forge-studio\`，与开发期共享，换 exe 不丢配置。
- 如需安装包版本，把 `targets` 改回 `["nsis"]` 或 `["msi"]` 重新打包即可。
- **不要**直接跑 `cargo build --release`——它不走 Tauri 的前端资源嵌入流程，出来的 exe 启动会报 `ERR_CONNECTION_REFUSED`。

### 换图标流程

```powershell
npx tauri icon logo.png                                  # 生成全平台图标
Remove-Item src-tauri\target\debug\forge-studio.exe       # 强制重链（否则 cargo 增量编译不会重嵌）
npm run tauri dev
```

---

## 十一、更新日志

- `2026-08-03`：UX 打磨 + 新功能 + 出包流程定型。
  - **Bug 修复**：列表操作后闪烁（改静默刷新）· 滚动时快捷应用被顶走（改双层布局）· 多 cbp/dcf 无法辨认当前项（加 ✔ 与 tooltip）· 空白区弹出 WebView 原生右键菜单 · 快捷应用按钮文字 descender 被裁。
  - **新功能**：默认 IDE（VSCode / Trae）· Trae 路径自动识别 · 工具目录自动扫描（启动时 / 保存时 / 手动）· 开机自启开关 · `.dcf` 右键在资源管理器中定位。
  - **UI 调整**：项目卡片按钮从 emoji 改为斜体粗体文字（*IDE* / *.cbp* / *.dcf* / *copy*），移除按钮改 `✕`，去掉重扫按钮。
  - **工程**：`bundle.targets` 改 `[]` 出单 exe · 数据目录迁到 `%USERPROFILE%\.forge-studio\`（dev / release 共享）· 图标重制为透明底 · 修复 `vue-tsc` 在 `keyof AppConfig` 含 boolean 字段时的 `never` 赋值报错。
- `2026-07-30 (4)`：README 全面刷新——同步"已实现功能清单"、把项目状态升级为 `0.1.0` 可用版、修正环境要求（补充实测 Rust 1.97 通过）。
- `2026-07-30 (3)`：第二步功能落地。
  - **物理重命名**：双击项目名 → 校验非法字符 → 冲突检测 → `std::fs::rename` → 清缓存 → 自动重扫。
  - **复制副本**：`tokio::spawn_blocking + timeout(120s)` + 前端全屏遮罩 + 按钮防连点 + `_copy_N` 自动命名 + 自动加为新项目。
  - **红点下拉**：多 `.cbp` / `.dcf` 时按钮上出现红点，点击弹下拉列表，选择后立即持久化 `selected_cbp` / `selected_dcf`。
  - **拖拽添加**：拖文件夹到窗口 → 悬浮蓝色虚线高亮 + 提示 → 松手即添加为项目。
  - **快捷应用完整 UI**：`+ 添加应用` 弹选择框、点击启动、右键菜单（星标 / 移除）、星标置顶。
  - `run_launcher` 后端真实启动 + 写日志。
- `2026-07-30 (2)`：第一步功能落地。设置面板三个 exe 路径的浏览、`+` 添加项目、四大打开操作、重扫、移除项目、星标切换、路径失效灰显、`last_accessed` 自动刷新，全部真实生效并写入日志。
- `2026-07-30 (1)`：初始化项目骨架，编写 README、Tauri v2 + Vue 3 + TS + Naive UI 脚手架、Rust Commands 占位。
