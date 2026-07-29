<script setup lang="ts">
// 单个项目卡片（骨架版：展示 + 操作按钮占位）
import { ref } from "vue";
import { NButton, NSpace, NTag } from "naive-ui";
import type { Project } from "../types";

const props = defineProps<{ project: Project; invalid: boolean }>();
const emit = defineEmits<{ (e: "refresh"): void }>();

const editing = ref(false);
const nameDraft = ref(props.project.name);

// 双击进入编辑时自动全选（script setup 中以 v 前缀导出即为局部指令）
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
  // 后续接入 renameProject
  editing.value = false;
}
</script>

<template>
  <div class="card" :class="{ invalid }">
    <div class="left">
      <span v-if="project.starred" class="star">⭐</span>
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
      <NButton size="small" :disabled="invalid" title="打开文件夹">📁</NButton>
      <NButton size="small" :disabled="invalid" title="VSCode 打开">✍️</NButton>
      <NButton size="small" :disabled="invalid" title="CodeBlocks 打开">🔧</NButton>
      <NButton size="small" :disabled="invalid" title="烧录工具">🔥</NButton>
      <NButton size="small" :disabled="invalid" title="复制副本">📋</NButton>
      <NButton size="small" :disabled="invalid" title="重扫">🔄</NButton>
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
