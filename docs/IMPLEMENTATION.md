# 开发实施清单（供 Coding Agent 执行）

状态：M0 离线 Demo 垂直切片已实现，Windows 基础交互已实测；macOS 与完整 M0 仍待验收。每项以验收为完成标准；不以创建文件数衡量。先完成 M0 垂直切片，再接真实接口。遇到证据缺失的 Provider 标记 blocked，转其他候选。

## M0：工程基座与 Demo 垂直切片
| ID | 工作 | 依赖 | 验收 |
|---|---|---|---|
| QP-001 | 初始化 Cargo Workspace、Tauri 2 + React TS + pnpm；锁定版本；基础 lint、格式化、目录规范 | 无 | 干净工作区安装依赖后能启动桌面窗口；记录实际命令与工具链 |
| QP-002 | Core 实体、错误、Provider trait、DTO/schema、Demo 适配器 | 001 | 单元测试包含多窗口、缺值、零、余额、滚动时间与单位；Demo 不发网络 |
| QP-003 | 托盘、窗口生命周期、快捷面板、主窗口，接入 Demo DTO | 002 | 真实窗口和托盘能开关；关闭/退出语义正确；无网络的 Demo 全流程；实际 OS 验证 |
| QP-004 | 设计 token、中英文、深浅色、空态/加载/过期/错误、键盘 | 003 | 视觉截图和交互测试；状态可见，不用颜色唯一表达 |
| QP-005 | CI 基线、lint/test/typecheck、Windows/macOS 构建矩阵 | 001 | PR 运行真实命令；不依赖真实 Key；未实测 OS 标记待验 |

M0 门槛：可运行的 Demo + 测试 + 文档真实构建命令。不要直接开发 6 家 Provider 或未来功能。

### QP-001–003 本次验证（2026-11-26）

- Windows build 26300、Rust 1.100.0 MSVC、Node 24.22.1、pnpm 10.18.3、WebView2 已安装。`pnpm install`、`pnpm tauri dev` 与 `pnpm tauri build --no-bundle` 成功，开发窗口和生产窗口均显示 Demo。
- `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --workspace` 通过（Rust 4 个测试）；`pnpm format:check`、`pnpm lint`、`pnpm typecheck`、`pnpm test` 通过（UI 3 个测试）；`pnpm build` 通过。
- Windows UI Automation 实测托盘图标、打开面板、Esc、固定/失焦、打开主窗口、主窗口关闭后进程驻留、第二实例唤醒主窗口、托盘菜单退出。已检查生产窗口截图；未进行离线网络断连抓包。
- 未在 macOS 编译或运行；未完成多显示器、各方向任务栏、DPI 矩阵、屏幕阅读器、安装/卸载及性能测量。定位单测不替代 OS 实测。QP-003 跨平台验收仍未完成。
- QP-004 尚未完成：目前为英文界面并跟随系统明暗主题，缺少中英文切换、手动主题、完整状态与无障碍矩阵；QP-005 CI 尚未实现。没有真实 Provider、账户、凭据、缓存或通知。

## M1：可靠核心
| ID | 工作 | 依赖 | 验收 |
|---|---|---|---|
| QP-010 | SQLite schema、事务迁移、账户及最近快照仓储 | 002 | 临时数据库测试，重启读取、迁移失败回滚 |
| QP-011 | OS keychain、secret ref 与 mock backend | 002 | 真实目标 OS smoke test；失败不落明文；添加/删除/更新恢复 |
| QP-012 | 账户应用用例、Provider registry、typed IPC | 010,011 | 多账户隔离；无返回 secret；删除一致性；设置持久化 |
| QP-013 | Scheduler、时钟注入、singleflight、并发与退避 | 012 | fake clock 测试 401/429/超时/取消；睡眠恢复限流 |
| QP-014 | 缓存与错误 UX、状态事件、通知去重 | 013 | 保留上次成功数据；通知不使用过期快照；事件乱序处理 |
| QP-015 | HTTP 共用层、安全与脱敏诊断 | 011,013 | redirect、body limit、TLS、敏感头日志测试 |

M1 门槛：Mock provider 与本地 fixture 能完成全部主要流程；失败不伪装成零；真实上游调用前先验安全边界。

## M2：真实 Provider 与 v0.1 发布
| ID | 工作 | 依赖 | 验收 |
|---|---|---|---|
| QP-020 | Kimi Code 调研与适配 | 015 | evidence + fixture + normalize + 授权 smoke test |
| QP-021 | MiniMax Token Plan | 015 | 同上，区分版本/区域 |
| QP-022 | GLM Coding | 015 | 同上，处理多个窗口 |
| QP-023 | DeepSeek 余额 | 015 | 币种与余额单独显示，不伪造百分比 |
| QP-024 | Codex | 015 | 官方可用数据、认证限制与状态验证 |
| QP-025 | Claude Code | 015 | 官方允许的访问方式，未知则 blocked |
| QP-026 | 账户设置、托盘定位、刷新通知完善 | 014 | MVP F01–F12 全部验收 |
| QP-027 | 发布安全检查、打包、安装/卸载、README 截图 | 026 | 至少 3 家 verified；两 OS 测试证据；产物校验；签名状态透明 |

顺序可在有证据情况下调整，真实账号 smoke 由用户本地授权执行。不得把社区接口推测当作通过。若未达到 3 个真实 Provider，只称 Demo/开发预览，不标记完整 MVP。

## M3：v0.2 历史与订阅
QP-030 历史保留策略和聚合（不跨单位、不重复共享池）；QP-031 历史视图和周期利用率；QP-032 续费与手动价格；QP-033 预测（最少样本、异常重置处理、不确定性）；QP-034 脱敏导出；QP-035 只读 CLI（权限明确，不输出 secret）。每项先写数据契约与测试。v0.2 不依赖网关。

## 验证策略
- Rust：`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --workspace`（具体 features 按 M0 确定）。
- UI：pnpm scripts 定义 typecheck/lint/test/build；组件测试覆盖状态和键盘；真实 Tauri IPC 与 WebView 不仅靠 mock 验收。
- 本地集成：mock HTTP server、fake time、temporary SQLite、fake keychain；真实 keychain 与系统托盘必须在目标 OS 手动/自动验收。
- 性能：冷启动/热弹窗 p50/p95、闲置内存/CPU、20 个账户刷新；以目标机器实测，记录 OS、版本、采样次数，禁止把架构推断当基准结果。
- CI：最小 contents:read；PR 不访问 release secrets；后续发布 workflow 单独审批/保护。签名和公证需要用户自己的证书与账户，不在本项目上传密钥。

## 每个 PR 的完成定义
功能可运行、测试通过或清楚披露失败、文档同步、错误可诊断、无秘密、对未知数据不造值、符合 MVP 范围、无无关重构。更新 roadmap 的真实状态。涉及依赖新增说明收益，涉及外部配置、网络权限或数据库不兼容变更必须写 ADR。

## 首次给本地 Agent 的提示词
```text
阅读 AGENTS.md、docs/MVP.md、docs/ARCHITECTURE.md、docs/UI.md、docs/IMPLEMENTATION.md。
先检查当前目录与 Git 状态、系统工具链，然后实施 M0 的 QP-001 至 QP-003。
要求真实 Tauri 2 + Rust + React TS 工程可运行，使用离线 Demo provider 实现从 Core DTO 到托盘面板的垂直切片。
按当前操作系统执行构建、测试和交互验收，未验证平台如实记录；不得实现网关、配置切换或 Agent。
交付代码、验证结果、可复制的开发命令、下一步任务，并更新相关文档。不要提交真实 API Key。
```
