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
  scanProject,
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

async function rescan() {
  if (props.invalid) return;
  try {
    await scanProject(props.project.id);
    message.success("扫描完成");
    emit("refresh");
  } catch (e) {
    message.error(String(e));
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
const cbpOptions = computed(() =>
  props.project.cbp_files.map((p) => ({
    label: fileNameOf(p),
    key: p,
  })),
);
const dcfOptions = computed(() =>
  props.project.dcf_files.map((p) => ({
    label: fileNameOf(p),
    key: p,
  })),
);
const hasMultiCbp = computed(() => props.project.cbp_files.length > 1);
const hasMultiDcf = computed(() => props.project.dcf_files.length > 1);

function fileNameOf(p: string) {
  const idx = Math.max(p.lastIndexOf("\\"), p.lastIndexOf("/"));
  return idx >= 0 ? p.slice(idx + 1) : p;
}

async function onSelectCbp(path: string) {
  try {
    await selectCbp(props.project.id, path);
    message.success(`已选择：${fileNameOf(path)}`);
    emit("refresh");
  } catch (e) {
    message.error(String(e));
  }
}

async function onSelectDcf(path: string) {
  try {
    await selectDcf(props.project.id, path);
    message.success(`已选择：${fileNameOf(path)}`);
    emit("refresh");
  } catch (e) {
    message.error(String(e));
  }
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
        title="VSCode 打开"
        @click="run('vscode')"
      >
        ✍️
      </NButton>

      <!-- CodeBlocks + 红点下拉 -->
      <NBadge dot :show="hasMultiCbp" processing>
        <template v-if="hasMultiCbp">
          <NDropdown
            trigger="click"
            :options="cbpOptions"
            @select="onSelectCbp"
          >
            <NButton
              size="small"
              :disabled="invalid"
              title="CodeBlocks 打开（多个 .cbp，点开选择）"
            >
              🔧
            </NButton>
          </NDropdown>
        </template>
        <template v-else>
          <NButton
            size="small"
            :disabled="invalid"
            title="CodeBlocks 打开"
            @click="run('codeblocks')"
          >
            🔧
          </NButton>
        </template>
      </NBadge>

      <!-- 烧录 + 红点下拉 -->
      <NBadge dot :show="hasMultiDcf" processing>
        <template v-if="hasMultiDcf">
          <NDropdown
            trigger="click"
            :options="dcfOptions"
            @select="onSelectDcf"
          >
            <NButton
              size="small"
              :disabled="invalid"
              title="烧录工具（多个 .dcf，点开选择）"
            >
              🔥
            </NButton>
          </NDropdown>
        </template>
        <template v-else>
          <NButton
            size="small"
            :disabled="invalid"
            title="烧录工具"
            @click="run('burn')"
          >
            🔥
          </NButton>
        </template>
      </NBadge>

      <NButton
        size="small"
        :disabled="invalid"
        title="复制副本"
        @click="duplicate"
      >
        📋
      </NButton>
      <NButton
        size="small"
        :disabled="invalid"
        title="重扫"
        @click="rescan"
      >
        🔄
      </NButton>
      <NButton size="small" title="移除" @click="confirmRemove">🗑️</NButton>
    </NSpace>
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
</style>
