import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";
import { App } from "./App";
import { ipc } from "./ipc";
import type { Overview } from "./generated/types";

vi.mock("./ipc", () => ({
  ipc: {
    getOverview: vi.fn(),
    onRefresh: vi.fn(),
    refresh: vi.fn(),
    hidePanel: vi.fn(),
    showDashboard: vi.fn(),
    setPanelPinned: vi.fn(),
  },
}));
afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  delete document.documentElement.dataset.theme;
  document.documentElement.lang = "en";
});

test("shows unknown quotas separately from zero and displays balance without a percentage", async () => {
  const data: Overview = {
    demo: true,
    accounts: [
      {
        accountId: "demo-1",
        providerName: "Example",
        alias: "Test",
        snapshot: {
          observedAt: "2026-01-01T00:00:00Z",
          source: "demo",
          balances: [{ amount: "24.50", currency: "USD" }],
          windows: [
            {
              windowId: "zero",
              label: "Empty",
              unit: "request",
              period: { kind: "fixed" },
              used: "100",
              remaining: "0",
              limit: "100",
              usedPercent: 100,
              remainingPercent: 0,
              resetAt: null,
              nextRecoveryAt: null,
            },
            {
              windowId: "unknown",
              label: "Unknown",
              unit: "token",
              period: { kind: "unknown" },
              used: null,
              remaining: null,
              limit: null,
              usedPercent: null,
              remainingPercent: null,
              resetAt: null,
              nextRecoveryAt: null,
            },
          ],
        },
      },
    ],
  };
  vi.mocked(ipc.getOverview).mockResolvedValue(data);
  vi.mocked(ipc.onRefresh).mockResolvedValue(() => {});
  render(<App />);
  expect(await screen.findByText("0%")).toBeInTheDocument();
  expect(screen.getByText("Remaining amount unavailable")).toBeInTheDocument();
  expect(screen.getByText("USD 24.50")).toBeInTheDocument();
  expect(screen.getAllByRole("progressbar")).toHaveLength(1);
});

test("switches quota view without inventing used values, sorts known limits first and expands details", async () => {
  const data: Overview = {
    demo: true,
    accounts: [
      {
        accountId: "unknown",
        providerName: "Unknown",
        alias: "A",
        snapshot: {
          observedAt: "2026-01-01T00:00:00Z",
          source: "demo",
          balances: [{ amount: "24.50", currency: "USD" }],
          windows: [
            {
              windowId: "unknown",
              label: "Tokens",
              unit: "token",
              period: { kind: "unknown" },
              used: null,
              remaining: null,
              limit: null,
              usedPercent: null,
              remainingPercent: null,
              resetAt: null,
              nextRecoveryAt: null,
            },
          ],
        },
      },
      {
        accountId: "working",
        providerName: "Working",
        alias: "B",
        snapshot: {
          observedAt: "2026-01-01T00:00:00Z",
          source: "demo",
          balances: [],
          windows: [
            {
              windowId: "main",
              label: "Main",
              unit: "request",
              period: { kind: "rolling", seconds: 18000 },
              used: "32",
              remaining: "68",
              limit: "100",
              usedPercent: 32,
              remainingPercent: 68,
              resetAt: null,
              nextRecoveryAt: null,
            },
            {
              windowId: "weekly",
              label: "Weekly",
              unit: "percent",
              period: { kind: "fixed" },
              used: null,
              remaining: null,
              limit: null,
              usedPercent: null,
              remainingPercent: 45,
              resetAt: null,
              nextRecoveryAt: null,
            },
            {
              windowId: "large",
              label: "Large",
              unit: "token",
              period: { kind: "unknown" },
              used: "9007199254740993",
              remaining: null,
              limit: null,
              usedPercent: null,
              remainingPercent: null,
              resetAt: null,
              nextRecoveryAt: null,
            },
          ],
        },
      },
      {
        accountId: "alert",
        providerName: "Alert",
        alias: "C",
        snapshot: {
          observedAt: "2026-01-01T00:00:00Z",
          source: "demo",
          balances: [],
          windows: [
            {
              windowId: "credits",
              label: "Credits",
              unit: "credit",
              period: { kind: "fixed" },
              used: "88",
              remaining: "12",
              limit: "100",
              usedPercent: 88,
              remainingPercent: 12,
              resetAt: null,
              nextRecoveryAt: null,
            },
          ],
        },
      },
    ],
  };
  vi.mocked(ipc.getOverview).mockResolvedValue(data);
  vi.mocked(ipc.onRefresh).mockResolvedValue(() => {});
  render(<App />);
  await screen.findByText("Working");

  fireEvent.click(screen.getByRole("radio", { name: "Used" }));
  expect(
    screen.getByRole("progressbar", { name: "Main used" }),
  ).toHaveAttribute("aria-valuenow", "32");
  expect(screen.getByText("Low remaining")).toBeInTheDocument();
  expect(screen.getByText("32 / 100 requests used")).toBeInTheDocument();
  const workingCard = screen.getByText("Working").closest("article");
  if (!workingCard) throw new Error("Working account card not found");
  expect(
    within(workingCard).getByText("Used amount unavailable"),
  ).toBeInTheDocument();
  expect(screen.getByText("9007199254740993 tokens used")).toBeInTheDocument();
  expect(screen.queryByText("55%")).not.toBeInTheDocument();
  expect(screen.getByText("USD 24.50")).toBeInTheDocument();

  fireEvent.change(screen.getByRole("combobox", { name: "Sort accounts" }), {
    target: { value: "lowest" },
  });
  const cards = screen.getAllByRole("article");
  expect(within(cards[0]).getByText("Alert")).toBeInTheDocument();
  expect(within(cards[1]).getByText("Working")).toBeInTheDocument();
  expect(within(cards[2]).getByText("Unknown")).toBeInTheDocument();
  const expand = screen.getByRole("button", {
    name: "Show details for Working, B",
  });
  expect(expand).toHaveAttribute("aria-expanded", "false");
  fireEvent.click(expand);
  expect(
    screen.getByRole("button", { name: "Hide details for Working, B" }),
  ).toHaveAttribute("aria-expanded", "true");
  expect(screen.getByText("Rolling · 18000 seconds")).toBeVisible();
});

test("switches language and applies the selected theme", async () => {
  const data: Overview = {
    demo: true,
    accounts: [
      {
        accountId: "demo-1",
        providerName: "示例",
        alias: "测试",
        snapshot: {
          observedAt: "2026-01-01T00:00:00Z",
          source: "demo",
          balances: [],
          windows: [],
        },
      },
    ],
  };
  vi.mocked(ipc.getOverview).mockResolvedValue(data);
  vi.mocked(ipc.onRefresh).mockResolvedValue(() => {});
  render(<App />);
  await screen.findByText("示例");

  fireEvent.change(screen.getByRole("combobox", { name: "Language" }), {
    target: { value: "zh" },
  });
  expect(
    screen.getByRole("heading", { name: "更清晰地查看额度。" }),
  ).toBeInTheDocument();
  expect(screen.getByText("把所有限制集中在一处")).toBeInTheDocument();
  expect(document.documentElement.lang).toBe("zh-CN");

  fireEvent.change(screen.getByRole("combobox", { name: "主题" }), {
    target: { value: "light" },
  });
  expect(document.documentElement).toHaveAttribute("data-theme", "light");
});

test("shows an actionable empty demo state", async () => {
  vi.mocked(ipc.getOverview).mockResolvedValue({
    demo: true,
    accounts: [],
  });
  vi.mocked(ipc.onRefresh).mockResolvedValue(() => {});
  render(<App />);

  expect(await screen.findByText("No accounts in demo")).toBeInTheDocument();
  expect(
    screen.getByText("Reload the local demo to restore its sample accounts."),
  ).toBeInTheDocument();
  const callCount = vi.mocked(ipc.getOverview).mock.calls.length;
  fireEvent.click(screen.getByRole("button", { name: "Reload demo" }));
  await waitFor(() =>
    expect(ipc.getOverview).toHaveBeenCalledTimes(callCount + 1),
  );
});

test("reports IPC errors rather than inventing demo data", async () => {
  vi.mocked(ipc.getOverview).mockRejectedValue({
    message: "Backend unavailable",
  });
  vi.mocked(ipc.onRefresh).mockResolvedValue(() => {});
  render(<App />);
  await waitFor(() =>
    expect(screen.getByRole("alert")).toHaveTextContent("Backend unavailable"),
  );
  expect(screen.getByText("Awaiting data")).toBeInTheDocument();
  expect(screen.queryByText("No low limits reported")).not.toBeInTheDocument();
  expect(screen.queryByText("Simulated data")).not.toBeInTheDocument();
});

test("keeps the previous snapshot and reports a failed refresh", async () => {
  const data: Overview = {
    demo: true,
    accounts: [
      {
        accountId: "demo-1",
        providerName: "Example",
        alias: "Test",
        snapshot: {
          observedAt: "2026-01-01T00:00:00Z",
          source: "demo",
          windows: [],
          balances: [],
        },
      },
    ],
  };
  let onRefresh: (() => void) | undefined;
  vi.mocked(ipc.onRefresh).mockImplementation(async (handler) => {
    onRefresh = handler;
    return () => {};
  });
  vi.mocked(ipc.getOverview)
    .mockResolvedValueOnce(data)
    .mockRejectedValueOnce({ message: "Refresh failed" });
  vi.mocked(ipc.refresh).mockImplementation(async () => {
    onRefresh?.();
  });
  render(<App />);
  expect(await screen.findByText("Example")).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Refresh demo" }));
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Showing previous data",
  );
  expect(screen.getByText("Example")).toBeInTheDocument();
});
