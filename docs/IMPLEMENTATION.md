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

### QP-004 后续切片（进行中）

- Demo 总览和托盘面板增加已用/剩余切换、默认/最低剩余排序、账户详情展开及低额度文字；排序按各账户已知窗口中最低的剩余百分比，未知排后，同值保持原顺序。视图、排序和展开状态仅为各窗口内存状态。
- DTO 分别传输已用和剩余数值及百分比；不以剩余推断已用，余额保持独立。契约和 UI 测试覆盖未知值、余额、排序和展开。
- 本切片回归通过：`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --workspace`（4 个测试）、`pnpm format:check`、`pnpm lint`、`pnpm typecheck`、`pnpm test`（4 个测试）和 `pnpm tauri build --no-bundle`。
- Windows 生产窗口实测了已用、未知值与展开详情。浏览器中使用模拟 IPC 验证了单选框方向键、排序和 Enter 展开；桌面锁屏期间未能验证原生窗口的键盘操作和实际排序顺序。QP-004 仍未完成，其他平台与无障碍矩阵仍待验。

### QP-004 当前切片（2026-11-27）

- Demo 总览和托盘面板增加英文/中文切换及跟随系统、浅色、深色主题选择；切换只更新当前窗口，不持久化或跨窗口同步。
- 语言切换覆盖页面标题、状态、额度控件、详情、错误、刷新及页脚文案；主题切换应用明暗 token 和主要卡片/面板表面。
- 组件测试由 4 项扩充至 5 项，并新增语言切换、主题属性和中文可访问名称回归。
- 进一步补充空账户状态及其重载操作，并为账户状态区提供实时状态语义；组件测试扩充至 6 项。

### QP-005 CI 基线（2026-11-27）

- 新增最小只读 CI，针对 `main` 推送和 Pull Request 验证 Windows 与 macOS。每个平台执行依赖安装、Rust 格式/Clippy/测试、前端格式/lint/typecheck/测试及 Tauri 无安装包构建。
- CI 不使用真实密钥、不构建签名安装包、不执行网络抓包或发布上传；实际 macOS runner 结果仍需首次 CI 运行确认。
- GitHub Actions run `45918419853` 已通过 Windows 与 macOS：Rust 格式/Clippy/测试、前端格式/lint/typecheck/测试及 Tauri 无安装包构建均成功。
- 生成 DTO 契约测试会归一化 CRLF/LF 换行，避免 Windows checkout 与生成文件的换行差异造成假失败。
- 仓库启用 LF Git 行尾约束及 Prettier `endOfLine: lf`，避免 Windows checkout 导致格式检查误报。

## M1：可靠核心
| ID | 工作 | 依赖 | 验收 |
|---|---|---|---|
| QP-010 | SQLite schema、事务迁移、账户及最近快照仓储 | 002 | 临时数据库测试，重启读取、迁移失败回滚（已实现） |
| QP-011 | OS keychain、secret ref 与 mock backend | 002 | 真实目标 OS smoke test；失败不落明文；添加/删除/更新恢复（进行中） |
| QP-012 | 账户应用用例、Provider registry、typed IPC | 010,011 | 多账户隔离；无返回 secret；删除一致性；设置持久化（进行中） |
| QP-013 | Scheduler、时钟注入、singleflight、并发与退避 | 012 | fake clock 测试 401/429/超时/取消；睡眠恢复限流（进行中） |
| QP-014 | 缓存与错误 UX、状态事件、通知去重 | 013 | 保留上次成功数据；通知不使用过期快照；事件乱序处理 |
| QP-015 | HTTP 共用层、安全与脱敏诊断 | 011,013 | redirect、body limit、TLS、敏感头日志测试 |

M1 门槛：Mock provider 与本地 fixture 能完成全部主要流程；失败不伪装成零；真实上游调用前先验安全边界。

### QP-010 SQLite 存储切片（2026-11-27）

- 新增独立 `prism-storage` crate：文件/内存数据库、外键、WAL、事务迁移和账户与最近快照仓储。
- 实现账户快照统一事务写入、最近快照读取、账户列表、账户删除及迁移失败回滚；不保存凭据或原始 HTTP 响应。
- 当前临时数据库测试覆盖重启读取、账户删除、迁移失败回滚；目标 OS 文件权限、WAL恢复和并发仍待验收。

### QP-011 凭据存储切片（2026-11-27）

- 新增独立 `prism-security` crate：受格式校验的 `SecretRef`、脱敏 `SecretValue`、`SecretStore` trait、内存 mock 和 `keyring` OS 后端。
- 移除真实凭据的持久化风险：SQLite 不保存 secret，DTO 和日志不得携带原始值；mock 测试覆盖添加、更新、读取、删除、缺失及 Debug 脱敏。
- 当前未执行真实 OS Keychain/Credential Manager 写入 smoke test；不得把 mock 成功当作目标 OS 验证。

### QP-012 账户应用与 typed IPC 切片（2026-11-27）

- 新增独立 `prism-app` crate：Demo Provider registry、账户增删改、快照刷新持久化和账户隔离用例。
- SQLite v2 增加 provider、enabled、credentialRef 和创建时间；v1→v2 使用事务迁移并保持用户账户别名。
- Tauri 增加 provider/账户查询、添加、更新、删除 typed command；前端生成 DTO 包含统一账户与输入契约，不返回原始 secret。
- 已覆盖账户更新隔离、删除一致性、Demo 刷新别名保留和关联 secret 删除；设置持久化、真实刷新、认证表单和跨窗口事件仍未实现。

### QP-013 调度器切片（2026-11-27）

- 新增独立 `prism-scheduler` crate：可注入时钟、每账户 cold-down、同账户 singleflight、跨账户并发和指数退避。
- 取消、认证、权限、429、网络、超时、schema、不支持、存储和未知错误具有独立状态语义；取消不会执行刷新动作。
- Fake clock 测试覆盖 401 无重试、429 Retry-After、超时指数退避、取消和同/不同账户并发。
- 当前为同步内存调度核心，不建立后台 Tokio 任务、持久化 refresh_state、睡眠恢复或真实 HTTP 请求。

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
