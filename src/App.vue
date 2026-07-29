<script setup lang="ts">
import { ref } from "vue";
import {
  NButton,
  NConfigProvider,
  NDialogProvider,
  NInput,
  NLayout,
  NLayoutContent,
  NLayoutHeader,
  NMessageProvider,
  zhCN,
  dateZhCN,
} from "naive-ui";
import LauncherBar from "./components/LauncherBar.vue";
import ProjectList from "./components/ProjectList.vue";
import SettingsDialog from "./components/SettingsDialog.vue";
import AddProjectButton from "./components/AddProjectButton.vue";

const search = ref("");
const showSettings = ref(false);

// 亮色主题（按需求：不做暗色）
const theme = null;
</script>

<template>
  <NConfigProvider :theme="theme" :locale="zhCN" :date-locale="dateZhCN">
    <NMessageProvider>
      <NDialogProvider>
        <NLayout class="app-root">
          <NLayoutHeader bordered class="app-header">
            <div class="title">🛠️ 工作助手</div>
            <NButton quaternary circle @click="showSettings = true" title="设置">
              ⚙️
            </NButton>
          </NLayoutHeader>

          <NLayoutContent class="app-content" content-style="padding: 12px;">
            <!-- 快捷应用区 -->
            <LauncherBar />

            <!-- 搜索框 -->
            <div class="search-row">
              <NInput
                v-model:value="search"
                placeholder="🔍 搜索项目名（实时过滤）"
                clearable
              />
            </div>

            <!-- 项目卡片列表 -->
            <ProjectList :search="search" />

            <!-- 悬浮添加项目按钮 -->
            <AddProjectButton />
          </NLayoutContent>
        </NLayout>

        <!-- 设置弹窗 -->
        <SettingsDialog v-model:visible="showSettings" />
      </NDialogProvider>
    </NMessageProvider>
  </NConfigProvider>
</template>

<style scoped>
.app-root {
  height: 100vh;
  display: flex;
  flex-direction: column;
}
.app-header {
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  background: #ffffff;
}
.title {
  font-size: 16px;
  font-weight: 600;
}
.app-content {
  flex: 1;
  overflow-y: auto;
}
.search-row {
  margin: 12px 0;
}
</style>
