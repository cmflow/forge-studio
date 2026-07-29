<script setup lang="ts">
// 快捷应用区（占位骨架，后续接入 launchers.json）
import { onMounted, ref } from "vue";
import { NButton, NSpace } from "naive-ui";
import { listLaunchers } from "../api";
import type { Launcher } from "../types";

const launchers = ref<Launcher[]>([]);

async function refresh() {
  try {
    launchers.value = await listLaunchers();
  } catch (e) {
    // 后端未就绪时忽略
    launchers.value = [];
  }
}

onMounted(refresh);
</script>

<template>
  <div class="launcher-bar">
    <NSpace v-if="launchers.length" :size="8" wrap>
      <NButton
        v-for="item in launchers"
        :key="item.id"
        secondary
        size="small"
      >
        <span v-if="item.starred">⭐</span>{{ item.name }}
      </NButton>
      <NButton dashed size="small">＋ 添加应用</NButton>
    </NSpace>
    <NSpace v-else :size="8">
      <NButton dashed size="small">＋ 添加应用</NButton>
      <span class="hint">（快捷应用区：拖入或点击添加常用 exe）</span>
    </NSpace>
  </div>
</template>

<style scoped>
.launcher-bar {
  padding: 8px 4px;
  background: #ffffff;
  border-radius: 6px;
  border: 1px solid #eef0f3;
}
.hint {
  color: #9ca3af;
  font-size: 12px;
  align-self: center;
}
</style>
