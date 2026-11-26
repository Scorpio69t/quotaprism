# QuotaPrism Agent development contract

## Mission
实现真正可用、轻量、精致、local-first 的 AI 额度监控软件。当前阶段只实现 Monitor。Switch / Gateway / Router / Agent 不创建运行时服务、不引入相关重型依赖、不实现推理转发。

## Read before editing
1. `docs/PRODUCT.md` 与 `docs/MVP.md`：确定产品边界。
2. `docs/ARCHITECTURE.md`、`docs/PROVIDERS.md`、`docs/SECURITY-DESIGN.md`：遵守核心契约。
3. `docs/UI.md`：按设计系统完成交互。
4. `docs/IMPLEMENTATION.md`、`docs/ROADMAP.md`：按依赖推进；`docs/EVOLUTION.md` 仅供未来架构参考。

## Execution rules
- 先检查现有工作区、Git 状态、工具链与任务记录；不覆盖用户改动。每次只完成一个清晰的里程碑或垂直切片。
- M0 先跑通真实 Tauri 窗口和托盘，不使用空 crate 假装功能完成。采用稳定版本、锁文件与最小权限。不要猜测 API 版本、参数或真实额度。
- Rust 核心不依赖 Tauri、React；前端只调用类型化 IPC。网络和凭据操作在 Rust 后端；Demo 与真实数据使用相同 DTO 并明确标识。
- 严禁使用真实 Key 作为 fixture、打印授权头、导出浏览器 cookies、绕过验证码/设备验证、将凭据发给第三方。只使用用户授权且供应商允许的接口；未知接口不得标记已支持。
- 配额单位独立计算。未知不等于零；失败保留旧快照并标记 stale；滚动窗口不能假定统一重置。所有时间存 UTC，UI 转换本地时间。
- 不为未来功能引入空壳网关、复杂插件框架或独立 daemon。需要新抽象时写 ADR，解释当前收益。
- 优先使用 `Result`、结构化错误、取消与超时；生产代码避免无依据的 `unwrap`。React 使用可访问组件、键盘操作与清晰状态，不做过度视觉装饰。
- 每次提交前执行适用的格式化、lint、单元测试、类型检查及构建。涉及平台托盘的行为标明实测 OS，无法实测的记入报告。不要伪造测试结果。
- 文档、能力矩阵与变更同步；功能未验证则显示 Experimental / Planned。未经用户要求不要发布 release 或把仓库改为公开。

## Deliverable format per task
说明实现内容、关键决策、修改文件、实际执行的验证及结果、未验证项、下一任务。对于变更的领域契约更新文档及测试。

## First task
完成 `docs/IMPLEMENTATION.md` 中 M0 的 QP-001 至 QP-003。先搭建真实 workspace、Tauri 窗口、托盘与 Demo 垂直切片的基础，再推进真实 Provider。M0 结束前不得实现代理端口、配置写入或 Agent。
