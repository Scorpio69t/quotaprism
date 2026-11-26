# QuotaPrism · 额度棱镜

**每一份额度，尽在掌握。**

面向 AI 开发者的精致、本地优先、跨平台 AI 额度与订阅管理中心。未来逐步演进为个人 AI 控制中枢。

[English](README.md) · [开发路线](docs/ROADMAP.md) · [系统架构](docs/ARCHITECTURE.md) · [Agent 开发规范](AGENTS.md)

> 当前为设计与仓库初始化阶段，尚未实现桌面应用、真实厂商适配或安装包。以下均为计划功能。

## 为什么做这个项目

同时购买 Kimi、MiniMax、GLM、Codex、Claude Code 等套餐后，额度散落在不同平台。请求次数、Token、Credits、余额和滚动窗口还不能直接比较。QuotaPrism 希望用一个托盘解决查看问题，再用可靠的数据帮助用户规划资源。

## 第一阶段：Prism Monitor

- Windows 托盘、macOS 菜单栏；快捷面板与完整主窗口；中英文、深浅色和 Demo 模式。
- 多平台、多账户、多额度窗口；显示来源、更新时间、剩余量、重置倒计时以及错误状态。
- 优先验证 Kimi Code、MiniMax、GLM Coding、Codex、Claude Code、DeepSeek 余额；真实支持以接口与授权验证结果为准。
- 系统凭据库、本地 SQLite、自动刷新、手动刷新、低额度提醒；后续增加历史、利用率、续费管理与预测。

## 长期方向

| 模块 | 定位 |
| --- | --- |
| Prism Monitor | 额度与订阅可视化 |
| Prism Switch | 客户端配置切换、差异预览、备份与回滚 |
| Prism Gateway | 本地统一 API 与每客户端独立 Key |
| Prism Router | 额度感知、能力约束、可解释的自动调度 |
| Prism Agent | 优先集成现有 Harness，必要时实现独立执行层 |

**先做 Monitor，其他模块只预留边界，不提前实现。** 套餐权益不等于通用 API 权限；只对明确允许的访问方式开放网关功能。不提供公开转售、账号共享、规避访问限制或默认自动切换到付费 API。

## 开始开发

请首先阅读 [AGENTS.md](AGENTS.md)、[MVP 需求](docs/MVP.md) 和 [实施任务](docs/IMPLEMENTATION.md)。M0 建立 Rust Cargo Workspace + Tauri 2 + React + TypeScript + pnpm 工作区。按照 [Tauri 系统依赖说明](https://v2.tauri.app/start/prerequisites/) 安装环境；脚手架完成后补充经过验证的构建命令。

## 文档导航

- [产品规划](docs/PRODUCT.md)：竞品、产品边界、差异化。
- [MVP 详细需求](docs/MVP.md)：功能、流程、验收标准。
- [系统架构](docs/ARCHITECTURE.md)：模块、数据模型、接口、时序。
- [供应商适配规范](docs/PROVIDERS.md)：能力矩阵、接口验证与测试。
- [UI/UX 设计](docs/UI.md)：布局、视觉规范、系统托盘。
- [未来演进设计](docs/EVOLUTION.md)：配置切换、网关、路由、Harness。
- [安全设计](docs/SECURITY-DESIGN.md)：凭据、权限、数据与威胁模型。
- [开发执行计划](docs/IMPLEMENTATION.md)：任务依赖与验收。
- [路线图](docs/ROADMAP.md) · [参考资料](docs/REFERENCES.md) · [架构决策](docs/adr/0001-foundation.md)。

## 开源贡献

欢迎贡献 Provider、UI、测试及文档。请阅读 [贡献指南](CONTRIBUTING.md) 与 [安全报告](SECURITY.md)。严禁提交真实凭据、账户响应、日志或个人信息。

## 开源协议

MIT。第三方代码及图标需单独核查许可证与商标使用规则。
