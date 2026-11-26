import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Overview } from "./generated/types";

export const ipc = {
  getOverview: () => invoke<Overview>("get_overview"),
  refresh: () => invoke<void>("refresh"),
  showDashboard: () => invoke<void>("show_dashboard"),
  hidePanel: () => invoke<void>("hide_panel"),
  setPanelPinned: (pinned: boolean) =>
    invoke<void>("set_panel_pinned", { pinned }),
  onRefresh: (handler: () => void) => listen("demo://refresh", handler),
};
