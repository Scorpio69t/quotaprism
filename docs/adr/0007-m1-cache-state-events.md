# ADR-0007：M1 缓存、刷新状态与通知去重

- 状态：Accepted
- 日期：2026-11-27

## 背景

M1 需要在刷新失败时保留最近成功数据，并向桌面窗口提供稳定的刷新状态、乱序 revision 和通知去重语义。

## 决策

- 应用服务保留内存最近成功 Overview；失败只更新 RefreshStatus，不覆盖成功快照。
- RefreshStatus 包含 in-flight、尝试/成功时间、Retry-After、脱敏错误、stale 标记和递增 revision。
- Tauri 提供 `get_refresh_status` 并发出 `refresh://state` 与 `accounts://changed`；前端只消费事件和重查 DTO，不把事件当作数据源。
- 通知去重以账户、窗口、周期指纹和阈值为复合键；同键仅首次发送，同阈值新周期可再次发送，不用过期快照触发通知。
- 缓存和通知状态均不持久化原始 Provider 响应或 secret；通知持久化仍留待后续存储任务。

## 后果与限制

- 当前缓存只保存在应用进程内，重启后由 SQLite 最近快照恢复为后续任务；失败不会伪造百分比或零值。
- 乱序保护以 revision 提供基础，但跨进程持久化事件和完整通知状态仍待后续实现。
