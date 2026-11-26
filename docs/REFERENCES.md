# 调研参考与验证边界

调研基线：2026-11-26。参考公开 README、官方文档和项目说明。未逐项安装竞品、验证全部套餐、检查所有许可证细节或使用真实账户测试。不得复制竞品代码/图标，除非核查该文件与依赖的许可证及署名要求。链接可能变化，以当前官方版本为准。

| 来源 | 参考价值 |
|---|---|
| [CodexBar](https://github.com/steipete/CodexBar) | 额度菜单栏、适配器 |
| [OpenUsage](https://github.com/robinebers/openusage) | 紧凑多平台额度界面 |
| [Coding Quota Bar](https://github.com/hyizhou/coding-quota-bar) | 国内平台适配思路 |
| [TokenBar](https://github.com/lwh0630/TokenBar) | 多供应商与跨设备方向 |
| [QuotaBar](https://github.com/QuotaBar/QuotaBar) | macOS UI 细节 |
| [UsageDeck](https://github.com/lamchun1110/UsageDeck) | 用量与历史分析 |
| [CC Switch README](https://github.com/farion1231/cc-switch/blob/main/README.md) | 客户端配置、路由、聚合 |
| [CC Switch switching manual](https://github.com/farion1231/cc-switch/blob/main/docs/user-manual/en/2-providers/2.2-switch.md) | Direct 与 Routing 行为、重启边界 |
| [CLIProxyAPI](https://github.com/router-for-me/CLIProxyAPI) | 本地兼容协议及代理形态，仅参考合法授权用例 |
| [LiteLLM router](https://docs.litellm.ai/docs/router/learn/concepts) | 模型组、候选选择、回退 |
| [LiteLLM reliability](https://docs.litellm.ai/docs/router/learn/reliability) | 有界重试、冷却 |
| [DeepSeek Harness](https://github.com/deepseek-ai/deepseek-harness) | 插件化 Agent Harness |
| [DeepSeek Harness core](https://github.com/deepseek-ai/deepseek-harness/blob/master/packages/core/README.md) | Agent contract、事件、工具注册 |
| [DASH](https://github.com/songqikong/dash) | DeepSeek Harness 的 TUI 插件 |
| [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) | 真实系统依赖 |
| [Tauri system tray](https://v2.tauri.app/learn/system-tray/) | 托盘 API |
| [Tauri positioner](https://v2.tauri.app/plugin/positioner/) | 托盘窗口定位 |

## 后续实证记录模板
Provider / 套餐 / 地区 / 文档 URL / 认证类别 / quota 语义 / inference entitlement（allow/deny/unknown）/ 验证日期 / 证据与限制 / 脱敏 fixture 路径 / 验收人。缺失项目标 unknown。版本号、路径、参数以实际文档验证为准。
