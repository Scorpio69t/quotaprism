import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  AccountSummary,
  AddAccountInput,
  Overview,
  ProviderSummary,
  UpdateAccountInput,
} from "./generated/types";

export const ipc = {
  getOverview: () => invoke<Overview>("get_overview"),
  refresh: () => invoke<void>("refresh"),
  listProviders: () => invoke<ProviderSummary[]>("list_providers"),
  listAccounts: () => invoke<AccountSummary[]>("list_accounts"),
  addAccount: (input: AddAccountInput) =>
    invoke<AccountSummary>("add_account", { input }),
  updateAccount: (accountId: string, input: UpdateAccountInput) =>
    invoke<AccountSummary>("update_account", { accountId, input }),
  removeAccount: (accountId: string) =>
    invoke<boolean>("remove_account", { accountId }),
  showDashboard: () => invoke<void>("show_dashboard"),
  hidePanel: () => invoke<void>("hide_panel"),
  setPanelPinned: (pinned: boolean) =>
    invoke<void>("set_panel_pinned", { pinned }),
  onRefresh: (handler: () => void) => listen("demo://refresh", handler),
};
