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

CEF main-thread 生命周期与 Rust renderer 工作线程的集成已确定为共享 Web 宿主进程，见 `web-runtime-integration.md`。Chromium/CEF 为设置中显式安装的可选运行组件，基础包不包含运行库；方案、规格和任务已同步。

可选 Web 组件管理增量验证：

- 默认状态查询不创建组件目录、不联网；启动/扫描/应用不触发组件安装。Web 项目默认保留在列表，缺失组件时禁用应用按钮并提供设置入口。
- Rust 组件测试 10 项通过（其中官方归档测试默认不执行实际归档步骤）：校验失败保留已有 runtime、绝对/逃逸/重复路径、错误归档根、截断、链接、缺失资源、版本不匹配、同长度文件损坏、取消和废弃暂存清理、卸载不改用户项目。
- 使用已验证 SHA-256 的官方 Linux x86_64 CEF 135.0.17 minimal 归档单独运行实际 fixture，通过解包、完整资源清单/逐文件哈希、原子提交、重启检查和卸载；该测试安装到临时目录，没有为用户安装组件。
- Flutter 全量 50 项回归通过；随后新增进度/取消/重试与 Web 详情入口测试，界面测试 10 项全部通过。CI 指定范围 analyze 无问题；workspace clippy（所有 targets、warnings 为错误）通过。
- 状态来自真实组件服务，显示版本、下载来源、空间和错误，支持中英文。取消可中断等待 HTTP 响应及流读取；拒绝 HTTP 重定向/非成功状态，固定 URL、版本与 SHA-256，限制下载和解包大小。下载暂存不保留为长期缓存。

5.7 尚缺断网/磁盘不足等故障 fixture；5.8 尚缺运行中 browser/宿主停止确认；5.9 尚缺实际 renderer ABI/赋值恢复联动。Web 渲染宿主尚未实现，组件安装成功不能视作 Web 壁纸播放完成。未执行完整 CI quality、安装包验收或 push。


2026-10-11：共享 Chromium 宿主基础播放链路已接通（5.1–5.3）。

- 原生独立 CMake helper 构建通过，固定官方 CEF 135.0.17 SDK/runtime 和 IPC version 1。CEF 在宿主 main 初始化/关闭；helper 复用为 subprocess，沙箱保持启用。
- 真实 CEF fixture 通过：相对 CSS/JS/SVG、Canvas 动画、WebGL 首帧、独立双 browser、resize、关闭 browser 的 pause/resume；远程请求、file URL、symlink escape、下载、外部协议、入口 traversal 和超限 viewport 被拒绝并可诊断；退出后进程组无残留。
- Rust/Xvfb OpenGL fixture 通过：BGRA→RGBA 颜色及上下方向、非默认 color mask/blend/sRGB 状态恢复、首帧提交、取消 Web 候选保留媒体、实际 StopWeb 确认清除 Web 并保留另一媒体输出，最终 native acquisitions 回到基线。
- 服务扫描在组件和 helper 可用时将合法 Web DTO 转为 ready；安装状态变化刷新列表。应用再次检查 availability，卸载先阻止新请求、取消下载并等待 StopWeb 确认，再删除 owned runtime/cache/staging。清除事件移除运行中的显示映射，保存赋值不删除。
- 浏览器缓存使用组件目录内按宿主 PID 划分的目录，正常退出清理，卸载清除残留。空间统计不跟随 cache 内的链接；设置页定期刷新实际空间占用。
- 官方完整归档 fixture 再次通过（79.72 s），包括 LICENSE.txt 与 CREDITS.html 的解包、逐文件校验、重启检查及卸载。测试仍使用临时目录，不安装到用户数据目录。
- Rust workspace 回归：engine 37、library 40、GUI 26 项通过（未设置原生环境的 gated native fixture 在该轮提前返回，真实 Web fixture 单独运行）。发现并修复现有 ThumbnailService 发布结果早于 pending 计数更新的竞态，同时在发送请求失败时恢复计数。
- Flutter 全量 52 项通过；随后新增安装状态/清除事件保存赋值测试及设置空间刷新，controller + shell 27 项通过。CI 指定范围 analyze 与 clippy warnings-as-errors 通过；fmt、diff 检查及固定源校验通过。
- 实际 Linux release bundle 构建成功。bundle 包含 libexec/varpaper-web-host、runtime.json 和 wrapper 的 BSD 许可；主 GUI 无 libcef 动态依赖，helper 无 SDK 绝对 RPATH。bundle 不包含 libcef、CEF 快照/资源包/locales；data/icudtl.dat 是经哈希确认的 Flutter 自身 ICU 文件。
- 构建脚本在 build 阶段获取 SDK，不是用户 runtime 安装入口。Debian 最低 glibc 检查覆盖可选 helper。

同日后续加固与最终回归：

- 本地资源通过 owned directory descriptor 和逐级 `openat(O_NOFOLLOW)` 读取，拒绝路径交换/链接重定向；文件读取受打开时确认的长度限制。目录使用 `O_PATH`，不额外要求目录可列举权限。
- 所有项目资源包含 CSP 响应头，覆盖 Worker 的外部请求；CSP report 返回明确诊断。页面初始化前阻止 WebSocket/WebTransport/WebRTC 连接 API；真实 WebSocket、WebRTC 和 Worker fixture 均通过。JS 弹窗、文件选择器和设备/权限提示不会弹出桌面窗口，JS alert fixture 通过。
- 增加 Worker 后发现 CEF shutdown 仍可能保留 zygote。宿主使用独立进程组和 Linux subreaper，在关闭 CEF 后终止并回收剩余子进程。最终真实 CEF 回归退出码为 0，退出后进程组不存在。
- 暂停测试以 close 确认后的 frame sequence 判断无推进，避免 resize 的最后一帧在 pause 前到达造成测试竞态；恢复要求收到新的 paint。
- 实际 X11/OpenGL Web fixture 最终通过（2.55 s），包含同尺寸 texture 更新和停止 Web 时保留媒体输出。Flutter 全量最终 53 项通过，workspace clippy warnings-as-errors、fmt、shell syntax、diff 检查与 OpenSpec strict validation 通过。
- Linux release bundle 以最终 helper 重新构建成功；逐字节哈希确认 bundle 内 helper 与最新构建一致，固定清单/许可存在，CEF runtime 不在包中，GUI 无 libcef 依赖，helper 无 RPATH/RUNPATH。

仍未完成：Web 属性及音频接口、Web Audio 音量/声音 fixture 和 browser crash 验收；安装器断网/磁盘不足故障 fixture；完整卸载/reinstall/恢复综合验收；Scene 功能矩阵/音频可视化；Wayland/真实桌面、多包验收与完整 CI quality。5.8 的停止确认已接入，但其完整 consumer/retry/reinstall 验收尚未完成，未勾选。未执行 push。
