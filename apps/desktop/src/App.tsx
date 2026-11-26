import { useCallback, useEffect, useState } from "react";
import {
  ArrowDownRight,
  ArrowUpRight,
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

function localTime(value: string): string {
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(new Date(value));
}

function QuotaRow({ quota }: { quota: QuotaWindow }) {
  const percent = quota.remainingPercent;
  const low = percent !== null && percent <= 15;
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
          aria-label={`${quota.label} remaining`}
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
        <span>
          {quota.remaining !== null && quota.limit !== null
            ? `${quota.remaining} / ${quota.limit} ${quota.unit}s remaining`
            : percent === null
              ? "Amount unavailable"
              : "Remaining"}
        </span>
        {quota.nextRecoveryAt && (
          <span>Next recovery {localTime(quota.nextRecoveryAt)}</span>
        )}
        {quota.resetAt && <span>Resets {localTime(quota.resetAt)}</span>}
      </div>
    </div>
  );
}

function ProviderCard({ account }: { account: AccountSnapshot }) {
  return (
    <article className="provider-card">
      <div className="provider-header">
        <div
          className={`provider-mark mark-${account.accountId.slice(-1)}`}
          aria-hidden="true"
        >
          {account.providerName.charAt(0)}
        </div>
        <div className="provider-title">
          <h3>{account.providerName}</h3>
          <span>{account.alias}</span>
        </div>
        <span className="source-tag">Demo</span>
      </div>
      <div className="quota-list">
        {account.snapshot.windows.map((quota) => (
          <QuotaRow key={quota.windowId} quota={quota} />
        ))}
      </div>
      {account.snapshot.balances.map((balance) => (
        <div className="balance" key={balance.currency}>
          <span>Balance</span>
          <strong>
            {balance.currency} {balance.amount}
          </strong>
        </div>
      ))}
      <div className="card-footer">
        <span>
          <span className="status-dot" /> Simulated data
        </span>
        <span>Updated {localTime(account.snapshot.observedAt)}</span>
      </div>
    </article>
  );
}

export function App() {
  const [overview, setOverview] = useState<Overview | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [pinned, setPinned] = useState(false);
  const [loading, setLoading] = useState(true);

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
            <LayoutDashboard size={17} /> Overview
          </div>
          <div className="sidebar-bottom">
            <ShieldCheck size={17} />
            <div>
              <strong>Private by design</strong>
              <span>Everything stays local.</span>
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
              Workspace <span>/</span> <strong>Overview</strong>
            </div>
          )}
          <div className="top-actions">
            <span className="demo-pill">
              <Sparkles size={12} /> DEMO MODE
            </span>
            {panel && (
              <button
                className={`icon-button ${pinned ? "selected" : ""}`}
                title={pinned ? "Unpin panel" : "Pin panel"}
                aria-label={pinned ? "Unpin panel" : "Pin panel"}
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
              title="Refresh demo"
              aria-label="Refresh demo"
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
                <span className="status-dot" /> LOCAL OVERVIEW
              </div>
              <h1>
                {panel ? "Your quotas" : "A clearer view of your quotas."}
              </h1>
              <p>
                {panel
                  ? "Your usage, at a glance."
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
              <strong>You're exploring with demo data.</strong> These accounts
              and values are simulated.
            </span>
          </div>
          <div className="section-heading">
            <div>
              <h2>
                Accounts <span>{overview?.accounts.length ?? "—"}</span>
              </h2>
              <p>All your limits, in one place</p>
            </div>
            <div className="section-status">
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
                ? "Awaiting data"
                : lowCount > 0
                  ? `${lowCount} needs attention`
                  : "No low limits reported"}
            </div>
          </div>
          {error && (
            <div className="error-notice" role="alert">
              <CircleHelp size={17} />
              <span>
                {overview ? "Showing previous data. " : ""}Could not load demo:{" "}
                {error}
              </span>
              <button onClick={() => void load()}>Try again</button>
            </div>
          )}
          {loading && !overview && (
            <p className="loading" role="status">
              Loading demo accounts…
            </p>
          )}
          {overview && (
            <div className="cards">
              {overview.accounts.map((account) => (
                <ProviderCard key={account.accountId} account={account} />
              ))}
            </div>
          )}
          {!panel && (
            <div className="footnote">
              <ShieldCheck size={15} /> No credentials or network connection
              needed for this demo.{" "}
              <span>Built for a little peace of mind.</span>
            </div>
          )}
        </div>
        {panel && (
          <footer className="panel-footer">
            <span>
              <span className="status-dot" /> Offline demo
            </span>
            <button
              className="open-button"
              onClick={() => void action(ipc.showDashboard)}
            >
              Open dashboard <ArrowUpRight size={15} />
            </button>
          </footer>
        )}
      </main>
      {!panel && (
        <div className="corner-detail" aria-hidden="true">
          <ArrowDownRight size={14} /> YOUR SPACE, YOUR DATA
        </div>
      )}
    </div>
  );
}
