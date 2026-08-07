<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from "vue";
import {
  NButton,
  NButtonGroup,
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
import EventBoard from "./components/EventBoard.vue";

/** 当前模块：workspace = 项目工作台，events = 事件进展。两者互相独立 */
const activeModule = ref<"workspace" | "events">("workspace");
const search = ref("");
const eventSearch = ref("");
const showSettings = ref(false);
/** 打开设置弹窗时默认停留的页签：齿轮→项目工作台；同步条"去设置"→事件进展 */
const settingsTab = ref("workspace");
function openSettings(tab: string) {
  settingsTab.value = tab;
  showSettings.value = true;
}
/** 设置弹窗每次关闭后自增，通知同步面板重新拉取配置（可能在设置里启用了同步） */
const syncVersion = ref(0);
watch(showSettings, (v) => {
  if (!v) syncVersion.value++;
});
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
      // 事件进展模块不接收拖拽
      if (activeModule.value !== "workspace") return;
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
            <NButtonGroup size="small">
              <NButton
                class="module-tab"
                :type="activeModule === 'workspace' ? 'primary' : 'default'"
                @click="activeModule = 'workspace'"
              >
                项目工作台
              </NButton>
              <NButton
                class="module-tab"
                :type="activeModule === 'events' ? 'primary' : 'default'"
                @click="activeModule = 'events'"
              >
                事件进展
              </NButton>
            </NButtonGroup>
            <NButton
              class="settings-btn"
              quaternary
              circle
              @click="openSettings('workspace')"
              title="设置"
            >
              ⚙️
            </NButton>
          </NLayoutHeader>

          <!-- 模块一：项目工作台 -->
          <div v-show="activeModule === 'workspace'" class="app-content">
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

          <!-- 模块二：事件进展（与工作台互相独立） -->
          <div v-show="activeModule === 'events'" class="app-content">
            <div class="sticky-top no-gutter">
              <div class="search-row">
                <NInput
                  v-model:value="eventSearch"
                  placeholder="🔍 搜索事件标题或进展内容"
                  clearable
                />
              </div>
            </div>
            <!-- 事件看板自己管内部滚动（新建行与分类条固定） -->
            <div class="event-area">
              <EventBoard
                 :search="eventSearch"
                 :sync-version="syncVersion"
                 @goto-settings="openSettings('events')"
               />
            </div>
          </div>
        </NLayout>

        <SettingsDialog
          v-model:visible="showSettings"
          :initial-tab="settingsTab"
          :sync-version="syncVersion"
        />
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
  justify-content: center;
  padding: 0 16px;
  background: #ffffff;
  position: relative;
}
/* 两个模块页签等宽，避免一长一短 */
.module-tab {
  width: 104px;
}
/* 仅定位顶栏的设置按钮。不可用 :deep(.n-button--quaternary)，
   那会命中事件卡片内所有 quaternary 按钮并把它们绝对定位，遮挡点击 */
.settings-btn {
  position: absolute;
  right: 12px;
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
  /* 右侧多留出滚动条宽度，与下方滚动区内容左右对齐 */
  padding: 12px 22px 0 12px;
  background: #f5f6f8;
}
/* 事件模块的搜索区：滚动条由 EventBoard 内部预留，这里无需额外留白 */
.sticky-top.no-gutter {
  padding-right: 12px;
}
.scroll-area {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  /* 始终预留滚动条宽度，内容变长出现滚动条时布局不再左右跳动 */
  scrollbar-gutter: stable;
  padding: 0 12px 12px 12px;
}
.scroll-area::-webkit-scrollbar {
  width: 10px;
}
.scroll-area::-webkit-scrollbar-thumb {
  background: #d4d8dd;
  border-radius: 5px;
}
.scroll-area::-webkit-scrollbar-thumb:hover {
  background: #bfc5cc;
}
.scroll-area::-webkit-scrollbar-track {
  background: transparent;
}
/* 事件模块：不在外层滚动，交给 EventBoard 内部的滚动区 */
.event-area {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  padding: 0 12px;
}
.search-row {
  margin: 12px 0;
}
</style>
