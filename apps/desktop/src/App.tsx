import { useCallback, useEffect, useState } from "react";
import {
  ArrowDownRight,
  ArrowUpRight,
  ChevronDown,
  CircleHelp,
  LayoutDashboard,
  Pin,
  RefreshCw,
  ShieldCheck,
  Sparkles,
} from "lucide-react";
import type { AccountSnapshot, Overview, QuotaWindow } from "./generated/types";
import { ipc } from "./ipc";

const panel = new URLSearchParams(window.location.search).has("panel");

type QuotaView = "remaining" | "used";
type AccountSort = "default" | "lowest";
type Language = "en" | "zh";
type ThemePreference = "system" | "light" | "dark";

type Copy = {
  remaining: string;
  used: string;
  remainingUnavailable: string;
  usedUnavailable: string;
  lowRemaining: string;
  nextRecovery: string;
  resets: string;
  balance: string;
  source: string;
  simulatedDemo: string;
  rolling: string;
  fixedPeriod: string;
  periodUnavailable: string;
  simulatedData: string;
  updated: string;
  emptyTitle: string;
  emptyDescription: string;
  reloadDemo: string;
};

const copy: Record<Language, Copy> = {
  en: {
    remaining: "Remaining",
    used: "Used",
    remainingUnavailable: "Remaining amount unavailable",
    usedUnavailable: "Used amount unavailable",
    lowRemaining: "Low remaining",
    nextRecovery: "Next recovery",
    resets: "Resets",
    balance: "Balance",
    source: "Source",
    simulatedDemo: "Simulated demo",
    rolling: "Rolling",
    fixedPeriod: "Fixed period",
    periodUnavailable: "Period unavailable",
    simulatedData: "Simulated data",
    updated: "Updated",
    emptyTitle: "No accounts in demo",
    emptyDescription: "Reload the local demo to restore its sample accounts.",
    reloadDemo: "Reload demo",
  },
  zh: {
    remaining: "剩余",
    used: "已用",
    remainingUnavailable: "剩余数量不可用",
    usedUnavailable: "已用数量不可用",
    lowRemaining: "剩余偏低",
    nextRecovery: "下次恢复",
    resets: "重置",
    balance: "余额",
    source: "来源",
    simulatedDemo: "模拟演示",
    rolling: "滚动周期",
    fixedPeriod: "固定周期",
    periodUnavailable: "周期不可用",
    simulatedData: "模拟数据",
    updated: "更新于",
    emptyTitle: "演示中没有账户",
    emptyDescription: "重新加载本地演示以恢复示例账户。",
    reloadDemo: "重新加载演示",
  },
};

function message(error: unknown): string {
  if (
    typeof error === "object" &&
    error !== null &&
    "message" in error &&
    typeof error.message === "string"
  )
    return error.message;
  return String(error);
}

function localTime(value: string, language: Language): string {
  return new Intl.DateTimeFormat(language === "zh" ? "zh-CN" : "en-US", {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(value));
}

function QuotaRow({
  quota,
  view,
  language,
  t,
}: {
  quota: QuotaWindow;
  view: QuotaView;
  language: Language;
  t: Copy;
}) {
  const percent =
    view === "remaining" ? quota.remainingPercent : quota.usedPercent;
  const amount = view === "remaining" ? quota.remaining : quota.used;
  const low = quota.remainingPercent !== null && quota.remainingPercent <= 15;
  const unit =
    quota.unit === "percent"
      ? "%"
      : `${quota.unit}${amount === "1" ? "" : "s"}`;
  const amountLabel =
    amount !== null
      ? language === "zh"
        ? `${view === "used" ? t.used : t.remaining} ${amount}${quota.limit !== null ? ` / ${quota.limit}` : ""}${unit === "%" ? "%" : ` ${unit}`}`
        : `${amount}${quota.limit !== null ? ` / ${quota.limit}` : ""} ${unit} ${view}`
      : percent === null
        ? view === "used"
          ? t.usedUnavailable
          : t.remainingUnavailable
        : view === "used"
          ? t.used
          : t.remaining;
  return (
    <div className="quota-row">
      <div className="quota-label">
        <span>{quota.label}</span>
        <strong className={low ? "warning-text" : ""}>
          {percent === null ? "—" : `${Number(percent.toFixed(1))}%`}
        </strong>
      </div>
      {percent !== null && (
        <div
          className="meter"
          role="progressbar"
          aria-label={`${quota.label} ${view}`}
          aria-valuenow={percent}
          aria-valuemin={0}
          aria-valuemax={100}
        >
          <span
            className={low ? "meter-fill low" : "meter-fill"}
            style={{ width: `${percent}%` }}
          />
        </div>
      )}
      <div className="quota-meta">
        <span>{amountLabel}</span>
        {low && <span className="low-label">{t.lowRemaining}</span>}
        {quota.nextRecoveryAt && (
          <span>
            {t.nextRecovery} {localTime(quota.nextRecoveryAt, language)}
          </span>
        )}
        {quota.resetAt && (
          <span>
            {t.resets} {localTime(quota.resetAt, language)}
          </span>
        )}
      </div>
    </div>
  );
}

function ProviderCard({
  account,
  view,
  language,
  t,
}: {
  account: AccountSnapshot;
  view: QuotaView;
  language: Language;
  t: Copy;
}) {
  const [expanded, setExpanded] = useState(false);
  const detailsId = `details-${account.accountId}`;
  return (
    <article className="provider-card">
      <div className="provider-header">
        <button
          className="provider-toggle"
          aria-expanded={expanded}
          aria-controls={detailsId}
          aria-label={`${expanded ? (language === "zh" ? "隐藏" : "Hide") : language === "zh" ? "显示" : "Show"} ${language === "zh" ? "详情：" : "details for"} ${account.providerName}, ${account.alias}`}
          onClick={() => setExpanded(!expanded)}
        >
          <span
            className={`provider-mark mark-${account.accountId.slice(-1)}`}
            aria-hidden="true"
          >
            {account.providerName.charAt(0)}
          </span>
          <span className="provider-title">
            <strong>{account.providerName}</strong>
            <span>{account.alias}</span>
          </span>
          <ChevronDown
            size={16}
            className={expanded ? "chevron expanded" : "chevron"}
            aria-hidden="true"
          />
        </button>
        <span className="source-tag">Demo</span>
      </div>
      <div className="quota-list">
        {account.snapshot.windows.map((quota) => (
          <QuotaRow
            key={quota.windowId}
            quota={quota}
            view={view}
            language={language}
            t={t}
          />
        ))}
      </div>
      {account.snapshot.balances.map((balance) => (
        <div className="balance" key={balance.currency}>
          <span>{t.balance}</span>
          <strong>
            {balance.currency} {balance.amount}
          </strong>
        </div>
      ))}
      <div className="provider-details" id={detailsId} hidden={!expanded}>
        <div>
          <span>{t.source}</span>
          <strong>{t.simulatedDemo}</strong>
        </div>
        {account.snapshot.windows.map((quota) => (
          <div key={quota.windowId}>
            <span>{quota.label}</span>
            <strong>
              {quota.period.kind === "rolling"
                ? `${t.rolling} · ${quota.period.seconds}${language === "zh" ? " 秒" : " seconds"}`
                : quota.period.kind === "fixed"
                  ? t.fixedPeriod
                  : t.periodUnavailable}
            </strong>
          </div>
        ))}
      </div>
      <div className="card-footer">
        <span>
          <span className="status-dot" /> {t.simulatedData}
        </span>
        <span>
          {t.updated} {localTime(account.snapshot.observedAt, language)}
        </span>
      </div>
    </article>
  );
}

export function App() {
  const [overview, setOverview] = useState<Overview | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [pinned, setPinned] = useState(false);
  const [loading, setLoading] = useState(true);
  const [quotaView, setQuotaView] = useState<QuotaView>("remaining");
  const [accountSort, setAccountSort] = useState<AccountSort>("default");
  const [language, setLanguage] = useState<Language>("en");
  const [theme, setTheme] = useState<ThemePreference>("system");
  const t = copy[language];

  useEffect(() => {
    const media = window.matchMedia?.("(prefers-color-scheme: light)");
    const applyTheme = () => {
      const resolved =
        theme === "system" ? (media?.matches ? "light" : "dark") : theme;
      document.documentElement.dataset.theme = resolved;
    };
    applyTheme();
    document.documentElement.lang = language === "zh" ? "zh-CN" : "en";
    media?.addEventListener?.("change", applyTheme);
    return () => media?.removeEventListener?.("change", applyTheme);
  }, [language, theme]);

  const load = useCallback(async () => {
    try {
      const data = await ipc.getOverview();
      setOverview(data);
      setError(null);
    } catch (reason) {
      setError(message(reason));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    let active = true;
    let unlisten: (() => void) | undefined;
    ipc
      .onRefresh(() => {
        void load();
      })
      .then((dispose) => {
        if (active) {
          unlisten = dispose;
          void load();
        } else dispose();
      })
      .catch((reason: unknown) => {
        setError(message(reason));
        setLoading(false);
      });
    return () => {
      active = false;
      unlisten?.();
    };
  }, [load]);

  useEffect(() => {
    if (!panel) return;
    const handleKey = (event: KeyboardEvent) => {
      if (event.key === "Escape")
        void ipc
          .hidePanel()
          .catch((reason: unknown) => setError(message(reason)));
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, []);

  const action = async (operation: () => Promise<void>) => {
    try {
      await operation();
    } catch (reason) {
      setError(message(reason));
    }
  };
  const lowCount = overview
    ? overview.accounts.filter((account) =>
        account.snapshot.windows.some(
          (item) =>
            item.remainingPercent !== null && item.remainingPercent <= 15,
        ),
      ).length
    : null;
  const accounts = overview?.accounts.map((account, index) => {
    const known = account.snapshot.windows
      .map((quota) => quota.remainingPercent)
      .filter(
        (percent): percent is number =>
          percent !== null &&
          Number.isFinite(percent) &&
          percent >= 0 &&
          percent <= 100,
      );
    return { account, index, lowest: known.length ? Math.min(...known) : null };
  });
  if (accountSort === "lowest") {
    accounts?.sort((a, b) => {
      if (a.lowest === null) return b.lowest === null ? a.index - b.index : 1;
      if (b.lowest === null) return -1;
      return a.lowest - b.lowest || a.index - b.index;
    });
  }
  return (
    <div className={panel ? "app panel" : "app dashboard"}>
      {!panel && (
        <aside className="sidebar">
          <div className="brand">
            <div className="brand-icon">◇</div>
            <div>
              <strong>QuotaPrism</strong>
              <small>MONITOR</small>
            </div>
          </div>
          <div className="nav-caption">WORKSPACE</div>
          <div className="nav-active">
            <LayoutDashboard size={17} />{" "}
            {language === "zh" ? "总览" : "Overview"}
          </div>
          <div className="sidebar-bottom">
            <ShieldCheck size={17} />
            <div>
              <strong>
                {language === "zh" ? "本地优先" : "Private by design"}
              </strong>
              <span>
                {language === "zh"
                  ? "数据始终保留在本机。"
                  : "Everything stays local."}
              </span>
            </div>
          </div>
        </aside>
      )}
      <main className="content">
        <header className="topbar">
          {panel ? (
            <div className="compact-brand">
              <span className="brand-icon">◇</span>
              <strong>QuotaPrism</strong>
            </div>
          ) : (
            <div className="breadcrumb">
              {language === "zh" ? "工作区" : "Workspace"} <span>/</span>{" "}
              <strong>{language === "zh" ? "总览" : "Overview"}</strong>
            </div>
          )}
          <div className="top-actions">
            <label className="appearance-control">
              <span className="sr-only">
                {language === "zh" ? "语言" : "Language"}
              </span>
              <select
                aria-label={language === "zh" ? "语言" : "Language"}
                value={language}
                onChange={(event) =>
                  setLanguage(event.target.value === "zh" ? "zh" : "en")
                }
              >
                <option value="en">English</option>
                <option value="zh">中文</option>
              </select>
            </label>
            <label className="appearance-control">
              <span className="sr-only">
                {language === "zh" ? "主题" : "Theme"}
              </span>
              <select
                aria-label={language === "zh" ? "主题" : "Theme"}
                value={theme}
                onChange={(event) =>
                  setTheme(
                    event.target.value === "light"
                      ? "light"
                      : event.target.value === "dark"
                        ? "dark"
                        : "system",
                  )
                }
              >
                <option value="system">
                  {language === "zh" ? "跟随系统" : "System"}
                </option>
                <option value="light">
                  {language === "zh" ? "浅色" : "Light"}
                </option>
                <option value="dark">
                  {language === "zh" ? "深色" : "Dark"}
                </option>
              </select>
            </label>
            <span className="demo-pill">
              <Sparkles size={12} />{" "}
              {language === "zh" ? "演示模式" : "DEMO MODE"}
            </span>
            {panel && (
              <button
                className={`icon-button ${pinned ? "selected" : ""}`}
                title={
                  pinned
                    ? language === "zh"
                      ? "取消固定面板"
                      : "Unpin panel"
                    : language === "zh"
                      ? "固定面板"
                      : "Pin panel"
                }
                aria-label={
                  pinned
                    ? language === "zh"
                      ? "取消固定面板"
                      : "Unpin panel"
                    : language === "zh"
                      ? "固定面板"
                      : "Pin panel"
                }
                aria-pressed={pinned}
                onClick={() =>
                  void action(async () => {
                    await ipc.setPanelPinned(!pinned);
                    setPinned(!pinned);
                  })
                }
              >
                <Pin size={16} />
              </button>
            )}
            <button
              className="icon-button"
              title={language === "zh" ? "刷新演示数据" : "Refresh demo"}
              aria-label={language === "zh" ? "刷新演示数据" : "Refresh demo"}
              onClick={() => void action(ipc.refresh)}
            >
              <RefreshCw size={16} />
            </button>
          </div>
        </header>
        <div className="content-scroll">
          <div className="page-heading">
            <div>
              <div className="eyebrow">
                <span className="status-dot" />{" "}
                {language === "zh" ? "本地总览" : "LOCAL OVERVIEW"}
              </div>
              <h1>
                {panel
                  ? language === "zh"
                    ? "你的额度"
                    : "Your quotas"
                  : language === "zh"
                    ? "更清晰地查看额度。"
                    : "A clearer view of your quotas."}
              </h1>
              <p>
                {panel
                  ? language === "zh"
                    ? "一目了然的使用情况。"
                    : "Your usage, at a glance."
                  : language === "zh"
                    ? "让下一个创意多一点把握。"
                    : "A little more clarity for your next big idea."}
              </p>
            </div>
            {!panel && (
              <div className="heading-art" aria-hidden="true">
                ◇
              </div>
            )}
          </div>
          <div className="demo-notice">
            <Sparkles size={16} />
            <span>
              {panel ? (
                <>
                  <strong>
                    {language === "zh" ? "演示数据。" : "Demo data."}
                  </strong>{" "}
                  {language === "zh"
                    ? "所有账户和值均为模拟数据。"
                    : "All accounts and values are simulated."}
                </>
              ) : (
                <>
                  <strong>
                    {language === "zh"
                      ? "你正在查看演示数据。"
                      : "You're exploring with demo data."}
                  </strong>{" "}
                  {language === "zh"
                    ? "这些账户和值均为模拟数据。"
                    : "These accounts and values are simulated."}
                </>
              )}
            </span>
          </div>
          <div className="section-heading">
            <div>
              <h2>
                {language === "zh" ? "账户" : "Accounts"}{" "}
                <span>{overview?.accounts.length ?? "—"}</span>
              </h2>
              <p>
                {language === "zh"
                  ? "把所有限制集中在一处"
                  : "All your limits, in one place"}
              </p>
            </div>
            <div className="section-status" role="status" aria-live="polite">
              <span
                className={
                  lowCount === null
                    ? "neutral-dot"
                    : lowCount > 0
                      ? "warning-dot"
                      : "status-dot"
                }
              />{" "}
              {lowCount === null
                ? language === "zh"
                  ? "等待数据"
                  : "Awaiting data"
                : lowCount > 0
                  ? language === "zh"
                    ? `${lowCount} 项需要关注`
                    : `${lowCount} needs attention`
                  : language === "zh"
                    ? "未报告低额度"
                    : "No low limits reported"}
            </div>
          </div>
          {error && (
            <div className="error-notice" role="alert">
              <CircleHelp size={17} />
              <span>
                {overview
                  ? language === "zh"
                    ? "正在显示之前的数据。"
                    : "Showing previous data. "
                  : ""}
                {language === "zh"
                  ? "无法加载演示数据："
                  : "Could not load demo: "}
                {error}
              </span>
              <button onClick={() => void load()}>
                {language === "zh" ? "重试" : "Try again"}
              </button>
            </div>
          )}
          {loading && !overview && (
            <p className="loading" role="status">
              {language === "zh"
                ? "正在加载演示账户…"
                : "Loading demo accounts…"}
            </p>
          )}
          {overview && overview.accounts.length === 0 && !loading && !error && (
            <div className="empty-state" role="status">
              <div className="empty-state-mark" aria-hidden="true">
                ◇
              </div>
              <strong>{t.emptyTitle}</strong>
              <p>{t.emptyDescription}</p>
              <button
                className="empty-state-action"
                onClick={() => void load()}
              >
                <RefreshCw size={14} />
                {t.reloadDemo}
              </button>
            </div>
          )}
          {overview && overview.accounts.length > 0 && (
            <div className="quota-controls">
              <fieldset className="quota-view">
                <legend className="sr-only">
                  {language === "zh" ? "额度视图" : "Quota view"}
                </legend>
                {(["remaining", "used"] as const).map((view) => (
                  <label key={view}>
                    <input
                      type="radio"
                      name="quota-view"
                      value={view}
                      checked={quotaView === view}
                      onChange={() => setQuotaView(view)}
                    />
                    <span>{view === "remaining" ? t.remaining : t.used}</span>
                  </label>
                ))}
              </fieldset>
              <label className="sort-control">
                <span className="sr-only">
                  {language === "zh" ? "账户排序" : "Sort accounts"}
                </span>
                <select
                  aria-label={language === "zh" ? "账户排序" : "Sort accounts"}
                  value={accountSort}
                  onChange={(event) => {
                    if (
                      event.target.value === "default" ||
                      event.target.value === "lowest"
                    ) {
                      setAccountSort(event.target.value);
                    } else {
                      setError(
                        language === "zh"
                          ? "不支持的排序选项"
                          : "Unsupported sort option",
                      );
                    }
                  }}
                >
                  <option value="default">
                    {language === "zh" ? "默认顺序" : "Default order"}
                  </option>
                  <option value="lowest">
                    {language === "zh" ? "最低剩余" : "Lowest remaining"}
                  </option>
                </select>
              </label>
            </div>
          )}
          {accounts && (
            <div className="cards">
              {accounts.map(({ account }) => (
                <ProviderCard
                  key={account.accountId}
                  account={account}
                  view={quotaView}
                  language={language}
                  t={t}
                />
              ))}
            </div>
          )}
          {!panel && (
            <div className="footnote">
              <ShieldCheck size={15} />{" "}
              {language === "zh"
                ? "此演示无需凭据或网络连接。"
                : "No credentials or network connection needed for this demo."}{" "}
              <span>
                {language === "zh"
                  ? "为安心使用而设计。"
                  : "Built for a little peace of mind."}
              </span>
            </div>
          )}
        </div>
        {panel && (
          <footer className="panel-footer">
            <span>
              <span className="status-dot" />{" "}
              {language === "zh" ? "离线演示" : "Offline demo"}
            </span>
            <button
              className="open-button"
              onClick={() => void action(ipc.showDashboard)}
            >
              {language === "zh" ? "打开仪表盘" : "Open dashboard"}{" "}
              <ArrowUpRight size={15} />
            </button>
          </footer>
        )}
      </main>
      {!panel && (
        <div className="corner-detail" aria-hidden="true">
          <ArrowDownRight size={14} />{" "}
          {language === "zh" ? "你的空间，你的数据" : "YOUR SPACE, YOUR DATA"}
        </div>
      )}
    </div>
  );
}
