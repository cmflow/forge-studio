<script setup lang="ts">
// 右下角悬浮的『添加项目』按钮
import { NButton, useMessage } from "naive-ui";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { addProject } from "../api";

const message = useMessage();
const emit = defineEmits<{ (e: "added"): void }>();

async function pick() {
  try {
    const picked = await openDialog({
      title: "选择项目文件夹",
      multiple: false,
      directory: true,
    });
    if (typeof picked === "string" && picked) {
      const project = await addProject(picked);
      message.success(`已添加：${project.name}`);
      emit("added");
    }
  } catch (e) {
    message.error(String(e));
  }
}
</script>

<template>
  <div class="fab">
    <NButton type="primary" circle size="large" @click="pick" title="添加项目">
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
