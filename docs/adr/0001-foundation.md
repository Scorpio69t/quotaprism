# ADR-0001：Local-first、模块化单体与分阶段演进

- 状态：Accepted for design baseline（实现待验证）
- 日期：2026-11-26

## 背景
需要轻量桌面托盘应用，支持多个异构 AI Plan，未来可能具备配置切换、模型代理、调度和 Agent，但第一阶段必须控制范围。

## 决策
采用 Rust/Tauri 2 + React TS；核心领域/应用服务与 UI、持久化、适配器分离。SQLite 保存非秘密状态，OS keychain 保存凭据。默认单进程、静态 Provider 注册。显式区分 quota_read 与 inference entitlement，未知默认拒绝推理。未来模块依赖领域读接口，不反向依赖桌面。网关、Agent 需单独 ADR 才能开始。

## 替代方案
Electron：UI 生态成熟，但常驻体积与开销需权衡；Wails：Go 熟悉、开发效率高，但本项目希望深入 Rust，Tauri 生态具备托盘与定位支持；Qt：原生能力强，前端开发方式和跨平台分发成本不同；Swift：macOS 原生体验好但 Windows 另写。当前是定性选型，不声称实测性能优胜。

## 后果
跨 OS WebView 和 keychain 行为有差异，需要测试；Rust 异步和 UI IPC 复杂度高于单语言脚本。未来抽象有可能调整，但不为了可选功能提前拆 daemon 或造动态插件 ABI。若真实托盘体验或安装成本不达标，用实际数据评估替代方案。
