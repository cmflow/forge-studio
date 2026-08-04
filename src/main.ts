import { createApp } from "vue";
import App from "./App.vue";
import "./style.css";

// 屏蔽 WebView 自带的右键菜单，只在输入框内保留（粘贴等操作需要）
window.addEventListener("contextmenu", (e) => {
  const el = e.target as HTMLElement | null;
  if (el?.closest("input, textarea, [contenteditable='true']")) return;
  e.preventDefault();
});

createApp(App).mount("#app");
