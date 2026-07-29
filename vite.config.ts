import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [vue()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // Tauri 只关心自己的目录，忽略 src-tauri 变更避免热更新循环
      ignored: ["**/src-tauri/**"],
    },
  },
}));
