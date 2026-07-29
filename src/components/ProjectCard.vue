<script setup lang="ts">
// 单个项目卡片：操作按钮全部接线
import { ref } from "vue";
import { NButton, NSpace, NTag, useDialog, useMessage } from "naive-ui";
import type { OpenKind, Project } from "../types";
import {
  openTarget,
  removeProject,
  scanProject,
  toggleProjectStar,
} from "../api";

const props = defineProps<{ project: Project; invalid: boolean }>();
const emit = defineEmits<{ (e: "refresh"): void }>();

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
}

async function commitEdit() {
  // 物理重命名将在下一轮实现，本轮先关闭编辑态
  editing.value = false;
}

async function run(kind: OpenKind) {
  if (props.invalid) return;
  try {
    await openTarget(kind, props.project.id);
    emit("refresh");
  } catch (e) {
    message.error(String(e));
    emit("refresh"); // 失败也要刷新（last_accessed 已更新）
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
        :title="project.path"
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

    <NSpace :size="4">
      <NButton size="small" :disabled="invalid" title="打开文件夹" @click="run('folder')">📁</NButton>
      <NButton size="small" :disabled="invalid" title="VSCode 打开" @click="run('vscode')">✍️</NButton>
      <NButton size="small" :disabled="invalid" title="CodeBlocks 打开" @click="run('codeblocks')">🔧</NButton>
      <NButton size="small" :disabled="invalid" title="烧录工具" @click="run('burn')">🔥</NButton>
      <NButton size="small" title="复制副本（待实现）" disabled>📋</NButton>
      <NButton size="small" :disabled="invalid" title="重扫" @click="rescan">🔄</NButton>
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
