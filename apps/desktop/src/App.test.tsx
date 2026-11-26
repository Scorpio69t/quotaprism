import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
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
              remaining: "0",
              limit: "100",
              remainingPercent: 0,
              resetAt: null,
              nextRecoveryAt: null,
            },
            {
              windowId: "unknown",
              label: "Unknown",
              unit: "token",
              period: { kind: "unknown" },
              remaining: null,
              limit: null,
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
  expect(screen.getByText("Amount unavailable")).toBeInTheDocument();
  expect(screen.getByText("USD 24.50")).toBeInTheDocument();
  expect(screen.getAllByRole("progressbar")).toHaveLength(1);
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
