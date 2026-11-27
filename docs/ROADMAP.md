# Roadmap

所有版本为目标而非已发布。具体时间以真实接口验证、开发资源与跨平台测试为准，不预先承诺日期。

| Version | 范围 | 阶段门 |
|---|---|---|
| v0.1.0 | Monitor：托盘、Demo、账户、缓存、刷新、真实 Provider、通知 | 至少 3 个 verified Provider，Windows/macOS 验收，安全与打包检查 |
| v0.2.0 | Monitor+：历史、订阅资产、预测、CLI、更多 Provider | 历史语义与保留策略、预测准确性说明、CLI 权限 |
| v0.3.0 | Switch：客户端配置切换、diff/backup/rollback | 客户端版本兼容矩阵、恢复与冲突测试 |
| v0.4.0 | Gateway：本地 Key、允许的上游 API、OpenAI compatible | entitlement matrix、loopback 安全、流式兼容性 |
| v0.5.0 | Router：手动→优先→额度感知，兼容组与预算 | 可解释决策、共享池、并发预留、熔断和无重复流式请求 |
| v0.6+ | Harness integration，必要时 Prism Agent | 先验证 SDK/插件集成；自研前批准独立 ADR、权限与审计设计 |
| v1.0 | 以成熟度而非功能数发布稳定版本 | 数据迁移、长期维护、可靠发行与用户反馈 |

## 当前任务
- [x] M0 QP-001 工程基座（Windows 构建和开发启动已验证；macOS 待验）
- [x] M0 QP-002 领域契约与离线 Demo（展示子集；完整 Provider 契约留待 M1）
- [ ] M0 QP-003 托盘垂直切片（Windows 基础交互已验证；macOS 和平台边界待验）
- [ ] M0 QP-004 UI polish（Demo 视图切换、排序、详情、语言、主题和空态已实现；完整状态及无障碍/DPI 验收待完成）
- [x] M0 QP-005 CI（Windows/macOS 矩阵首轮构建通过）
- [ ] M1 核心稳定性（QP-010 已实现；QP-011 凭据存储切片进行中）
- [ ] M2 真实 Provider 与发布

## 决策纪律
每阶段可独立交付；未来方向不是承诺。Linux、VS Code/tmux 集成、只读 MCP、自托管同步、声明式 Provider 插件按真实需求排序。任何网关或 Agent 工作不得挤占 MVP。长期任务详见 EVOLUTION；细粒度编号详见 IMPLEMENTATION。
