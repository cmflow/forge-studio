<script setup lang="ts">
// 单个事件卡片：标题 + 进展时间线，点击节点圆点可循环切换状态
// 展开态由父级统一控制（手风琴：同时只允许一个展开）
import { computed, h, ref } from "vue";
import { NButton, NDropdown, NInput, NTag, useDialog, useMessage } from "naive-ui";
import type { ProgressEvent, StepState } from "../types";
import { fmtDateTime, shortDate } from "../utils/date";
import {
  addStep,
  cycleStepState,
  removeEvent,
  removeStep,
  setEventCategory,
  toggleEventStar,
  toggleEventStatus,
} from "../api";

const props = defineProps<{
  event: ProgressEvent;
  expanded: boolean;
  /** 已存在的分类列表，供快速选择 */
  categories: string[];
}>();
const emit = defineEmits<{
  (e: "refresh"): void;
  (e: "toggle", id: string): void;
}>();

const message = useMessage();
const dialog = useDialog();

const stepDraft = ref("");
const busy = ref(false);

const STEP_META: Record<StepState, { icon: string; label: string }> = {
  pending: { icon: "○", label: "待办" },
  doing: { icon: "◉", label: "进行中" },
  done: { icon: "●", label: "已完成" },
};

const isDone = computed(() => props.event.status === "done");

/** 进展节点总数。不再显示 done/total，因为 add_step 会自动收敛上一节点，
    done 恒等于 total-1，该比值无信息量 */
const stepCount = computed(() => props.event.steps.length);

/** 当前正在进行的节点文案，用于折叠时概览 */
const currentStep = computed(
  () =>
    props.event.steps.find((s) => s.state === "doing") ??
    props.event.steps[props.event.steps.length - 1] ??
    null,
);

/** 事件创建日期 */
const createdDate = computed(() => shortDate(props.event.created_at));

/** 归档日期，与创建日期同一格式规则 */
const archivedDate = computed(() =>
  props.event.archived_at ? shortDate(props.event.archived_at) : "",
);

/** 统一包装：串行化请求 + 错误提示 + 刷新 */
async function guard(fn: () => Promise<unknown>) {
  if (busy.value) return;
  busy.value = true;
  try {
    await fn();
    emit("refresh");
  } catch (e) {
    message.error(String(e));
  } finally {
    busy.value = false;
  }
}

async function submitStep() {
  const text = stepDraft.value.trim();
  if (!text) return;
  await guard(async () => {
    await addStep(props.event.id, text);
    stepDraft.value = "";
  });
}

function onCycle(stepId: string) {
  guard(() => cycleStepState(props.event.id, stepId));
}

function onRemoveStep(stepId: string) {
  guard(() => removeStep(props.event.id, stepId));
}

function onToggleStar() {
  guard(() => toggleEventStar(props.event.id));
}

/** 分类下拉：已有分类 + 未分类 + 新建 */
const categoryOptions = computed(() => {
  const opts = [
    { label: "未分类", key: "__none__" },
    ...props.categories
      .filter((c) => c && c !== props.event.category)
      .map((c) => ({ label: c, key: `set:${c}` })),
  ];
  opts.push({ label: "＋ 新建分类…", key: "__new__" });
  return opts;
});

function onCategorySelect(key: string) {
  if (key === "__none__") {
    guard(() => setEventCategory(props.event.id, ""));
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
        if (!name) return;
        guard(() => setEventCategory(props.event.id, name));
      },
    });
    return;
  }
  guard(() => setEventCategory(props.event.id, key.slice(4)));
}

function onToggleStatus() {
  guard(() => toggleEventStatus(props.event.id));
}

function onRemoveEvent() {
  dialog.warning({
    title: "删除事件",
    content: `确定删除『${props.event.title}』及其全部进展记录？`,
    positiveText: "删除",
    negativeText: "取消",
    onPositiveClick: () => {
      guard(() => removeEvent(props.event.id));
    },
  });
}
</script>

<template>
  <div class="event-card" :class="{ archived: isDone, 'is-expanded': expanded }">
    <div class="card-head">
      <button class="star" :title="event.starred ? '取消置顶' : '置顶'" @click="onToggleStar">
        {{ event.starred ? "★" : "☆" }}
      </button>

      <div class="head-main" @click="emit('toggle', event.id)">
        <div class="title-row">
          <span class="title">{{ event.title }}</span>
          <NTag v-if="isDone" size="small" type="success" :bordered="false">已归档</NTag>
        </div>
        <div class="meta">
          <span class="date" :title="'创建于 ' + fmtDateTime(event.created_at)">
            📅 {{ createdDate }}
          </span>
          <span v-if="archivedDate" class="date archived-date" :title="'归档于 ' + fmtDateTime(event.archived_at)">
            ✓ {{ archivedDate }}
          </span>
          <span v-if="stepCount" class="count">{{ stepCount }} 条进展</span>
          <span v-else class="count empty-count">尚无进展</span>
          <span v-if="currentStep && !expanded" class="current">
            {{ STEP_META[currentStep.state].icon }} {{ currentStep.text }}
          </span>
          <span v-if="event.note && expanded" class="note">{{ event.note }}</span>
        </div>
      </div>

      <div class="actions">
        <NDropdown
          trigger="click"
          :options="categoryOptions"
          @select="onCategorySelect"
        >
          <NButton size="tiny" quaternary :title="'当前分类：' + (event.category || '未分类')">
            🏷 {{ event.category || "未分类" }}
          </NButton>
        </NDropdown>
        <NButton size="tiny" quaternary @click="onToggleStatus">
          {{ isDone ? "重开" : "归档" }}
        </NButton>
        <NButton size="tiny" quaternary type="error" @click="onRemoveEvent">删除</NButton>
        <button class="fold" @click="emit('toggle', event.id)">
          {{ expanded ? "▾" : "▸" }}
        </button>
      </div>
    </div>

    <Transition name="collapse">
      <div v-if="expanded" class="timeline-wrap">
        <div class="timeline-inner">
          <div class="timeline">
          <div v-if="!event.steps.length" class="empty-steps">
            还没有进展，在下方输入第一步处理动作
          </div>

          <div
            v-for="(s, i) in event.steps"
            :key="s.id"
            class="step"
            :class="s.state"
          >
            <div class="rail">
              <button
                class="dot"
                :title="`点击切换状态（当前：${STEP_META[s.state].label}）`"
                @click="onCycle(s.id)"
              >
                {{ STEP_META[s.state].icon }}
              </button>
              <div v-if="i !== event.steps.length - 1" class="line" />
            </div>

            <div class="step-body">
              <div class="step-text">{{ s.text }}</div>
              <div class="step-meta">
                <span>{{ STEP_META[s.state].label }}</span>
                <span>{{ fmtDateTime(s.created_at) }}</span>
                <button class="del-step" title="删除该进展" @click="onRemoveStep(s.id)">
                  ✕
                </button>
              </div>
            </div>
          </div>

          <div class="add-step">
            <NInput
              v-model:value="stepDraft"
              size="small"
              placeholder="记录下一步进展，回车添加（上一步会自动标记完成）"
              :disabled="busy"
              @keyup.enter="submitStep"
            />
            <NButton size="small" type="primary" :disabled="!stepDraft.trim()" @click="submitStep">
              添加
            </NButton>
          </div>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.event-card {
  background: #ffffff;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  padding: 10px 12px;
  transition:
    box-shadow 0.2s ease,
    border-color 0.2s ease;
}
/* 展开态：卡片整体抬起并高亮左边缘，强调「当前正在看这一个」 */
.event-card.is-expanded {
  border-color: #c7dbfa;
  box-shadow: 0 2px 10px rgba(47, 128, 237, 0.1);
}
.event-card.archived {
  opacity: 0.62;
}
.card-head {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  cursor: pointer;
}
.star,
.fold,
.del-step {
  background: none;
  border: none;
  cursor: pointer;
  padding: 0;
  color: #9ca3af;
  font-size: 14px;
  line-height: 1.2;
}
.star {
  color: #f59e0b;
  font-size: 16px;
}
.head-main {
  flex: 1;
  min-width: 0;
  cursor: pointer;
}
.title-row {
  display: flex;
  align-items: center;
  gap: 6px;
}
.title {
  font-size: 14px;
  font-weight: 600;
  color: #111827;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.meta {
  margin-top: 3px;
  font-size: 12px;
  color: #6b7280;
  display: flex;
  gap: 8px;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}
.count {
  flex: none;
  padding: 0 6px;
  background: #f1f3f5;
  border-radius: 4px;
  font-size: 11px;
}
.date {
  flex: none;
  color: #9ca3af;
  font-size: 11px;
  font-variant-numeric: tabular-nums;
}
.archived-date {
  color: #18a058;
  font-weight: 500;
}
.empty-count {
  color: #b0b5bb;
}
.current {
  color: #2f80ed;
  overflow: hidden;
  text-overflow: ellipsis;
}
.note {
  color: #9ca3af;
  overflow: hidden;
  text-overflow: ellipsis;
}
.actions {
  display: flex;
  align-items: center;
  gap: 2px;
  flex: none;
}

/* 展开区做成「内嵌面板」：浅灰底 + 内缩，
   与白色标题区形成层次差，但仍在同一张卡片边框内 */
.timeline {
  padding: 10px 12px 4px 12px;
  background: #f8f9fb;
  border: 1px solid #eef0f3;
  border-radius: 6px;
  position: relative;
}
/* 面板顶部的小三角，指向标题区，暗示两者的从属关系 */
.timeline::before {
  content: "";
  position: absolute;
  top: -5px;
  left: 18px;
  width: 8px;
  height: 8px;
  background: #f8f9fb;
  border-left: 1px solid #eef0f3;
  border-top: 1px solid #eef0f3;
  transform: rotate(45deg);
}

/* 展开/收起过渡：用 grid-template-rows 0fr→1fr 实现自动高度动画，
   无需 JS 测量元素高度 */
.timeline-wrap {
  display: grid;
  grid-template-rows: 1fr;
  overflow: hidden;
}
/* 内层再留间距，避免 wrap 的 overflow:hidden 裁掉面板小三角 */
.timeline-inner {
  padding-top: 12px;
}
.collapse-enter-active,
.collapse-leave-active {
  transition:
    grid-template-rows 0.24s ease,
    opacity 0.24s ease;
}
.collapse-enter-from,
.collapse-leave-to {
  grid-template-rows: 0fr;
  opacity: 0;
}
.collapse-enter-from > .timeline-inner,
.collapse-leave-to > .timeline-inner {
  overflow: hidden;
}
.empty-steps {
  font-size: 12px;
  color: #9ca3af;
  padding: 4px 0 8px 0;
}
.step {
  display: flex;
  gap: 8px;
}
.rail {
  display: flex;
  flex-direction: column;
  align-items: center;
  flex: none;
  width: 18px;
}
.dot {
  background: none;
  border: none;
  cursor: pointer;
  padding: 0;
  font-size: 13px;
  line-height: 16px;
  color: #c0c4cc;
}
.step.doing .dot {
  color: #2f80ed;
}
.step.done .dot {
  color: #18a058;
}
.line {
  flex: 1;
  width: 1px;
  background: #e5e7eb;
  margin: 2px 0;
}
.step-body {
  flex: 1;
  min-width: 0;
  padding-bottom: 10px;
}
.step-text {
  font-size: 13px;
  color: #374151;
  word-break: break-word;
}
.step.done .step-text {
  color: #9ca3af;
  text-decoration: line-through;
}
.step-meta {
  margin-top: 2px;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 11px;
  color: #9ca3af;
}
.add-step {
  display: flex;
  gap: 6px;
  margin-top: 2px;
}
</style>
