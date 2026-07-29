<script setup lang="ts">
// 设置弹窗
import { ref, watch } from "vue";
import {
  NButton,
  NInput,
  NInputGroup,
  NModal,
  NSpace,
  useDialog,
  useMessage,
} from "naive-ui";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { clearAllData, loadConfig, openLogsDir, saveConfig } from "../api";
import type { AppConfig } from "../types";

const props = defineProps<{ visible: boolean }>();
const emit = defineEmits<{ (e: "update:visible", v: boolean): void }>();

const message = useMessage();
const dialog = useDialog();

const cfg = ref<AppConfig>({
  vscode_path: "",
  codeblocks_path: "",
  burn_tool_path: "",
});

watch(
  () => props.visible,
  async (v) => {
    if (v) {
      try {
        cfg.value = await loadConfig();
      } catch (e) {
        // 首次运行时忽略
      }
    }
  },
);

async function pickExe(field: keyof AppConfig, title: string) {
  try {
    const picked = await openDialog({
      title,
      multiple: false,
      directory: false,
      filters: [{ name: "可执行文件", extensions: ["exe"] }],
    });
    if (typeof picked === "string" && picked) {
      cfg.value[field] = picked;
    }
  } catch (e) {
    message.error(String(e));
  }
}

async function save() {
  try {
    await saveConfig(cfg.value);
    message.success("已保存");
    emit("update:visible", false);
  } catch (e) {
    message.error(String(e));
  }
}

async function openLogs() {
  try {
    await openLogsDir();
  } catch (e) {
    message.error(String(e));
  }
}

function confirmClear() {
  dialog.warning({
    title: "确认清空所有数据？",
    content:
      "该操作将删除 config.json / launchers.json / projects.json，不可撤销。",
    positiveText: "清空",
    negativeText: "取消",
    onPositiveClick: async () => {
      try {
        await clearAllData();
        message.success("已清空，请重启软件或手动刷新");
        emit("update:visible", false);
      } catch (e) {
        message.error(String(e));
      }
    },
  });
}
</script>

<template>
  <NModal
    :show="visible"
    preset="card"
    title="设置"
    style="width: 620px"
    @update:show="(v) => emit('update:visible', v)"
  >
    <NSpace vertical :size="12">
      <div>
        <div class="label">VSCode 路径</div>
        <NInputGroup>
          <NInput v-model:value="cfg.vscode_path" placeholder="Code.exe 完整路径" />
          <NButton @click="pickExe('vscode_path', '选择 VSCode 可执行文件')">
            浏览…
          </NButton>
        </NInputGroup>
      </div>

      <div>
        <div class="label">CodeBlocks 路径</div>
        <NInputGroup>
          <NInput
            v-model:value="cfg.codeblocks_path"
            placeholder="codeblocks.exe 完整路径"
          />
          <NButton
            @click="pickExe('codeblocks_path', '选择 CodeBlocks 可执行文件')"
          >
            浏览…
          </NButton>
        </NInputGroup>
      </div>

      <div>
        <div class="label">烧录工具路径</div>
        <NInputGroup>
          <NInput
            v-model:value="cfg.burn_tool_path"
            placeholder="BurnTool.exe 完整路径"
          />
          <NButton @click="pickExe('burn_tool_path', '选择烧录工具')">
            浏览…
          </NButton>
        </NInputGroup>
      </div>

      <NSpace justify="space-between">
        <NSpace>
          <NButton @click="openLogs">打开日志目录</NButton>
          <NButton type="error" ghost @click="confirmClear">清空所有数据</NButton>
        </NSpace>
        <NSpace>
          <NButton @click="emit('update:visible', false)">取消</NButton>
          <NButton type="primary" @click="save">保存</NButton>
        </NSpace>
      </NSpace>
    </NSpace>
  </NModal>
</template>

<style scoped>
.label {
  font-size: 12px;
  color: #6b7280;
  margin-bottom: 4px;
}
</style>
