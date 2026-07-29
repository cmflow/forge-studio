<script setup lang="ts">
// 右下角悬浮的『添加项目』按钮（占位骨架）
import { NButton, useMessage } from "naive-ui";

const message = useMessage();

async function add() {
  // TODO: 调用 @tauri-apps/plugin-dialog 选择文件夹 → addProject
  message.info("待接入：选择文件夹 → addProject");
}
</script>

<template>
  <div class="fab">
    <NButton type="primary" circle size="large" @click="add" title="添加项目">
      ＋
    </NButton>
  </div>
</template>

<style scoped>
.fab {
  position: fixed;
  right: 24px;
  bottom: 24px;
  z-index: 100;
}
</style>
