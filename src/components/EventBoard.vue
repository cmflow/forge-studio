<script setup lang="ts">
// 事件进展看板：新建事件 + 分类筛选 + 事件列表（独立模块，不依赖项目管理）
import { computed, onMounted, ref } from "vue";
import { NButton, NEmpty, NInput, NSpin, useMessage } from "naive-ui";
import { addEvent, listEvents } from "../api";
import type { ProgressEvent } from "../types";
import EventCard from "./EventCard.vue";

const props = defineProps<{ search: string }>();

const message = useMessage();
const events = ref<ProgressEvent[]>([]);
const firstLoading = ref(true);
const titleDraft = ref("");
const showArchived = ref(false);
/** 当前选中的分类：null = 全部；"" = 未分类 */
const activeCategory = ref<string | null>(null);
/** 手风琴：当前展开的事件 id，null 表示全部折叠（默认） */
const expandedId = ref<string | null>(null);

async function refresh() {
  try {
    events.value = await listEvents();
  } catch (e) {
    events.value = [];
  } finally {
    firstLoading.value = false;
  }
}

onMounted(refresh);

/** 点同一个则收起，点别的则切换过去（同时只展开一个） */
function onToggle(id: string) {
  expandedId.value = expandedId.value === id ? null : id;
}

async function submitEvent() {
  const title = titleDraft.value.trim();
  if (!title) return;
  try {
    await addEvent(title);
    titleDraft.value = "";
    await refresh();
  } catch (e) {
    message.error(String(e));
  }
}

/** 所有已使用的分类名（不含未分类），供卡片下拉复用 */
const categories = computed(() => {
  const set = new Set<string>();
  for (const e of events.value) {
    if (e.category) set.add(e.category);
  }
  return Array.from(set).sort();
});

/** 分类标签条：全部 / 未分类 / 各分类，带计数 */
const categoryTabs = computed(() => {
  const pool = showArchived.value
    ? events.value
    : events.value.filter((e) => e.status !== "done");
  const tabs: { key: string | null; label: string; count: number }[] = [
    { key: null, label: "全部", count: pool.length },
    {
      key: "",
      label: "未分类",
      count: pool.filter((e) => !e.category).length,
    },
  ];
  for (const c of categories.value) {
    tabs.push({
      key: c,
      label: c,
      count: pool.filter((e) => e.category === c).length,
    });
  }
  return tabs;
});

const archivedCount = computed(
  () => events.value.filter((e) => e.status === "done").length,
);

const visibleEvents = computed(() => {
  const kw = props.search.trim().toLowerCase();
  let list = events.value.slice();
  if (!showArchived.value) list = list.filter((e) => e.status !== "done");
  if (activeCategory.value !== null) {
    list = list.filter((e) => (e.category || "") === activeCategory.value);
  }
  if (kw) {
    list = list.filter(
      (e) =>
        e.title.toLowerCase().includes(kw) ||
        e.steps.some((s) => s.text.toLowerCase().includes(kw)),
    );
  }
  list.sort((a, b) => {
    if (a.starred !== b.starred) return a.starred ? -1 : 1;
    return b.updated_at - a.updated_at;
  });
  return list;
});

defineExpose({ refresh });
</script>

<template>
  <div class="event-board">
    <!-- 固定区：新建 + 分类筛选，不随列表滚动 -->
    <div class="board-fixed">
      <div class="create-row">
        <NInput
          v-model:value="titleDraft"
          placeholder="新建事件，例如「客户 A 设备返修」，回车创建"
          @keyup.enter="submitEvent"
        />
        <NButton type="primary" :disabled="!titleDraft.trim()" @click="submitEvent">
          新建事件
        </NButton>
      </div>

      <!-- 分类标签条：点击只看该分类 -->
      <div class="cat-row">
        <button
          v-for="t in categoryTabs"
          :key="String(t.key)"
          class="cat-tab"
          :class="{ active: activeCategory === t.key }"
          @click="activeCategory = t.key"
        >
          {{ t.label }}
          <span class="cat-count">{{ t.count }}</span>
        </button>

        <div class="cat-spacer" />

        <NButton
          v-if="archivedCount"
          size="tiny"
          quaternary
          @click="showArchived = !showArchived"
        >
          {{ showArchived ? "隐藏已归档" : `显示已归档（${archivedCount}）` }}
        </NButton>
      </div>
    </div>

    <!-- 滚动区：仅事件列表 -->
    <div class="board-scroll">
      <NSpin v-if="firstLoading" size="small" />
      <NEmpty
        v-else-if="!visibleEvents.length"
        description="该分类下暂无事件，在上方输入标题创建，然后逐条记录进展"
      />
      <template v-else>
        <EventCard
          v-for="e in visibleEvents"
          :key="e.id"
          :event="e"
          :expanded="expandedId === e.id"
          :categories="categories"
          @refresh="refresh"
          @toggle="onToggle"
        />
      </template>
    </div>
  </div>
</template>

<style scoped>
.event-board {
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
}
/* 固定区：不参与滚动 */
.board-fixed {
  flex: none;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-bottom: 8px;
  background: #f5f6f8;
}
/* 滚动区：只有事件列表滚动 */
.board-scroll {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  scrollbar-gutter: stable;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-bottom: 12px;
}
.board-scroll::-webkit-scrollbar {
  width: 10px;
}
.board-scroll::-webkit-scrollbar-thumb {
  background: #d4d8dd;
  border-radius: 5px;
}
.board-scroll::-webkit-scrollbar-thumb:hover {
  background: #bfc5cc;
}
.board-scroll::-webkit-scrollbar-track {
  background: transparent;
}
.create-row {
  display: flex;
  gap: 8px;
}
.cat-row {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}
.cat-spacer {
  flex: 1;
}
.cat-tab {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 3px 10px;
  font-size: 12px;
  color: #4b5563;
  background: #fff;
  border: 1px solid #e5e7eb;
  border-radius: 999px;
  cursor: pointer;
  transition:
    background 0.15s ease,
    color 0.15s ease,
    border-color 0.15s ease;
}
.cat-tab:hover {
  border-color: #2f80ed;
  color: #2f80ed;
}
.cat-tab.active {
  background: #2f80ed;
  border-color: #2f80ed;
  color: #fff;
}
.cat-count {
  font-size: 11px;
  opacity: 0.7;
}
</style>
