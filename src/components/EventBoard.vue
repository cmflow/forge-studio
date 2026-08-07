<script setup lang="ts">
// 事件进展看板：新建事件 + 分类筛选 + 事件列表（独立模块，不依赖项目管理）
import { computed, h, onMounted, ref } from "vue";
import {
  NButton,
  NDropdown,
  NEmpty,
  NInput,
  NSpin,
  useDialog,
  useMessage,
} from "naive-ui";
import { addEvent, listEvents } from "../api";
import type { ProgressEvent } from "../types";
import { monthKey, monthLabel } from "../utils/date";
import EventCard from "./EventCard.vue";
import SyncPanel from "./SyncPanel.vue";

const props = defineProps<{ search: string; syncVersion?: number }>();
const emit = defineEmits<{ (e: "goto-settings"): void }>();

const message = useMessage();
const dialog = useDialog();
const events = ref<ProgressEvent[]>([]);
const firstLoading = ref(true);
const titleDraft = ref("");
const showArchived = ref(false);
/** 归档月份筛选，"" = 全部月份。仅在归档视图下生效 */
const activeMonth = ref("");

function toggleArchived() {
  showArchived.value = !showArchived.value;
  // 退出归档视图时清掉月份筛选，避免残留状态影响进行中列表
  if (!showArchived.value) activeMonth.value = "";
}
/** 当前选中的分类：null = 全部；"" = 未分类 */
const activeCategory = ref<string | null>(null);
/** 手风琴：当前展开的事件 id，null 表示全部折叠（默认） */
const expandedId = ref<string | null>(null);

/** 新建事件时使用的分类，选定后保持不变（含重启），直到手动更改 */
const NEW_CAT_KEY = "forge-studio:new-event-category";
const newCategory = ref<string>(localStorage.getItem(NEW_CAT_KEY) ?? "");

function setNewCategory(v: string) {
  newCategory.value = v;
  localStorage.setItem(NEW_CAT_KEY, v);
}

async function refresh() {
  try {
    events.value = await listEvents();
  } catch (e) {
    console.error("加载事件失败:", e);
    // 首次加载失败直接提示；后续静默重试（可能只是磁盘暂时不可读）
    if (firstLoading.value) message.error(String(e));
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
    await addEvent(title, undefined, newCategory.value);
    titleDraft.value = "";
    await refresh();
  } catch (e) {
    message.error(String(e));
  }
}

/** 新建分类下拉：未分类 + 已有分类 + 新建 */
const newCategoryOptions = computed(() => {
  const opts: { label: string; key: string }[] = [
    { label: "未分类", key: "__none__" },
  ];
  for (const c of categories.value) {
    opts.push({ label: c, key: `set:${c}` });
  }
  opts.push({ label: "＋ 新建分类…", key: "__new__" });
  return opts;
});

function onNewCategorySelect(key: string) {
  if (key === "__none__") {
    setNewCategory("");
    return;
  }
  if (key === "__new__") {
    const draft = ref("");
    dialog.create({
      title: "新建分类",
      content: () =>
        h(NInput, {
          value: draft.value,
          placeholder: "输入分类名，如「客户返修」",
          onUpdateValue: (v: string) => (draft.value = v),
        }),
      positiveText: "确定",
      negativeText: "取消",
      onPositiveClick: () => {
        const name = draft.value.trim();
        if (name) setNewCategory(name);
      },
    });
    return;
  }
  setNewCategory(key.slice(4));
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

/** 归档月份选项，按归档时间倒序（仅归档视图使用） */
const archivedMonths = computed(() => {
  const set = new Set<string>();
  for (const e of events.value) {
    if (e.status === "done" && e.archived_at) {
      set.add(monthKey(e.archived_at));
    }
  }
  return Array.from(set).sort().reverse();
});

/** 一行最多平铺的月份按钮数，其余收进「更多月份」下拉 */
const MAX_MONTH_TABS = 5;

/** 平铺显示的月份：最近 5 个；若当前选中的是更早月份，则把它替换进来以保持可见 */
const visibleMonths = computed(() => {
  const all = archivedMonths.value;
  const head = all.slice(0, MAX_MONTH_TABS);
  const cur = activeMonth.value;
  if (cur && !head.includes(cur)) {
    // 选中了下拉里的月份，替换最后一项让它保持在可见区
    return [...head.slice(0, MAX_MONTH_TABS - 1), cur];
  }
  return head;
});

/** 收进下拉的月份 */
const overflowMonths = computed(() =>
  archivedMonths.value.filter((m) => !visibleMonths.value.includes(m)),
);

const monthDropdownOptions = computed(() =>
  overflowMonths.value.map((m) => ({ label: monthLabel(m), key: m })),
);

const visibleEvents = computed(() => {
  const kw = props.search.trim().toLowerCase();
  let list = events.value.slice();
  if (!showArchived.value) {
    list = list.filter((e) => e.status !== "done");
  } else if (activeMonth.value) {
    // 月份筛选仅作用于归档事件
    list = list.filter((e) => {
      if (e.status !== "done" || !e.archived_at) return false;
      return monthKey(e.archived_at) === activeMonth.value;
    });
  }
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
    // 归档视图按归档时间倒序，更贴合「按月回顾」的用法
    if (showArchived.value && a.archived_at && b.archived_at) {
      return b.archived_at - a.archived_at;
    }
    return b.updated_at - a.updated_at;
  });
  return list;
});

defineExpose({ refresh });
</script>

<template>
  <div class="event-board">
    <!-- 固定区：同步 + 新建 + 分类筛选，不随列表滚动 -->
    <div class="board-fixed">
      <!-- 云同步面板：每台设备独立存档，上传不覆盖别人 -->
      <SyncPanel
        :version="props.syncVersion"
        @pulled="refresh"
        @goto-settings="emit('goto-settings')"
      />

      <div class="create-row">
        <NInput
          v-model:value="titleDraft"
          placeholder="新建事件，例如「客户 A 设备返修」，回车创建"
          @keyup.enter="submitEvent"
        />
        <!-- 新建时归入的分类，选定后保持不变 -->
        <NDropdown
          trigger="click"
          :options="newCategoryOptions"
          @select="onNewCategorySelect"
        >
          <NButton class="new-cat-btn" :title="'新建事件将归入：' + (newCategory || '未分类')">
            🏷 {{ newCategory || "未分类" }} ▾
          </NButton>
        </NDropdown>
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
          :quaternary="!showArchived"
          :type="showArchived ? 'success' : 'default'"
          @click="toggleArchived"
        >
          {{ showArchived ? "隐藏已归档" : `显示已归档（${archivedCount}）` }}
        </NButton>
      </div>

      <!-- 归档月份筛选：仅在归档视图下出现 -->
      <div v-if="showArchived && archivedMonths.length" class="month-row">
        <span class="month-label">归档月份</span>
        <button
          class="month-tab"
          :class="{ active: activeMonth === '' }"
          @click="activeMonth = ''"
        >
          全部
        </button>
        <button
          v-for="m in visibleMonths"
          :key="m"
          class="month-tab"
          :class="{ active: activeMonth === m }"
          @click="activeMonth = m"
        >
          {{ monthLabel(m) }}
        </button>

        <!-- 超出 5 个的更早月份收进下拉 -->
        <NDropdown
          v-if="overflowMonths.length"
          trigger="click"
          :options="monthDropdownOptions"
          @select="(k: string) => (activeMonth = k)"
        >
          <button class="month-tab more" title="选择更早的月份">
            更早 ({{ overflowMonths.length }}) ▾
          </button>
        </NDropdown>
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
/* 分类按钮固定宽度，切换分类时不挤动输入框与新建按钮 */
.new-cat-btn {
  flex: none;
  width: 116px;
}
.new-cat-btn :deep(.n-button__content) {
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
  display: block;
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

/* 归档月份筛选：绿色系，与蓝色分类标签区分，暗示这是归档专属维度 */
.month-row {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  background: #f2f9f5;
  border: 1px solid #dcefe4;
  border-radius: 6px;
}
.month-label {
  font-size: 11px;
  color: #18a058;
  font-weight: 600;
}
.month-tab {
  padding: 2px 9px;
  font-size: 11px;
  color: #4b5563;
  background: #fff;
  border: 1px solid #dcefe4;
  border-radius: 999px;
  cursor: pointer;
  font-variant-numeric: tabular-nums;
  transition:
    background 0.15s ease,
    color 0.15s ease,
    border-color 0.15s ease;
}
.month-tab:hover {
  border-color: #18a058;
  color: #18a058;
}
.month-tab.active {
  background: #18a058;
  border-color: #18a058;
  color: #fff;
}
/* 「更早」下拉入口：虚线边框以区别于具体月份按钮 */
.month-tab.more {
  border-style: dashed;
  color: #18a058;
}
</style>
