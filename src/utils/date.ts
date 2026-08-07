// 事件模块共用的日期格式化工具。
// 以前 fmt / shortDate / 月份键逻辑散落在 EventCard / EventBoard 多处重复，
// 尤其「YYYY-MM 月份键」复制了两份 —— 改一处漏一处会让筛选静默失效。
// 现在统一收口在这里，任何显示规则变化只改这一个文件。

const pad = (n: number) => String(n).padStart(2, "0");

/** 完整时间 MM-DD HH:mm，用于悬停 tooltip。0/空 返回空串 */
export function fmtDateTime(ts: number): string {
  if (!ts) return "";
  const d = new Date(ts);
  return `${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

/** 短日期：今年不显示年份（MM-DD），跨年才带年份（YYYY-MM-DD） */
export function shortDate(ts: number): string {
  if (!ts) return "";
  const d = new Date(ts);
  const md = `${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
  return d.getFullYear() === new Date().getFullYear()
    ? md
    : `${d.getFullYear()}-${md}`;
}

/** 时间戳 → 月份键 "YYYY-MM"，用于归档月份分组 / 筛选（本地时区） */
export function monthKey(ts: number): string {
  const d = new Date(ts);
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}`;
}

/** 月份键 → 显示文案：2026-05 → "2026 年 05 月" */
export function monthLabel(key: string): string {
  const [y, mm] = key.split("-");
  return `${y} 年 ${mm} 月`;
}
