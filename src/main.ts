import { createApp } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App.vue";
import "./style.css";

// 屏蔽 WebView 自带的右键菜单，只在输入框内保留（粘贴等操作需要）
window.addEventListener("contextmenu", (e) => {
  const el = e.target as HTMLElement | null;
  if (el?.closest("input, textarea, [contenteditable='true']")) return;
  e.preventDefault();
});

const app = createApp(App);
app.mount("#app");

// 窗口在 tauri.conf.json 里配了 visible: false。
// WebView2 内核初始化约需数百毫秒，期间无法渲染任何 HTML，若窗口可见就是一片空白。
// 这里等界面真正画出来后再显示窗口，用户看到的第一帧就是完整界面（同 VSCode / Figma 做法）。
requestAnimationFrame(() => {
  requestAnimationFrame(() => {
    getCurrentWindow().show().catch(() => {
      // 非 Tauri 环境（纯浏览器调试）忽略
    });
  });
});
