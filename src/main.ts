import { createApp } from "vue";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
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
// 这里等界面画出来后：显式恢复尺寸 + 居中 + 显示（见底）。
// 注意：visible: false 时 Tauri 可能以异常的小尺寸创建窗口，且隐藏窗口下
// requestAnimationFrame 不保证触发，因此用 setTimeout 兜底，避免窗口永远出不来。
const win = getCurrentWindow();
function reveal() {
  win.setSize(new LogicalSize(960, 680)).catch(() => {});
  win.center().catch(() => {});
  win.show().catch(() => {
    // 非 Tauri 环境（纯浏览器调试）忽略
  });
}
requestAnimationFrame(() => {
  requestAnimationFrame(reveal);
});
setTimeout(reveal, 800);
