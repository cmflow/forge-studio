<script setup lang="ts">
// 快捷应用区：拖拽/选择 exe 添加、右键删除/星标、点击启动
import { computed, onMounted, ref } from "vue";
import { NButton, NDropdown, NSpace, useDialog, useMessage } from "naive-ui";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import {
  addLauncher,
  getLauncherIcon,
  listLaunchers,
  loadConfig,
  removeLauncher,
  runLauncher,
  scanDevUtils,
  toggleLauncherStar,
} from "../api";
import type { Launcher } from "../types";

defineProps<{ dragHover?: boolean }>();

const message = useMessage();
const dialog = useDialog();

const launchers = ref<Launcher[]>([]);
/** id -> DataURL；获取失败时不写入，模板降级为纯文字 */
const icons = ref<Record<string, string>>({});

async function loadIcon(item: Launcher) {
  if (icons.value[item.id]) return;
  try {
    icons.value[item.id] = await getLauncherIcon(item.path, 32);
  } catch {
    // 忽略；模板降级
  }
}

async function refresh() {
  try {
    launchers.value = await listLaunchers();
    // 并行拉取全部图标，不阻塞列表渲染
    launchers.value.forEach(loadIcon);
  } catch (e) {
    launchers.value = [];
  }
}

onMounted(async () => {
  // 启动时若用户开启"自动扫描工具目录"，则先扫一遍，再 refresh
  try {
    const cfg = await loadConfig();
    const root = (cfg.dev_utils_root ?? "").trim();
    if (cfg.scan_dev_utils_on_start && root) {
      try {
        const added = await scanDevUtils(root);
        if (added.length > 0) {
          message.success(
            `已从工具目录自动加入 ${added.length} 个：${added.map((a) => a.name).join("、")}`,
          );
        }
      } catch {
        // 静默：自动扫描失败不影响主流程
      }
    }
  } catch {
    // config 还没生成时忽略
  }
  await refresh();
});

/** 支持拖入作为快捷应用的扩展名（不含快捷方式 .lnk） */
const ALLOWED_EXTS = ["exe", "bat", "cmd"];

/** 由 App.vue 派发调用：把拖入的可执行文件加为快捷应用 */
async function handleDrop(paths: string[]) {
  if (!paths?.length) return;
  let ok = 0;
  let fail = 0;
  let skipped = 0;
  for (const p of paths) {
    const ext = p.split(".").pop()?.toLowerCase() ?? "";
    if (!ALLOWED_EXTS.includes(ext)) {
      skipped++;
      continue;
    }
    try {
      await addLauncher("", p);
      ok++;
    } catch (e) {
      fail++;
      message.error(`添加失败：${String(e)}`);
    }
  }
  if (ok > 0) {
    message.success(
      `已添加 ${ok} 个应用${fail ? `（失败 ${fail}）` : ""}${skipped ? `（跳过 ${skipped} 个非可执行文件）` : ""}`,
    );
    await refresh();
  } else if (skipped > 0 && fail === 0) {
    message.warning(`快捷应用只支持 ${ALLOWED_EXTS.join(" / ")}，已跳过 ${skipped} 项`);
  }
}

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
        { name: "可执行文件", extensions: ["exe", "bat", "cmd"] },
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

defineExpose({ handleDrop });
</script>

<template>
  <div class="launcher-bar" :class="{ 'drag-hover': dragHover }">
    <NSpace :size="[8, 8]" wrap align="center" item-style="max-width: 100%;">
      <NButton
        v-for="item in sorted"
        :key="item.id"
        secondary
        size="small"
        class="launcher-btn"
        :title="`左键启动 · 右键菜单\n${item.path}`"
        @click="run(item)"
        @contextmenu="(e: MouseEvent) => onContextMenu(e, item)"
      >
        <span class="btn-inner">
          <img
            v-if="icons[item.id]"
            :src="icons[item.id]"
            class="launcher-icon"
            alt=""
            draggable="false"
          />
          <span v-else class="launcher-icon icon-fallback">📦</span>
          <span class="launcher-name">
            <span v-if="item.starred" style="margin-right: 2px">⭐</span>{{ item.name }}
          </span>
        </span>
      </NButton>
      <NButton dashed size="small" @click="pickAndAdd">＋ 添加应用</NButton>
      <span v-if="!launchers.length" class="hint">
        （拖 exe/bat/cmd 到此处，或点『＋ 添加应用』）
      </span>
    </NSpace>

    <div v-if="dragHover" class="drop-hint-inline">📥 松开鼠标即可添加为快捷应用</div>

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
  position: relative;
  transition: outline 0.12s;
}
.launcher-bar.drag-hover {
  outline: 2px dashed #2f80ed;
  outline-offset: -4px;
}
.drop-hint-inline {
  margin-top: 6px;
  padding: 4px 10px;
  background: rgba(47, 128, 237, 0.9);
  color: #fff;
  border-radius: 4px;
  font-size: 12px;
  display: inline-block;
}
.hint {
  color: #9ca3af;
  font-size: 12px;
  align-self: center;
}
.launcher-btn {
  /* 关掉 NButton 默认的 line-height: 1，避免 descender（g/y/p）被裁 */
  line-height: 1.4;
}
.launcher-btn :deep(.n-button__content) {
  display: flex;
  align-items: center;
}
.btn-inner {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  /* 给文字预留 1px 下边距，确保 g 的下钩不被切 */
  padding: 1px 0;
}
.launcher-icon {
  width: 16px;
  height: 16px;
  object-fit: contain;
  flex-shrink: 0;
  image-rendering: -webkit-optimize-contrast;
}
.icon-fallback {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  line-height: 1;
}
.launcher-name {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 180px;
}
</style>
