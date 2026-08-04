<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import {
  NButton,
  NConfigProvider,
  NDialogProvider,
  NInput,
  NLayout,
  NLayoutHeader,
  NMessageProvider,
  zhCN,
  dateZhCN,
} from "naive-ui";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import LauncherBar from "./components/LauncherBar.vue";
import ProjectList from "./components/ProjectList.vue";
import SettingsDialog from "./components/SettingsDialog.vue";
import AddProjectButton from "./components/AddProjectButton.vue";

const search = ref("");
const showSettings = ref(false);
const launcherBarRef = ref<InstanceType<typeof LauncherBar> | null>(null);
const projectListRef = ref<InstanceType<typeof ProjectList> | null>(null);
const launcherAreaEl = ref<HTMLElement | null>(null);
const projectAreaEl = ref<HTMLElement | null>(null);
/** 当前拖拽悬浮的区域：'launcher' | 'project' | null */
const dragHoverArea = ref<null | "launcher" | "project">(null);

function refreshProjects() {
  projectListRef.value?.refresh();
}

/**
 * 根据物理坐标 y 判断落点属于哪个区域。
 * Tauri onDragDropEvent 的 position 是物理像素，需要除以 devicePixelRatio 才能与 CSS
 * 坐标（getBoundingClientRect）比较。
 */
function resolveArea(physicalY: number): "launcher" | "project" | null {
  const y = physicalY / window.devicePixelRatio;
  const l = launcherAreaEl.value?.getBoundingClientRect();
  const p = projectAreaEl.value?.getBoundingClientRect();
  if (l && y >= l.top && y <= l.bottom) return "launcher";
  if (p && y >= p.top && y <= p.bottom) return "project";
  // 落在两者之间的空白：默认按项目区处理（大部分场景就是往项目区拖）
  if (p && y > p.top) return "project";
  return null;
}

let unlisten: (() => void) | null = null;

onMounted(async () => {
  try {
    const webview = getCurrentWebview();
    unlisten = await webview.onDragDropEvent(async (event) => {
      const t = event.payload.type;
      const anyPayload = event.payload as any;
      if (t === "enter" || t === "over") {
        const pos = anyPayload.position;
        dragHoverArea.value = pos ? resolveArea(pos.y) : null;
      } else if (t === "leave") {
        dragHoverArea.value = null;
      } else if (t === "drop") {
        const paths: string[] = anyPayload.paths ?? [];
        const pos = anyPayload.position;
        const area = pos ? resolveArea(pos.y) : dragHoverArea.value;
        dragHoverArea.value = null;
        if (!paths.length) return;
        if (area === "launcher") {
          await launcherBarRef.value?.handleDrop(paths);
        } else {
          await projectListRef.value?.handleDrop(paths);
        }
      }
    });
  } catch (e) {
    console.warn("drag-drop listener failed", e);
  }
});

onUnmounted(() => {
  unlisten?.();
});

const theme = null;
</script>

<template>
  <NConfigProvider :theme="theme" :locale="zhCN" :date-locale="dateZhCN">
    <NMessageProvider>
      <NDialogProvider>
        <NLayout class="app-root">
          <NLayoutHeader bordered class="app-header">
            <div class="title">🛠️ 工作助手</div>
            <NButton quaternary circle @click="showSettings = true" title="设置">
              ⚙️
            </NButton>
          </NLayoutHeader>

          <div class="app-content">
            <!-- 固定区：快捷应用 + 搜索框，不随项目列表滚动 -->
            <div class="sticky-top">
              <div ref="launcherAreaEl">
                <LauncherBar
                  ref="launcherBarRef"
                  :drag-hover="dragHoverArea === 'launcher'"
                />
              </div>

              <div class="search-row">
                <NInput
                  v-model:value="search"
                  placeholder="🔍 搜索项目名（实时过滤，拖入文件夹到下方 = 项目 / 拖入 exe 到上方 = 快捷应用）"
                  clearable
                />
              </div>
            </div>

            <!-- 滚动区：仅项目列表 -->
            <div ref="projectAreaEl" class="scroll-area">
              <ProjectList
                ref="projectListRef"
                :search="search"
                :drag-hover="dragHoverArea === 'project'"
              />
            </div>

            <AddProjectButton @added="refreshProjects" />
          </div>
        </NLayout>

        <SettingsDialog v-model:visible="showSettings" />
      </NDialogProvider>
    </NMessageProvider>
  </NConfigProvider>
</template>

<style scoped>
.app-root {
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.app-root :deep(.n-layout-scroll-container) {
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.app-header {
  height: 48px;
  flex: none;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  background: #ffffff;
}
.title {
  font-size: 16px;
  font-weight: 600;
}
.app-content {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
.sticky-top {
  flex: none;
  padding: 12px 12px 0 12px;
  background: #f5f6f8;
}
.scroll-area {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 0 12px 12px 12px;
}
.search-row {
  margin: 12px 0;
}
</style>
