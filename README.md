# QuotaPrism

**Every token counts.**

A beautiful, local-first AI quota and subscription monitor, designed to evolve into a personal AI control plane.

[简体中文](README.zh-CN.md) · [Roadmap](docs/ROADMAP.md) · [Architecture](docs/ARCHITECTURE.md) · [Agent instructions](AGENTS.md)

![Status](https://img.shields.io/badge/status-design%20%2F%20pre--alpha-blue) ![License](https://img.shields.io/badge/license-MIT-green) ![Stack](https://img.shields.io/badge/stack-Rust%20%2B%20Tauri%202-orange)

> **Project status:** specification and repository bootstrap. No runnable desktop application, live provider integrations or binaries have been implemented yet. All features below are planned unless explicitly marked complete.

## Why QuotaPrism?

AI coding subscriptions expose different quota windows, balances, request counts, credits and reset rules. Switching between dashboards is tedious, and a percentage alone does not tell you whether you can finish a task. QuotaPrism aims to make those constraints visible, understandable and actionable, without sending credentials or telemetry to a central service.

## First release: Prism Monitor

- Windows system tray and macOS menu bar, with a compact quota popover and a full dashboard.
- Multiple accounts, multiple quota windows, last successful refresh and transparent data provenance.
- Initial integration candidates: Kimi Code, MiniMax, GLM Coding, Codex, Claude Code and DeepSeek balance. Support depends on verified access and provider terms.
- Native credential storage, independent provider adapters, offline demo mode, light/dark themes, English and Simplified Chinese.
- Reset countdowns, low-quota alerts and configurable refresh intervals.

## Where we are going

| Stage | Purpose | Status |
| --- | --- | --- |
| Monitor | Quota, balance, history and subscription analytics | First priority |
| Switch | Safe client configuration switching, diff and rollback | Planned |
| Gateway | Authenticated local API for compatible clients | Planned |
| Router | Quota-aware, capability-aware routing with explicit budgets | Planned |
| Agent | Optional harness integration and eventually an agent runtime | Exploratory |

A monitor is not a gateway, and a subscription is not automatically an API entitlement. QuotaPrism will only relay traffic where the provider permits the credential, client and usage pattern. No public resale, account sharing, access-control bypass or automatic paid fallback by default.

## Development

This is a documentation-first repository. Start with [AGENTS.md](AGENTS.md), then [MVP specification](docs/MVP.md) and [implementation plan](docs/IMPLEMENTATION.md). Bootstrap Rust stable, Tauri 2, React, TypeScript and pnpm in milestone M0. Follow the official [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS. Build commands will be added once the workspace exists; there is intentionally no fake install command or placeholder CI build.

## Architecture & documentation

| Document | Contents |
| --- | --- |
| [Product vision](docs/PRODUCT.md) | Positioning, competitive research and scope |
| [MVP](docs/MVP.md) | User journeys, feature requirements and acceptance criteria |
| [Architecture](docs/ARCHITECTURE.md) | Core boundaries, domain model, IPC, data flows |
| [Provider SDK](docs/PROVIDERS.md) | Adapter contracts, evidence matrix and validation workflow |
| [Design system](docs/UI.md) | Tray, dashboard, visual tokens and platform details |
| [Evolution](docs/EVOLUTION.md) | Switch, Gateway, Router, Agent and their safety boundaries |
| [Security](docs/SECURITY-DESIGN.md) | Threat model, credentials and data lifecycle |
| [Implementation](docs/IMPLEMENTATION.md) | Agent-ready backlog, tests, release gates |
| [Roadmap](docs/ROADMAP.md) | Milestones and explicit go/no-go gates |
| [References](docs/REFERENCES.md) | Research links and verification limits |
| [ADRs](docs/adr/0001-foundation.md) | Initial architecture decisions |

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) and [SECURITY.md](SECURITY.md). Never submit API keys, OAuth tokens, cookies, or raw account responses. Provider integrations should include sanitized fixtures and evidence of supported access.

## License

MIT. Third-party code, brand assets and trademarks retain their respective rights; do not copy them without verifying the applicable license.
