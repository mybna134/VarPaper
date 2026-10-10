# 实现验证记录（进行中）

环境：Linux x86_64，Xvfb/Mesa software OpenGL，SDL dummy audio。原生测试显式启用 `VARPAPER_TEST_X11=1` 与 `VARPAPER_SCENE_LIBRARY`。这不是原生桌面、多输出、音频采集或安装包验收。

2026-10-10：

- 独立 CMake Scene 组件构建成功，只有六个 `vp_scene_*` ABI 符号导出；修复 QuickJS 与 libmpv/MuJS 符号冲突。
- Rust workspace 原生回归：engine 36、library 40、GUI bridge 14 项全部通过；包含 Image→Scene 首帧提交、候选失败保留旧源、GIF 红蓝帧循环、关联失败事件及反复释放。
- Mesa OpenGL 3.0：Scene 返回 OpenGL 3.3 能力错误且资源计数归零；媒体仍可应用、清除及退出。
- C ABI fixture 通过：首帧像素、线程拒绝、重复释放、初始化异常、二进制尺寸边界；生成 WAV 声音暂停/恢复/销毁及部分加载失败；未知/light 对象诊断；无限脚本退出预算；文本随注入时间改变、同一时间帧稳定。
- Flutter 全量回归测试 46 项通过。全项目 analyze 报告已有 preview_cache lint 与 Isar 生成代码 experimental_member_use，共 13 项；CI quality 指定范围 `flutter analyze lib/main.dart lib/src/l10n.dart lib/src/storage/settings_store.dart test` 无问题并通过。
- `cargo clippy --workspace --all-targets --offline -- -D warnings` 通过。
- 固定 9 个源依赖快照和 191 个适配源文件校验通过。

尚未完成：Scene 图形 fixture 对照、属性完整接口、共享 Pulse playback capture/FFT、Web browser 集成、五类型 GUI/持久化、三种安装包、真实桌面/soak、CI quality 全部门禁。已有通过结果不能替代这些验收；未执行 push。

CEF main-thread 生命周期与 Rust renderer 工作线程的集成决策见 `web-runtime-integration.md`，待确认后调整相关设计和任务。
