# 安全与隐私设计

## 信任边界
用户与 OS credential store 是凭据授权边界；Tauri WebView 视为低信任展示层；Rust 后端受限访问已注册 provider；上游响应与本地导入内容均不可信。未来 gateway 客户端、模型输出和 Agent 工具进一步独立隔离。默认仅本机，不设账号中心，不收集遥测。

| 风险 | 控制 | 验收 |
|---|---|---|
| API Key 泄露 | Windows Credential Manager / macOS Keychain；keyring 后端明确；secret ref 存 SQLite；内存短生命周期 | DB/日志/导出无 secret；凭据库不可用显示失败 |
| WebView 注入 | 本地静态资源、严格 CSP、禁远程脚本、IPC capability 白名单、输入校验 | 禁任意 shell/HTTP/file 读取，前端不读取已存 Key |
| 供应商恶意响应 | TLS、固定 host、限制响应大小、schema validation、无跨域授权头 | 畸形/重定向/超时测试 |
| 账户信息泄露 | 仅持久化规范化额度，禁止 raw HTTP、脱敏诊断 | 导出 fixture 扫描 |
| 依赖/发布链 | 锁文件、依赖扫描、最小权限 CI、签名更新和可校验制品 | 无 secret in PR、发布前 SBOM/校验文件 |
| 本地数据丢失 | 事务迁移、备份、操作恢复、删除确认 | 迁移/删除失败恢复测试 |
| 本机代理滥用（未来） | loopback、独立 key、Host/Origin/鉴权、预算与限流 | 未授权调用和 DNS rebinding 测试 |
| Agent 注入（未来） | 最小工具权限、沙箱、人工审批、工作区范围、审计 | 注入、越权读写、重复副作用测试 |

## 凭据生命周期
添加时只在安全输入与后端短期存在；不把内容缓存到前端 store/localStorage；响应不回传完整 Key。日志默认不输出 HTTP headers 或 body；避免 secret 类型 Debug 输出；崩溃报告默认本地，不自动上传。Keyring backend 不可用时禁止 fallback 明文，允许不持久化的临时会话（后续评估）。OS keychain 防止普通文件读取，但不能声称抵抗恶意同权限进程或已失陷设备。

更新时先存新 credential ref、验证、再切换 DB 引用并回收旧 ref；失败保留旧配置。删除账户先确认、标记并处理相关历史/secret 回收，失败可重试。共享凭据不得被错误删除；MVP 可每账户独立引用。退出不会自动删除已授权存储。设置提供清除历史、删除账户、导出脱敏数据；删除前明确影响。SQLite 并非默认全库加密，额度与别名仍可能敏感，使用 OS app data 权限，不宣称全盘加密。

## 供应商规则
区分数据可读、推理 API 使用权、客户端/地域/用途限制。拒绝未授权代理与访问控制规避。对实验性接口记录来源、授权和维护风险，需用户 opt-in，随时关闭。测试真实账户不得上传到 CI，不得使用生产 Key 作为示例。UI 要明确显示数据来源、时间及误差。用户自行管理订阅，软件不自动付款。
