<script setup lang="ts">
// 设置弹窗
import { ref, watch } from "vue";
import {
  NButton,
  NInput,
  NInputGroup,
  NModal,
  NRadio,
  NRadioGroup,
  NSpace,
  NSwitch,
  NTabPane,
  NTabs,
  useDialog,
  useMessage,
} from "naive-ui";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import {
  clearAllData,
  detectToolPath,
  diagnoseSync,
  getAutostart,
  getSyncCredential,
  loadConfig,
  openLogsDir,
  saveConfig,
  scanDevUtils,
  setAutostart,
  setSyncCredential,
} from "../api";
import type { AppConfig, SyncDiagnostic, WebdavCredential } from "../types";

const props = defineProps<{ visible: boolean }>();
const emit = defineEmits<{ (e: "update:visible", v: boolean): void }>();

const message = useMessage();
const dialog = useDialog();

const cfg = ref<AppConfig>({
  vscode_path: "",
  codeblocks_path: "",
  burn_tool_path: "",
  trae_path: "",
  default_ide: "vscode",
  dev_utils_root: "",
  scan_dev_utils_on_start: false,
});

/** 开机自启开关（直接读写注册表，不存 config.json） */
const autostart = ref(false);
const autostartBusy = ref(false);

/** 坚果云 WebDAV 凭据（跨项目共享，存在 .cloudsync\credential.json） */
const cred = ref<WebdavCredential>({
  server: "https://dav.jianguoyun.com/dav/",
  account: "",
  app_password: "",
});
/** 本项目在云端的目录，多项目共用一个账号时靠它隔离 */
const remoteDir = ref("apps/forge-studio");
const diagnosing = ref(false);
const diagResult = ref<SyncDiagnostic | null>(null);

/** 类型安全的"只设置 string 字段"辅助，避开 keyof AppConfig 含 boolean 时赋值 never 的推断陷阱 */
function setCfgField(field: keyof AppConfig, value: string) {
  (cfg.value as Record<string, unknown>)[field] = value;
}

watch(
  () => props.visible,
  async (v) => {
    if (v) {
      try {
        const loaded = await loadConfig();
        cfg.value = {
          vscode_path: loaded.vscode_path ?? "",
          codeblocks_path: loaded.codeblocks_path ?? "",
          burn_tool_path: loaded.burn_tool_path ?? "",
          trae_path: loaded.trae_path ?? "",
          default_ide: loaded.default_ide?.trim() ? loaded.default_ide : "vscode",
          dev_utils_root: loaded.dev_utils_root ?? "",
          scan_dev_utils_on_start: !!loaded.scan_dev_utils_on_start,
        };
      } catch (e) {
        // 首次运行时忽略
      }
      try {
        autostart.value = await getAutostart();
      } catch (e) {
        autostart.value = false;
      }
      try {
        const loadedCred = await getSyncCredential();
        cred.value = {
          server: loadedCred.server?.trim()
            ? loadedCred.server
            : "https://dav.jianguoyun.com/dav/",
          account: loadedCred.account ?? "",
          app_password: loadedCred.app_password ?? "",
        };
      } catch (e) {
        // 首次运行无凭据文件，保持默认值
      }
      diagResult.value = null;
    }
  },
);

/** 保存凭据并跑一遍连通性自检 */
async function runDiagnose() {
  if (diagnosing.value) return;
  diagnosing.value = true;
  diagResult.value = null;
  try {
    await setSyncCredential(cred.value);
    diagResult.value = await diagnoseSync(remoteDir.value);
    if (diagResult.value.ok) {
      message.success("坚果云连通性自检全部通过");
    } else {
      message.warning("自检未全部通过，请看下方明细");
    }
  } catch (e) {
    message.error(String(e));
  } finally {
    diagnosing.value = false;
  }
}

async function onAutostartChange(v: boolean) {
  autostartBusy.value = true;
  try {
    await setAutostart(v);
    autostart.value = v;
    message.success(v ? "已设置开机自启" : "已取消开机自启");
  } catch (e) {
    message.error(String(e));
    autostart.value = !v;
  } finally {
    autostartBusy.value = false;
  }
}

async function pickExe(field: keyof AppConfig, title: string) {
  try {
    const picked = await openDialog({
      title,
      multiple: false,
      directory: false,
      filters: [{ name: "可执行文件", extensions: ["exe"] }],
    });
    if (typeof picked === "string" && picked) {
      setCfgField(field, picked);
    }
  } catch (e) {
    message.error(String(e));
  }
}

async function autoDetect(
  kind: "vscode" | "codeblocks" | "burn" | "trae",
  field: keyof AppConfig,
  label: string,
) {
  try {
    const found = await detectToolPath(kind);
    if (found) {
      setCfgField(field, found);
      message.success(`已识别到 ${label}：${found}`);
    } else {
      message.warning(`未在常见位置找到 ${label}，请点『浏览…』手动选择`);
    }
  } catch (e) {
    message.error(String(e));
  }
}

async function pickDir(field: keyof AppConfig, title: string) {
  try {
    const picked = await openDialog({ title, multiple: false, directory: true });
    if (typeof picked === "string" && picked) {
      setCfgField(field, picked);
    }
  } catch (e) {
    message.error(String(e));
  }
}

const scanning = ref(false);
async function runScanNow() {
  const root = (cfg.value.dev_utils_root ?? "").trim();
  if (!root) {
    message.warning("请先填写工具目录");
    return;
  }
  scanning.value = true;
  try {
    const added = await scanDevUtils(root);
    if (added.length === 0) {
      message.info("未发现新工具（可能都已在列表中）");
    } else {
      message.success(`已添加 ${added.length} 个工具：${added.map((a) => a.name).join("、")}`);
    }
  } catch (e) {
    message.error(String(e));
  } finally {
    scanning.value = false;
  }
}

async function save() {
  try {
    await saveConfig(cfg.value);
    message.success("已保存");
    // 若开启了"启动时自动扫描"，保存时立即扫一遍，避免改完还得重启
    if (cfg.value.scan_dev_utils_on_start && (cfg.value.dev_utils_root ?? "").trim()) {
      try {
        const added = await scanDevUtils(cfg.value.dev_utils_root.trim());
        if (added.length > 0) {
          message.success(
            `已从工具目录自动加入 ${added.length} 个：${added.map((a) => a.name).join("、")}`,
          );
        } else {
          message.info("工具目录扫描完成，未发现新工具");
        }
      } catch (e) {
        message.error(`扫描失败：${String(e)}`);
      }
    }
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
    <NTabs type="line" animated>
      <!-- 板块一：项目工作台（原有设置） -->
      <NTabPane name="workspace" tab="项目工作台">
        <NSpace vertical :size="12">
          <div class="row">
            <div>
              <div class="label" style="margin-bottom: 0">开机自启</div>
              <div class="hint">开机后自动启动工作助手（写入当前用户注册表 Run 项）</div>
            </div>
            <NSwitch
              :value="autostart"
              :loading="autostartBusy"
              @update:value="onAutostartChange"
            />
          </div>

          <div>
            <div class="label">默认 IDE（点击项目卡片『IDE 打开』按钮时使用）</div>
            <NRadioGroup v-model:value="cfg.default_ide">
              <NRadio value="vscode">VSCode</NRadio>
              <NRadio value="trae">Trae</NRadio>
            </NRadioGroup>
          </div>

          <!-- 只显示当前选中 IDE 的路径设置，避免表单过长 -->
          <div v-if="cfg.default_ide !== 'trae'">
            <div class="label">VSCode 路径</div>
            <NInputGroup>
              <NInput v-model:value="cfg.vscode_path" placeholder="Code.exe 完整路径" />
              <NButton @click="autoDetect('vscode', 'vscode_path', 'VSCode')">
                自动识别
              </NButton>
              <NButton @click="pickExe('vscode_path', '选择 VSCode 可执行文件')">
                浏览…
              </NButton>
            </NInputGroup>
          </div>

          <div v-else>
            <div class="label">Trae 路径</div>
            <NInputGroup>
              <NInput
                v-model:value="cfg.trae_path"
                placeholder="Trae CN.exe / Trae.exe 完整路径"
              />
              <NButton @click="autoDetect('trae', 'trae_path', 'Trae')">
                自动识别
              </NButton>
              <NButton @click="pickExe('trae_path', '选择 Trae 可执行文件')">
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
                @click="autoDetect('codeblocks', 'codeblocks_path', 'CodeBlocks')"
              >
                自动识别
              </NButton>
              <NButton
                @click="pickExe('codeblocks_path', '选择 CodeBlocks 可执行文件')"
              >
                浏览…
              </NButton>
            </NInputGroup>
          </div>

          <div>
            <div class="label">工具目录（自动扫描并加入快捷应用）</div>
            <NInputGroup>
              <NInput
                v-model:value="cfg.dev_utils_root"
                placeholder="例如 C:\dev_utils"
              />
              <NButton @click="pickDir('dev_utils_root', '选择工具根目录')">浏览…</NButton>
              <NButton type="primary" :loading="scanning" @click="runScanNow">
                立即扫描
              </NButton>
            </NInputGroup>
            <div class="row" style="margin-top: 8px">
              <div class="hint" style="flex: 1">
                每层子目录视为一个工具，取该子目录下第一个 .exe；同 exe 文件名已存在则跳过
              </div>
              <NSwitch v-model:value="cfg.scan_dev_utils_on_start" />
            </div>
            <div class="hint" style="text-align: right; margin-top: 2px">
              启动应用时自动扫描
            </div>
          </div>

          <div>
            <div class="label">烧录工具路径</div>
            <NInputGroup>
              <NInput
                v-model:value="cfg.burn_tool_path"
                placeholder="Downloader.exe 完整路径"
              />
              <NButton @click="autoDetect('burn', 'burn_tool_path', '烧录工具')">
                自动识别
              </NButton>
              <NButton @click="pickExe('burn_tool_path', '选择烧录工具')">
                浏览…
              </NButton>
            </NInputGroup>
          </div>
        </NSpace>
      </NTabPane>

      <!-- 板块二：事件进展（云同步等独立设置） -->
      <NTabPane name="events" tab="事件进展">
        <NSpace vertical :size="12">
          <div>
            <div class="label" style="font-weight: 600; color: #374151">
              坚果云同步
            </div>
            <div class="hint" style="margin-bottom: 6px">
              凭据存在 %USERPROFILE%\.cloudsync\credential.json，多个项目共用同一账号；
              密码请填坚果云『第三方应用密码』，不是登录密码
            </div>

            <NInput
              v-model:value="cred.server"
              placeholder="https://dav.jianguoyun.com/dav/"
              style="margin-bottom: 6px"
            />
            <NInput
              v-model:value="cred.account"
              placeholder="坚果云账号（注册邮箱）"
              style="margin-bottom: 6px"
            />
            <NInput
              v-model:value="cred.app_password"
              type="password"
              show-password-on="click"
              placeholder="应用密码（16 位）"
              style="margin-bottom: 6px"
            />
            <NInputGroup>
              <NInput
                v-model:value="remoteDir"
                placeholder="云端目录，如 apps/forge-studio"
              />
              <NButton type="primary" :loading="diagnosing" @click="runDiagnose">
                保存并测试连接
              </NButton>
            </NInputGroup>

            <div v-if="diagResult" class="diag">
              <div
                v-for="(s, i) in diagResult.steps"
                :key="i"
                class="diag-row"
                :class="{ bad: !s.ok }"
              >
                <span class="diag-icon">{{ s.ok ? "✓" : "✕" }}</span>
                <span class="diag-name">{{ s.name }}</span>
                <span class="diag-detail">{{ s.detail }}</span>
              </div>
            </div>
          </div>
        </NSpace>
      </NTabPane>
    </NTabs>

    <template #footer>
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
    </template>
  </NModal>
</template>

<style scoped>
.label {
  font-size: 12px;
  color: #6b7280;
  margin-bottom: 4px;
}
.hint {
  font-size: 11px;
  color: #9ca3af;
  margin-top: 2px;
}
.row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}
.divider {
  height: 1px;
  background: #eceff1;
  margin: 4px 0;
}
.diag {
  margin-top: 8px;
  padding: 8px 10px;
  background: #f8f9fa;
  border-radius: 6px;
}
.diag-row {
  display: flex;
  gap: 6px;
  font-size: 12px;
  line-height: 20px;
  color: #374151;
}
.diag-row.bad {
  color: #d03050;
}
.diag-icon {
  width: 12px;
  flex: none;
}
.diag-name {
  flex: none;
  min-width: 72px;
  color: #6b7280;
}
.diag-row.bad .diag-name {
  color: #d03050;
}
.diag-detail {
  flex: 1;
  word-break: break-all;
}
</style>
