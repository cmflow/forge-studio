<script setup lang="ts">
// 快捷应用区：拖拽/选择 exe 添加、右键删除/星标、点击启动
import { computed, onMounted, ref } from "vue";
import { NButton, NDropdown, NSpace, useDialog, useMessage } from "naive-ui";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import {
  addLauncher,
  listLaunchers,
  removeLauncher,
  runLauncher,
  toggleLauncherStar,
} from "../api";
import type { Launcher } from "../types";

const message = useMessage();
const dialog = useDialog();

const launchers = ref<Launcher[]>([]);

async function refresh() {
  try {
    launchers.value = await listLaunchers();
  } catch (e) {
    launchers.value = [];
  }
}

onMounted(refresh);

// 星标置顶，其余按加入顺序（此处即 JSON 里的顺序）
const sorted = computed(() => {
  const arr = launchers.value.slice();
  arr.sort((a, b) => {
    if (a.starred !== b.starred) return a.starred ? -1 : 1;
    return 0;
  });
  return arr;
});

async function pickAndAdd() {
  try {
    const picked = await openDialog({
      title: "选择应用可执行文件",
      multiple: false,
      directory: false,
      filters: [
        { name: "可执行文件", extensions: ["exe", "bat", "cmd", "lnk"] },
      ],
    });
    if (typeof picked === "string" && picked) {
      const item = await addLauncher("", picked);
      message.success(`已添加：${item.name}`);
      await refresh();
    }
  } catch (e) {
    message.error(String(e));
  }
}

async function run(item: Launcher) {
  try {
    await runLauncher(item.id);
  } catch (e) {
    message.error(String(e));
  }
}

// ---- 右键菜单 ----
const menuShow = ref(false);
const menuX = ref(0);
const menuY = ref(0);
const menuTarget = ref<Launcher | null>(null);

const menuOptions = computed(() => {
  const it = menuTarget.value;
  return [
    {
      key: "toggle-star",
      label: it?.starred ? "取消星标" : "标为星标",
    },
    { type: "divider", key: "d1" },
    { key: "remove", label: "移除该应用", props: { style: "color:#d03050" } },
  ];
});

function onContextMenu(e: MouseEvent, item: Launcher) {
  e.preventDefault();
  menuTarget.value = item;
  menuX.value = e.clientX;
  menuY.value = e.clientY;
  menuShow.value = false;
  // nextTick 让下拉先关闭再展开，避免定位错乱
  requestAnimationFrame(() => (menuShow.value = true));
}

async function onMenuSelect(key: string) {
  menuShow.value = false;
  const it = menuTarget.value;
  if (!it) return;
  try {
    if (key === "toggle-star") {
      await toggleLauncherStar(it.id);
      await refresh();
    } else if (key === "remove") {
      dialog.warning({
        title: "移除该应用？",
        content: `将从快捷应用中移除『${it.name}』（不会删除硬盘上的文件）。`,
        positiveText: "移除",
        negativeText: "取消",
        onPositiveClick: async () => {
          try {
            await removeLauncher(it.id);
            message.success("已移除");
            await refresh();
          } catch (e) {
            message.error(String(e));
          }
        },
      });
    }
  } catch (e) {
    message.error(String(e));
  }
}

function onClickOutside() {
  menuShow.value = false;
}
</script>

<template>
  <div class="launcher-bar">
    <NSpace :size="8" wrap>
      <NButton
        v-for="item in sorted"
        :key="item.id"
        secondary
        size="small"
        :title="`左键启动 · 右键菜单\n${item.path}`"
        @click="run(item)"
        @contextmenu="(e: MouseEvent) => onContextMenu(e, item)"
      >
        <span v-if="item.starred" style="margin-right: 4px">⭐</span>{{ item.name }}
      </NButton>
      <NButton dashed size="small" @click="pickAndAdd">＋ 添加应用</NButton>
      <span v-if="!launchers.length" class="hint">
        （选择或拖拽 exe 到这里，右键可管理）
      </span>
    </NSpace>

    <NDropdown
      placement="bottom-start"
      trigger="manual"
      :x="menuX"
      :y="menuY"
      :options="menuOptions"
      :show="menuShow"
      @select="onMenuSelect"
      @clickoutside="onClickOutside"
    />
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
