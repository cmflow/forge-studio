<script setup lang="ts">
// 项目卡片列表
import { computed, onMounted, ref } from "vue";
import { NDropdown, NEmpty, NSpin, useMessage } from "naive-ui";
import {
  addProject,
  checkProjects,
  duplicateProject,
  rescanAllProjects,
} from "../api";
import type { Project } from "../types";
import ProjectCard from "./ProjectCard.vue";

const props = defineProps<{ search: string; dragHover?: boolean }>();

const message = useMessage();
const projects = ref<Project[]>([]);
const invalidIds = ref<Set<string>>(new Set());
/** 仅首次加载时展示骨架/加载态，后续刷新静默进行，避免列表整体闪烁 */
const firstLoading = ref(true);

/** 复制副本遮罩状态 */
const duplicating = ref(false);
const duplicatingHint = ref("");

async function refresh() {
  try {
    const [list, statuses] = await Promise.all([
      // 真正重扫磁盘上的 cbp/dcf（而不是重读缓存），才能反映增删变化
      rescanAllProjects(),
      checkProjects(),
    ]);
    projects.value = list;
    invalidIds.value = new Set(
      statuses.filter((s) => !s.exists).map((s) => s.id),
    );
  } catch (e) {
    projects.value = [];
    invalidIds.value = new Set();
  } finally {
    firstLoading.value = false;
  }
}

onMounted(refresh);

/** 由 App.vue 派发调用：批量把拖入的文件夹加为项目 */
async function handleDrop(paths: string[]) {
  if (!paths?.length) return;
  let ok = 0;
  let fail = 0;
  for (const p of paths) {
    try {
      await addProject(p);
      ok++;
    } catch (e) {
      fail++;
      message.error(`添加失败：${String(e)}`);
    }
  }
  if (ok > 0) {
    message.success(`已添加 ${ok} 个项目${fail ? `（失败 ${fail}）` : ""}`);
    await refresh();
  }
}

const visibleProjects = computed(() => {
  const kw = props.search.trim().toLowerCase();
  const filtered = kw
    ? projects.value.filter((p) => p.name.toLowerCase().includes(kw))
    : projects.value.slice();

  filtered.sort((a, b) => {
    if (a.starred !== b.starred) return a.starred ? -1 : 1;
    return b.last_accessed - a.last_accessed;
  });
  return filtered;
});

// ---------- 项目区域右键菜单（目前仅"刷新项目"） ----------
const ctxMenuShow = ref(false);
const ctxMenuX = ref(0);
const ctxMenuY = ref(0);

function onCtxMenu(e: MouseEvent) {
  ctxMenuX.value = e.clientX;
  ctxMenuY.value = e.clientY;
  ctxMenuShow.value = true;
}

async function onCtxSelect(key: string) {
  ctxMenuShow.value = false;
  if (key !== "refresh") return;
  await refresh();
  message.success(`已刷新 ${projects.value.length} 个项目`);
}

async function onDuplicate(id: string) {
  if (duplicating.value) return;
  duplicating.value = true;
  const target = projects.value.find((p) => p.id === id);
  duplicatingHint.value = target
    ? `正在复制『${target.name}』，请稍候…`
    : "正在复制文件，请稍候…";
  try {
    const proj = await duplicateProject(id);
    message.success(`复制成功，已添加项目：${proj.name}`);
    await refresh();
  } catch (e) {
    message.error(String(e));
  } finally {
    duplicating.value = false;
    duplicatingHint.value = "";
  }
}

defineExpose({ refresh, handleDrop });
</script>

<template>
  <!-- 右键菜单：挂在列表容器上，空白处或卡片上右键均可弹出（.cbp/.dcf 按钮有自己的右键菜单，不受影响） -->
  <div
    class="project-list"
    :class="{ 'drag-hover': dragHover }"
    @contextmenu.prevent="onCtxMenu"
  >
    <NSpin v-if="firstLoading" size="small" />
    <NEmpty
      v-else-if="!visibleProjects.length"
      description="暂无项目，点击右下角『＋』或拖拽文件夹添加"
    />
    <template v-else>
      <ProjectCard
        v-for="p in visibleProjects"
        :key="p.id"
        :project="p"
        :invalid="invalidIds.has(p.id)"
        @refresh="refresh"
        @duplicate="onDuplicate"
      />
    </template>

    <!-- 拖拽悬浮时的浮层提示 -->
    <div v-if="dragHover" class="drop-hint">
      <div class="drop-hint-inner">📥 松开鼠标即可添加为项目</div>
    </div>

    <!-- 全屏遮罩：复制副本进行中 -->
    <div v-if="duplicating" class="mask">
      <div class="mask-inner">
        <NSpin size="large" />
        <div class="mask-text">{{ duplicatingHint }}</div>
        <div class="mask-hint">最多等待 120 秒</div>
      </div>
    </div>

    <!-- 项目区域右键菜单 -->
    <NDropdown
      placement="bottom-start"
      trigger="manual"
      :x="ctxMenuX"
      :y="ctxMenuY"
      :options="[
        { label: '🔄 刷新项目', key: 'refresh' },
      ]"
      :show="ctxMenuShow"
      @select="onCtxSelect"
      @clickoutside="ctxMenuShow = false"
    />
  </div>
</template>

<style scoped>
.project-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-bottom: 72px;
  position: relative;
}
.project-list.drag-hover {
  outline: 2px dashed #2f80ed;
  outline-offset: -4px;
  border-radius: 8px;
}
.drop-hint {
  position: fixed;
  inset: 48px 12px 12px 12px;
  pointer-events: none;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
}
.drop-hint-inner {
  padding: 12px 24px;
  background: rgba(47, 128, 237, 0.9);
  color: #fff;
  border-radius: 8px;
  font-size: 14px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.16);
}
.mask {
  position: fixed;
  inset: 0;
  background: rgba(255, 255, 255, 0.72);
  backdrop-filter: blur(2px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 3000;
  pointer-events: all;
}
.mask-inner {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 24px 32px;
  background: #ffffff;
  border-radius: 8px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
}
.mask-text {
  font-size: 14px;
  color: #374151;
}
.mask-hint {
  font-size: 12px;
  color: #9ca3af;
}
</style>
