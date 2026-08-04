<script setup lang="ts">
// 单个项目卡片
import { computed, ref } from "vue";
import {
  NBadge,
  NButton,
  NDropdown,
  NSpace,
  NTag,
  useDialog,
  useMessage,
} from "naive-ui";
import type { OpenKind, Project } from "../types";
import {
  openTarget,
  removeProject,
  renameProject,
  revealInExplorer,
  selectCbp,
  selectDcf,
  toggleProjectStar,
} from "../api";

const props = defineProps<{ project: Project; invalid: boolean }>();
const emit = defineEmits<{
  (e: "refresh"): void;
  (e: "duplicate", id: string): void;
}>();

const message = useMessage();
const dialog = useDialog();

const editing = ref(false);
const nameDraft = ref(props.project.name);

const vFocusSelect = {
  mounted(el: HTMLInputElement) {
    el.focus();
    el.select();
  },
};

function startEdit() {
  if (props.invalid) return;
  nameDraft.value = props.project.name;
  editing.value = true;
}

function cancelEdit() {
  editing.value = false;
  nameDraft.value = props.project.name;
}

let committing = false;
async function commitEdit() {
  if (committing) return;
  committing = true;
  try {
    const trimmed = nameDraft.value.trim();
    if (!trimmed || trimmed === props.project.name) {
      editing.value = false;
      return;
    }
    await renameProject(props.project.id, trimmed);
    message.success("重命名成功");
    editing.value = false;
    emit("refresh");
  } catch (e) {
    message.error(String(e));
    // 失败保持编辑态方便修改，或回退
    nameDraft.value = props.project.name;
    editing.value = false;
  } finally {
    committing = false;
  }
}

async function run(kind: OpenKind) {
  if (props.invalid) return;
  try {
    await openTarget(kind, props.project.id);
    emit("refresh");
  } catch (e) {
    message.error(String(e));
    emit("refresh");
  }
}

async function toggleStar() {
  try {
    await toggleProjectStar(props.project.id);
    emit("refresh");
  } catch (e) {
    message.error(String(e));
  }
}

function confirmRemove() {
  dialog.warning({
    title: "移除该项目？",
    content: `将从列表中移除『${props.project.name}』（不会删除硬盘上的文件夹）。`,
    positiveText: "移除",
    negativeText: "取消",
    onPositiveClick: async () => {
      try {
        await removeProject(props.project.id);
        message.success("已移除");
        emit("refresh");
      } catch (e) {
        message.error(String(e));
      }
    },
  });
}

function duplicate() {
  if (props.invalid) return;
  emit("duplicate", props.project.id);
}

// ---------- 红点下拉：cbp / dcf ----------
const REVEAL_KEY = "__reveal_dcf__";

/** 当前生效的 cbp / dcf（未显式选择时取第一个，与后端 open 逻辑一致） */
const currentCbp = computed(
  () => props.project.selected_cbp ?? props.project.cbp_files[0] ?? null,
);
const currentDcf = computed(
  () => props.project.selected_dcf ?? props.project.dcf_files[0] ?? null,
);

const cbpOptions = computed(() =>
  props.project.cbp_files.map((p) => ({
    label: `${p === currentCbp.value ? "✔ " : "　"}${fileNameOf(p)}`,
    key: p,
  })),
);
/**
 * dcf 右键菜单：
 * - 顶部固定一条"在资源管理器中定位"（针对当前 selected_dcf）
 * - 有多个 dcf 时，下面列出所有 dcf 供切换，当前项前面带 ✔
 */
const dcfOptions = computed(() => {
  const items: any[] = [];
  const cur = currentDcf.value;
  if (cur) {
    items.push({
      key: REVEAL_KEY,
      label: `📂 在资源管理器中定位 · ${fileNameOf(cur)}`,
    });
  }
  if (props.project.dcf_files.length > 1) {
    items.push({ type: "divider", key: "__div__" });
    for (const p of props.project.dcf_files) {
      items.push({
        label: `${p === cur ? "✔ " : "　"}${fileNameOf(p)}`,
        key: p,
      });
    }
  }
  return items;
});
const hasMultiCbp = computed(() => props.project.cbp_files.length > 1);
/** 有 dcf 就允许右键（哪怕只有 1 个，也可以定位） */
const hasDcf = computed(() => props.project.dcf_files.length > 0);
const hasMultiDcf = computed(() => props.project.dcf_files.length > 1);

/** 悬浮提示里直接显示当前生效的文件名，不用右键也能确认用的是哪一个 */
const cbpTitle = computed(() => {
  if (!currentCbp.value) return "CodeBlocks 打开（未找到 .cbp）";
  const base = `当前：${fileNameOf(currentCbp.value)}`;
  return hasMultiCbp.value
    ? `${base}\n共 ${props.project.cbp_files.length} 个 · 左键打开 · 右键切换`
    : `${base}\n左键打开`;
});
const dcfTitle = computed(() => {
  if (!currentDcf.value) return "烧录工具（未找到 .dcf）";
  const base = `当前：${fileNameOf(currentDcf.value)}`;
  return hasMultiDcf.value
    ? `${base}\n共 ${props.project.dcf_files.length} 个 · 左键打开 · 右键切换/定位`
    : `${base}\n左键打开 · 右键定位`;
});

function fileNameOf(p: string) {
  const idx = Math.max(p.lastIndexOf("\\"), p.lastIndexOf("/"));
  return idx >= 0 ? p.slice(idx + 1) : p;
}

async function onSelectCbp(path: string) {
  cbpMenuShow.value = false;
  try {
    await selectCbp(props.project.id, path);
    message.success(`已选择：${fileNameOf(path)}`);
    emit("refresh");
  } catch (e) {
    message.error(String(e));
  }
}

async function onSelectDcf(key: string) {
  dcfMenuShow.value = false;
  // 特殊键：定位当前 dcf 到资源管理器
  if (key === REVEAL_KEY) {
    const target = currentDcf.value;
    if (!target) return;
    try {
      await revealInExplorer(target);
    } catch (e) {
      message.error(String(e));
    }
    return;
  }
  // 否则视为选择新的 dcf
  try {
    await selectDcf(props.project.id, key);
    message.success(`已选择：${fileNameOf(key)}`);
    emit("refresh");
  } catch (e) {
    message.error(String(e));
  }
}

// ---- 右键弹选择菜单（cbp/dcf 各一份） ----
const cbpMenuShow = ref(false);
const cbpMenuX = ref(0);
const cbpMenuY = ref(0);
const dcfMenuShow = ref(false);
const dcfMenuX = ref(0);
const dcfMenuY = ref(0);

function onContextCbp(e: MouseEvent) {
  if (props.invalid || !hasMultiCbp.value) return;
  e.preventDefault();
  cbpMenuX.value = e.clientX;
  cbpMenuY.value = e.clientY;
  cbpMenuShow.value = false;
  requestAnimationFrame(() => (cbpMenuShow.value = true));
}

function onContextDcf(e: MouseEvent) {
  if (props.invalid || !hasDcf.value) return;
  e.preventDefault();
  dcfMenuX.value = e.clientX;
  dcfMenuY.value = e.clientY;
  dcfMenuShow.value = false;
  requestAnimationFrame(() => (dcfMenuShow.value = true));
}
</script>

<template>
  <div class="card" :class="{ invalid }">
    <div class="left">
      <span
        class="star"
        :class="{ starred: project.starred }"
        @click="toggleStar"
        title="星标"
      >
        {{ project.starred ? "⭐" : "☆" }}
      </span>
      <span
        v-if="!editing"
        class="name"
        :class="{ 'name-invalid': invalid }"
        @dblclick="startEdit"
        :title="`双击重命名 · ${project.path}`"
      >
        {{ project.name }}
      </span>
      <input
        v-else
        v-model="nameDraft"
        class="name-input"
        @keyup.enter="commitEdit"
        @keyup.esc="cancelEdit"
        @blur="commitEdit"
        v-focus-select
      />
      <NTag v-if="invalid" size="small" type="warning">路径失效</NTag>
    </div>

    <NSpace :size="4" :wrap="false">
      <NButton
        size="small"
        :disabled="invalid"
        title="打开文件夹"
        @click="run('folder')"
      >
        📁
      </NButton>
      <NButton
        size="small"
        :disabled="invalid"
        title="用默认 IDE 打开（VSCode / Trae，可在设置切换）"
        @click="run('vscode')"
      >
        <span class="label-text">IDE</span>
      </NButton>

      <!-- CodeBlocks + 红点（静态） · 左键直接打开 · 右键选择 -->
      <NBadge dot :show="hasMultiCbp" :processing="false" color="#d03050">
        <NButton
          size="small"
          :disabled="invalid"
          :title="cbpTitle"
          @click="run('codeblocks')"
          @contextmenu="onContextCbp"
        >
          <span class="label-text">.cbp</span>
        </NButton>
      </NBadge>

      <!-- 烧录 + 红点（静态） · 左键直接打开 · 右键选择/定位 -->
      <NBadge dot :show="hasMultiDcf" :processing="false" color="#d03050">
        <NButton
          size="small"
          :disabled="invalid"
          :title="dcfTitle"
          @click="run('burn')"
          @contextmenu="onContextDcf"
        >
          <span class="label-text">.dcf</span>
        </NButton>
      </NBadge>

      <NButton
        size="small"
        :disabled="invalid"
        title="复制副本"
        @click="duplicate"
      >
        <span class="label-text">copy</span>
      </NButton>
      <NButton size="small" title="移除" @click="confirmRemove">
        <span class="remove-x">✕</span>
      </NButton>
    </NSpace>

    <!-- 手动定位的右键菜单：cbp / dcf -->
    <NDropdown
      placement="bottom-start"
      trigger="manual"
      :x="cbpMenuX"
      :y="cbpMenuY"
      :options="cbpOptions"
      :show="cbpMenuShow"
      @select="onSelectCbp"
      @clickoutside="cbpMenuShow = false"
    />
    <NDropdown
      placement="bottom-start"
      trigger="manual"
      :x="dcfMenuX"
      :y="dcfMenuY"
      :options="dcfOptions"
      :show="dcfMenuShow"
      @select="onSelectDcf"
      @clickoutside="dcfMenuShow = false"
    />
  </div>
</template>

<style scoped>
.card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 8px 12px;
  background: #ffffff;
  border: 1px solid #eef0f3;
  border-radius: 6px;
  transition: box-shadow 0.15s;
}
.card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}
.card.invalid {
  background: #fafafa;
}
.left {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
  flex: 1;
}
.star {
  cursor: pointer;
  color: #d1d5db;
  user-select: none;
  transition: color 0.15s;
}
.star.starred {
  color: #f5a524;
}
.name {
  font-weight: 500;
  cursor: text;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 320px;
}
.name-invalid {
  color: #9ca3af;
  font-style: italic;
}
.name-input {
  font-size: 14px;
  padding: 2px 6px;
  border: 1px solid #d1d5db;
  border-radius: 4px;
  outline: none;
  width: 240px;
}
/* 文字型功能按钮：斜体 + 粗体，便于快速区分 */
.label-text {
  font-style: italic;
  font-weight: 700;
  font-size: 12px;
  letter-spacing: 0.2px;
}
/* 移除按钮的叉：略加粗，视觉上与文字按钮区分 */
.remove-x {
  font-weight: 700;
  font-size: 13px;
  line-height: 1;
}
</style>
