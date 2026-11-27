# ADR-0005：M1 账户应用服务与 typed IPC

- 状态：Accepted
- 日期：2026-11-27

## 背景

M1 需要在不引入真实 Provider、调度器或凭据表单的情况下，建立账户增删改、Provider registry、快照持久化和桌面 typed IPC 的稳定边界。

## 决策

- 新增独立 `prism-app` crate，集中账户应用用例；Tauri 窗口只依赖应用服务，不直接操作 SQL、密钥或 Provider 私有实现。
- `ProviderRegistry` 当前只静态注册离线 Demo Provider，并通过统一元数据与能力列表暴露；真实 Provider registry 内容留待后续验证。
- 账户写入只接受 provider ID 和 alias，不接受原始 secret；凭据引用单独由 `prism-security` 管理。删除前处理匹配凭据引用，避免明文或 OS 密钥残留。
- Tauri 呢新增 `list_providers`、`list_accounts`、`add_account`、`update_account`、`remove_account` 命令，输入和输出复用生成的 Rust/TypeScript Core DTO。
- Demo 刷新只更新快照和 provider 名称，不覆盖用户已保存的账户 alias；账户停用与删除通过应用用例统一。

## 后果与限制

- 当前账户服务是同步本地实现；真实异步刷新、时钟注入、并发、认证和 HTTP 仍为后续任务。
- SQLite 账户表升级到 v2，保留 v1 迁移且测试重启读取和 v1→v2；账户 SecretRef 暂为展示/删除能力，不代表已经支持认证表单。
- IPC 不返回原始 secret；真实账户 smoke、权限拒绝、密钥删除恢复和跨窗口事件乱序仍需后续验收。
