# Provider SDK 与接入规范

## 证据驱动的能力矩阵
所有以下候选均未使用真实账号进行验证；路径来自公开资料或此前社区调研，仅为研究线索，不能直接当作已稳定可用的 API。M1/M2 按照官方文档、平台规则、账户授权和脱敏实测逐个确认。

| Provider | 目标能力 | 调研线索 | 优先级 | 状态 |
|---|---|---|---|---|
| Demo | 模拟窗口及余额 | 本地内存，无网络或凭据；名称和数值为虚构示例 | M0 | 实现（非真实 Provider） |
| Kimi Code | 套餐窗口/剩余/恢复 | coding usages；具体 host/auth 待验证 | P0 | Planned |
| MiniMax | Token Plan quota | token_plan/remains；区分区域和计划 | P0 | Planned |
| GLM Coding | 周期和 MCP 配额 | monitor usage quota/limit；认证待验证 | P0 | Planned |
| DeepSeek | API 余额 | 官方余额 API，查阅最新文档 | P0 | Planned |
| Codex | 官方 CLI/支持的用量来源 | 依当前 CLI 版本、官方可用接口 | P0 | Planned |
| Claude Code | 官方用量来源 | 仅允许的认证/客户端方式 | P0 | Planned |
| MiMo / 百炼 / 火山 / Step / OpenCode Go | Plan、credits 等 | 独立调研 | P1 | Planned |
| OpenRouter / 硅基流动 / New API | 余额、消费 | 官方管理 API | P1 | Planned |
| Cursor / Gemini / Copilot / Qoder | 可提供的额度指标 | 官方文档和授权 | P2 | Planned |
| Custom | 用户声明的 JSON 读取契约 | 后续 allowlist、schema 与 SSRF 防护 | P2 | Planned |

不假定每个账户具有 5h / weekly 两个窗口。Kimi 等套餐规则可能随版本/地区/新老计划变化。实际响应和官方语义优先。

当前 Demo 使用与桌面窗口相同的 Rust DTO，并以字符串传输计数和金额。同步 trait 是离线阶段的最小接口，真实适配前需要补充异步、取消、超时、认证与验证能力，见 [ADR-0002](adr/0002-m0-demo-contract.md)。

## 认证与权限
首选正式 quota/balance API、明确支持的 OAuth、官方 CLI 暴露的只读数据；其次是平台允许的实验性接口（用户主动启用）。不自动抓浏览器 cookies、不模拟身份、不绕过验证码、反自动化或访问限制。官方 CLI 数据来源也需检查其许可和文件权限，不默认扫描所有本地凭据。每个适配器的能力状态：planned / verified / experimental / unsupported / disabled；推理 capability 独立记录，默认 denied。没有证据即 unknown/unsupported，不猜测。

## 每个适配器交付清单
1. `docs` 中记录官方链接、认证类型、地区、套餐、数据含义、更新时间、请求频率、验证日期及可选实验开关。记录支持与禁止的用途，不记录账户身份。
2. Provider module：metadata、capabilities、validate、fetch_quota、normalize、错误映射。禁止直接写数据库、发通知或生成 UI。
3. HTTP：固定 HTTPS 目的地、TLS 校验、超时、取消、响应体大小限制、受控 redirect（不跨 origin 发送授权头），不为查询而消耗生成额度。避免无边界重试。
4. Fixture：使用手造或完全脱敏的 JSON，包含正常、零、无分母、缺字段、多个窗口、滚动窗口、共享池、429、401、非 JSON 及 schema 演变。
5. Golden test：断言 normalized snapshot、时间、精度与来源标签；不在 CI 请求真实上游。真实 smoke test 手动 opt-in，避免输出响应或 Key。
6. 文档和 UI 标记实际支持的认证/账户/区域；验收通过才改为 verified。

## 数据映射示例（纯模拟）
```json
{
  "accountId": "00000000-0000-4000-8000-000000000001",
  "observedAt": "2026-11-26T08:00:00Z",
  "sourceKind": "demo",
  "windows": [
    {"windowId":"rolling-main","label":"5 h","unit":"request","period":{"kind":"rolling","seconds":18000},"used":32,"limit":100,"remaining":68,"resetAt":null,"nextRecoveryAt":"2026-11-26T10:00:00Z"},
    {"windowId":"weekly-extra","label":"Weekly","unit":"percent","reportedRemainingPercent":45,"period":{"kind":"fixed"},"resetAt":null}
  ],
  "balances": []
}
```
按真实 provider 语义区别 resetAt 与 nextRecoveryAt；不能用下一次恢复推断整窗清空。数值异常（负数、分母为零、越界、NaN）须拒绝或降级 unknown 并给 warning，不静默截断掩盖错误。真实 token 整数用无损 DTO（必要时 decimal string 防 JS 精度丢失）。

## 维护机制
Provider 接口失效后仅标记对应账户 stale/error，不影响其他厂商。用 capability registry 控制展示；维护测试 fixture、变更日志和最后验证日期。默认不加载第三方动态代码；未来可先采用经过审核的声明式适配，动态插件须单独设计签名、权限和沙箱。
