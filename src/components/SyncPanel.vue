<script setup lang="ts">
// 事件进展的云同步面板：上传本机存档 + 从任一设备存档恢复
// 每台设备在云端是独立文件，所以上传永远不会覆盖别人的数据
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { NButton, NSpin, NSwitch, useDialog, useMessage } from "naive-ui";
import {
  getDeviceInfo,
  getSyncSettings,
  listRemoteArchives,
  pullEvents,
  pushEvents,
  setSyncAutoPush,
} from "../api";
import type { RemoteArchive, SyncSettings } from "../types";
import { fmtDateTime } from "../utils/date";

const props = defineProps<{
  embedded?: boolean;
  /** 外部每次自增时重新拉取设置（设置弹窗关闭等时机） */
  version?: number;
}>();
const emit = defineEmits<{
  (e: "pulled"): void;
  (e: "goto-settings"): void;
}>();

const message = useMessage();
const dialog = useDialog();

const settings = ref<SyncSettings | null>(null);
const deviceName = ref("");
const deviceId = ref("");
const archives = ref<RemoteArchive[]>([]);
const listing = ref(false);
const pushing = ref(false);
const pulling = ref("");
const expanded = ref(false);
/** 上次上传时间（本地会话内记录，仅用于界面提示） */
const lastPushAt = ref(0);
/** init 失败原因（展示在未启用条上，便于排查） */
const initError = ref("");

const enabled = computed(() => !!settings.value?.enabled);

async function init() {
  try {
    const [s, [id, name]] = await Promise.all([getSyncSettings(), getDeviceInfo()]);
    settings.value = s;
    deviceId.value = id;
    deviceName.value = name;
    initError.value = "";
  } catch (e) {
    initError.value = String(e);
    console.error("SyncPanel init 失败:", e);
  }
}

function fmtTime(ts: number) {
  if (!ts) return "从未";
  return fmtDateTime(ts);
}

/** 上传本机存档；检测到篡改时弹窗询问是否强制覆盖 */
async function doPush(force = false) {
  if (pushing.value) return;
  pushing.value = true;
  try {
    const r = await pushEvents(force);
    if (r.ok) {
      lastPushAt.value = r.updated_at;
      message.success(r.message);
      if (expanded.value) await loadArchives();
    } else if (r.tampered) {
      dialog.warning({
        title: "云端存档异常",
        content: r.message,
        positiveText: "强制覆盖上传",
        negativeText: "取消",
        onPositiveClick: () => doPush(true),
      });
    } else {
      message.error(r.message);
    }
  } catch (e) {
    message.error(String(e));
  } finally {
    pushing.value = false;
  }
}

async function loadArchives() {
  if (listing.value) return;
  listing.value = true;
  try {
    archives.value = await listRemoteArchives();
  } catch (e) {
    message.error(String(e));
  } finally {
    listing.value = false;
  }
}

async function toggleExpand() {
  expanded.value = !expanded.value;
  if (expanded.value && !archives.value.length) await loadArchives();
}

/** 下载会覆盖本地全部事件，必须二次确认 */
function doPull(a: RemoteArchive) {
  dialog.warning({
    title: "确认从该存档恢复？",
    content:
      `将用「${a.device_name}」的 ${a.event_count} 条事件覆盖本机全部事件数据，` +
      `本机当前数据会被替换且无法撤销。恢复完成后会自动同步为本机存档。`,
    positiveText: "确认恢复",
    negativeText: "取消",
    onPositiveClick: async () => {
      pulling.value = a.device_id;
      try {
        const r = await pullEvents(a.device_id);
        message.success(r.message);
        emit("pulled");
        await loadArchives();
      } catch (e) {
        message.error(String(e));
      } finally {
        pulling.value = "";
      }
    },
  });
}

async function toggleAutoPush(v: boolean) {
  if (!settings.value) return;
  try {
    await setSyncAutoPush(v);
    settings.value = { ...settings.value, auto_push: v };
  } catch (e) {
    message.error(String(e));
  }
}

// ---------- 自动上传定时器 ----------
// 每 10 分钟上传一次本机存档。不做轮询下载，避免消耗坚果云
// 600 次/30 分钟的账号级请求额度（多个项目共用该额度）。
const AUTO_PUSH_INTERVAL = 10 * 60 * 1000;
let timer: number | undefined;

function startTimer() {
  timer = window.setInterval(() => {
    if (!enabled.value || !settings.value?.auto_push) return;
    if (pushing.value) return;
    // 静默上传，失败不弹窗打扰；篡改检测不通过时会走 tampered 分支提示
    doPush(false);
  }, AUTO_PUSH_INTERVAL);
}

onMounted(() => {
  init();
  startTimer();
});
// 设置弹窗关闭（version 自增）时重新拉取设置，避免事件页面板停留在旧状态
watch(
  () => props.version,
  () => {
    if (props.version && props.version > 0) init();
  },
);
onUnmounted(() => {
  if (timer) window.clearInterval(timer);
});
</script>

<template>
  <div class="sync-bar">
    <!-- 未启用状态：始终可见，引导去设置里配置（已在设置页内则只给提示） -->
    <div v-if="!enabled" class="disabled">
      <span class="dev">💻 {{ deviceName || "本机" }}</span>
      <span class="hint">坚果云同步未启用</span>
      <span v-if="initError" class="init-err" :title="initError">初始化失败</span>
      <span class="spacer" />
      <NButton v-if="!embedded" size="tiny" @click="emit('goto-settings')">
        去设置
      </NButton>
    </div>

    <template v-else>
      <div class="line">
        <span class="dev" :title="'设备标识：' + deviceId">💻 {{ deviceName }}</span>

        <NButton size="tiny" type="primary" :loading="pushing" @click="doPush(false)">
          上传本机
        </NButton>

        <NButton size="tiny" quaternary @click="toggleExpand">
          {{ expanded ? "收起存档" : "云端存档" }}
        </NButton>

        <span class="spacer" />

        <span class="auto-label">自动上传</span>
        <NSwitch
          size="small"
          :value="settings?.auto_push ?? false"
          @update:value="toggleAutoPush"
        />
        <span v-if="lastPushAt" class="last">上次 {{ fmtTime(lastPushAt) }}</span>
      </div>

      <!-- 云端各设备存档，可任选一份恢复 -->
      <div v-if="expanded" class="archives">
        <NSpin v-if="listing" size="small" />
        <div v-else-if="!archives.length" class="empty">
          云端还没有任何存档，先点「上传本机」创建第一份
        </div>
        <div
          v-for="a in archives"
          v-else
          :key="a.device_id"
          class="arc"
          :class="{ self: a.is_self }"
        >
          <div class="arc-main">
            <div class="arc-name">
              {{ a.device_name }}
              <span v-if="a.is_self" class="badge self-badge">本机</span>
              <span v-if="!a.intact" class="badge warn-badge" title="内容与指纹不一致">
                已被改动
              </span>
            </div>
            <div class="arc-meta">
              {{ a.event_count }} 条事件 · 更新于 {{ fmtTime(a.updated_at) }}
            </div>
          </div>
          <NButton
            size="tiny"
            :loading="pulling === a.device_id"
            :disabled="!a.intact"
            :title="a.intact ? '' : '该存档内容与指纹不一致（可能被外部修改），已禁止恢复以防覆盖本机数据'"
            @click="doPull(a)"
          >
            恢复到此电脑
          </NButton>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.sync-bar {
  background: #ffffff;
  border: 1px solid #e5e7eb;
  border-radius: 8px;
  padding: 8px 12px;
  margin-bottom: 8px;
}
.disabled {
  display: flex;
  align-items: center;
  gap: 8px;
}
.hint {
  font-size: 12px;
  color: #9ca3af;
}
.init-err {
  font-size: 11px;
  color: #d03050;
}
.line {
  display: flex;
  align-items: center;
  gap: 8px;
}
.dev {
  font-size: 13px;
  font-weight: 600;
  color: #374151;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.spacer {
  flex: 1;
}
.auto-label {
  font-size: 12px;
  color: #6b7280;
}
.last {
  font-size: 11px;
  color: #9ca3af;
}
.archives {
  margin-top: 8px;
  border-top: 1px dashed #eceff1;
  padding-top: 8px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.empty {
  font-size: 12px;
  color: #9ca3af;
  padding: 4px 0;
}
.arc {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: 6px;
  background: #f8f9fb;
}
.arc.self {
  background: #eff6ff;
}
.arc-main {
  flex: 1;
  min-width: 0;
}
.arc-name {
  font-size: 13px;
  color: #1f2937;
  display: flex;
  align-items: center;
  gap: 6px;
}
.arc-meta {
  font-size: 11px;
  color: #9ca3af;
}
.badge {
  font-size: 10px;
  padding: 0 6px;
  border-radius: 999px;
  line-height: 16px;
}
.self-badge {
  background: #2f80ed;
  color: #fff;
}
.warn-badge {
  background: #fff3cd;
  color: #9a6700;
}
</style>
