# Proposal

## Why

用户报告当前 coverage 约为 64%，同时多轮架构迁移留下了空实现、并存封装与过时注释等清理候选。需要结合调用关系和覆盖率明细删除确实无用的代码，并为有效但缺少验证的分支补充测试，降低维护成本。

## What Changes

- 建立 Rust、Flutter 和 Codecov 的覆盖率口径与提交基线，分别记录覆盖行、总行及环境依赖测试的实际执行情况。
- 核查并清理 `WallpaperSession::render_frame` 空实现、未使用的 Wayland layer-shell 封装/输出辅助方法、残留字段和过时迁移注释；保留生产中使用的 `OutputManager` 及所有权资源。
- 审核 FRB 旧路径应用入口与新版 source 入口的重复调度和校验，优先合并内部逻辑，保留现有契约要求的入口和锁内可用性复查。
- 核查 scanner、项目路径处理、Web 组件状态和 Flutter controller/store 中未覆盖的分支；删除有不可达证据的分支，为仍有效的错误恢复、取消和清理分支补测试。
- 以相同统计范围比较清理前后覆盖率和未覆盖行数量；不增加生产代码排除规则，不重新启用用户已移除的两项 CI 检查。

## Capabilities

### New Capabilities

无。本变更是保持产品行为的内部重构与测试改进，设置 `skip_specs: true`。

### Modified Capabilities

无。不调整五类型壁纸、数据格式、错误处理或桌面生命周期的既有需求。若清理需要改变有效公共行为，另行提出范围调整。

## Impact

- Rust：`crates/wayvid-engine/src/{engine,wayland}`、`crates/wayvid-library/src/{scanner,project}.rs`、`rust/src/{bridge,engine,web_component}.rs` 及相关测试；具体文件以删除证据清单为准。
- Flutter：`lib/main.dart`、`lib/src/storage/settings_store.dart`、`test/` 与测试 fake；生成代码仅在接口确需重生成时由生成器更新。
- 验证：现有 quality job、Rust llvm-cov、Flutter lcov、`codecov.yml` 现有统计范围。无新增运行时依赖，无持久化迁移，无自动提交或推送。
- 与 `add-all-wallpaper-types`、`refactor-flutter-owned-state-and-engine-boundary` 的重叠处按当前用户批准的设计处理；既有 specs 仍包含旧 daemon/IPC 等历史要求，本次不据此恢复已移除架构，也不将本次清理当作历史规格同步。
