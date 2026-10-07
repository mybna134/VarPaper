# Tasks

## 1. 启动窗口控制

- [x] 1.1 在 `lib/main.dart` 提取可直接测试的启动窗口函数，等待 readiness/configuration 完成后显式等待显示操作，取消 async VoidCallback 用法；使用 method-channel 测试验证 true 分支不调用 show/focus、false 分支按顺序显示聚焦，且保留窗口尺寸等配置。
- [x] 1.2 删除 `linux/runner/my_application.cc` 无条件显示顶层窗口的首帧回调及连接，保留 Flutter view realization 和插件初始化，补充说明由 Dart 控制顶层窗口显示的注释；构建 Linux bundle，并以已保存 true/false 设置分别启动，确认隐藏启动不闪现且正常启动可显示。
- [x] 1.3 扩展 `test/settings_store_test.dart`，验证 startMinimized 为 true 和 false 的保存、关闭数据库后重开读取，以及无保存值时默认 false；运行该测试并确认重启设置契约成立。

## 2. 托盘恢复

- [x] 2.1 在 `lib/main.dart` 共用显式恢复窗口函数，依次清除 skipTaskbar、show、focus，将托盘 show/library/settings 和隐藏窗口图标点击接入；扩展 `test/shell_widgets_test.dart` 验证每个入口的方法顺序及页面导航，保留可见窗口点击隐藏和关闭/退出测试，运行相关测试通过。

## 3. 集成验证

- [x] 3.1 在具备托盘支持的 Wayland 和 X11 桌面检查完整进程重启：勾选后启动全程无主窗口闪现且不抢焦点、后台引擎和配置的壁纸恢复正常、托盘各入口恢复窗口和任务栏状态、取消勾选后正常显示；在实施记录中写明环境和观察结果，未覆盖环境明确记录为未验证。
- [x] 3.2 运行 `.github/workflows/ci.yml` 的 quality 检查：Rust fmt、workspace clippy（warnings 为错误）、llvm-cov workspace 测试、列出的打包脚本 bash -n 和 Flatpak JSON 校验、Flutter clean/pub get/analyze/test --coverage；如需 push，按 AGENTS.md 确认 quality 全部检查（含适用的覆盖率上传步骤）通过后再 push，并记录命令与结果。
