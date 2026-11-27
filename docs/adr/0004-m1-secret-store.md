# ADR-0004：M1 凭据引用与 OS 密钥后端

- 状态：Accepted
- 日期：2026-11-27

## 背景

M1 需要在真实 Provider 接入前建立凭据边界：应用可保存、读取、更新和删除认证材料，但不得把原始 secret 放入 SQLite、IPC DTO、日志或调试结构。

## 决策

- 新增独立 `prism-security` crate，定义 `SecretStore` 端口与 `set`、`get`、`delete` 操作。
- `SecretRef` 是受格式校验的不可见凭据引用，不是凭据本体；仅允许 ASCII 字母数字及 `. _ : -`，长度不超过 128。
- `SecretValue` 对 Debug 和格式化输出进行脱敏；空值拒绝。调用方仍需限制其作用域并避免复制到持久化结构。
- 提供内存 `MockSecretStore` 供确定性测试与离线开发；提供基于 `keyring` 的 `KeyringSecretStore`，由 Windows Credential Manager、macOS Keychain 等目标平台后端保存 secret。
- 后端错误区分未找到、无效引用、空 secret 和 OS store 失败，不返回第三方错误中的意外凭据内容。

## 后果与限制

- 当前切片只建立安全端口、mock 后端和 OS 后端构造，不把密钥接入账户、Provider 或 IPC。
- 单元测试验证引用格式、脱敏、更新、删除和未找到语义，不访问用户真实钥匙串。
- 真实目标 OS 写入、更新、删除、拒绝权限、迁移恢复与重新登录仍需人工或自动 smoke test；不得把 mock 成功当作 OS 后端成功。
