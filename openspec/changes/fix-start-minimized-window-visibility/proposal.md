# Proposal

## Why

勾选“启动时最小化”后，Linux 原生 runner 仍在 Flutter 首帧到达时无条件显示主窗口，覆盖 Dart 根据已保存设置作出的隐藏启动决定。需要消除这条独立显示路径，让启动设置真正控制初始窗口可见性。

## What Changes

- Linux 主窗口默认保持隐藏，由 Flutter 读取设置并显式决定是否显示；隐藏启动不得闪现主窗口或抢占焦点。
- 保留正常启动的窗口显示、尺寸设置及聚焦行为，隐藏启动仍初始化应用、托盘和壁纸引擎。
- 托盘恢复窗口时恢复正常任务栏状态并显示、聚焦，覆盖所有现有托盘打开入口。
- 增加设置持久化、启动窗口决策和托盘恢复回归验证，并在真实 Linux 桌面验证原生首帧行为。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `gui-integration`: 补充启动窗口可见性要求，明确“启动时最小化”代表隐藏主窗口并继续后台初始化，以及用户从托盘恢复窗口的行为。

## Impact

- `linux/runner/my_application.cc` 的首帧显示路径。
- `lib/main.dart` 的启动窗口配置与托盘打开窗口入口。
- `test/settings_store_test.dart`、`test/shell_widgets_test.dart` 及必要的启动窗口测试。
- 不增加依赖或设置字段，不改变设置数据库结构；不扩展命令行参数或自动启动管理。
- 现有 `gui-integration` 对 YAML 设置和 `--minimized` 自动启动参数的描述已与当前 Isar 存储和自动启动实现不一致，本次只补充窗口行为，不顺带重写这些历史规格。
