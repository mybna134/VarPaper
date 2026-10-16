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

2026-10-11 后续：共享音频、Web 回调与组件完整生命周期（4.2、5.4–5.9）。当前 22/48。

- 新共享音频服务动态加载 libpulse，订阅并重新发现默认输出的 playback monitor，禁止回退到麦克风；48 kHz stereo PCM 经 2048 点 Hann FFT 转为每声道 64 个对数频带及 RMS level。过期/断线采集返回零，并自动恢复；最后一个 consumer 释放后线程、stream/context 同步回收。
- 已知左右不同频率 PCM、零输入、非有限样本、分块、断线、monitor 切换、过期输入和共享 consumer 生命周期 fixture 通过。实际 PipeWire 1.6.9/Pulse 兼容接口测试读取到真正的 PCM sequence，6 项音频测试通过。
- Web 初始及修改属性回调保留 boolean/number/string 类型，声明范围、颜色与 choices 校验通过；128 项音频回调按左/右顺序交付，零输入和 pause/resume 后最新属性 fixture 通过。Scene 仅接入共享采集输入，完整 shader/effect/script 音频响应尚待 4.3。
- CEF AudioHandler 捕获 HTML media 和 Web Audio，SDL 输出统一控制 mute/volume，限定音频队列为 250 ms。真实 fixture 验证 Web Audio 与生成 WAV 的增益、静音、暂停计数冻结及恢复。
- 真实 CEF fixture 杀死独立 browser 所属实际 renderer 子进程，确认 crash 诊断、另一 browser 继续出帧。宿主退出后逐个检查原有所有 descendant PID 不存在，并确认进程组不存在。
- 原生 X11/Scene/Web 10 项回归通过（45.22 s），包含属性类型/范围、纹理方向、GL 状态、失败替换与 StopWeb 保留另一媒体输出。未开启原生 fixture 的 workspace 测试提前返回不能替代该真实回归。
- 安装组件 17 项测试通过（0.37 s）：HTTPS-only、不跟随重定向、断网/截断、声明超预算、ENOSPC 写入、等待响应时取消与重试，以及原子交换失败保留旧 runtime。官方归档实际解包/逐文件校验/重启检测/卸载 fixture 再次通过（52.41 s），均为临时目录。
- 新运行中卸载综合 fixture 在明确隔离的 Xvfb/X11 会话通过（10.17 s；XDG_SESSION_TYPE=x11 且移除 WAYLAND_DISPLAY）。验证缺失组件不创建目录，错误版本/校验和/缺少 ICU 资源禁用播放；helper 清单 ABI 不符在扫描/应用前拒绝，实际协议不符返回关联失败且不发提交/清除事件；正确 helper 可继续播放。
- 同一综合 fixture 通过 StopWeb 后实际 browser/host/descendant PID 与 audio consumer 归零。临时 owned-root 链接使删除保护检查失败，保留运行库和用户资产，移除链接后重试成功；空间为零、卸载后重启不下载，重新安装并重启后成功首帧，再次卸载保持用户项目/保存数据不变。其他类型输出保留由上述独立 X11 混合 fixture 验证。
- 固定 CEF 版本/架构/URL/SHA 不变，helper IPC 升至 2。扫描和应用校验有界清单与 executable permissions；实际响应再次校验 ABI，错误时关闭 helper。CMake 从同一 runtime.json 编译 IPC 常量并复制开发清单，旧 helper 不会被静默启用。
- 新 Flutter 保存赋值恢复测试覆盖 missing/damaged/version/helper ABI/uninstalled，验证恢复失败保留源/赋值、不下载，重启安全重试，显式修复后成功恢复。修复真实 BridgeError 被显示成 Instance of BridgeError 的问题。Flutter 全量 58 项通过，CI 指定范围 analyze 无问题。
- Workspace 最新音频轮回归 engine 43/library 40/GUI 31 项通过；后续新增协议与综合 fixture 后的最终回归待下方记录。workspace clippy warnings-as-errors、fmt 与 diff 检查通过。

仍待完成：Scene 图形/脚本/时钟/完整音频响应矩阵，描述式属性编辑与版本化 Isar 持久化，混合 Wayland/真实桌面与 soak，三种安装包及完整 CI quality。本轮通过的项目 manifest/保存数据保留测试不能替代 6.5 的新字段迁移/属性恢复。未执行 push。

同日：原生构建及 CI 接入（7.1、7.5），当前 24/48。

- 新 scripts/build-scene.sh 验证库存并构建 Scene。scripts/build-linux.sh 构建两种原生组件，Flutter CMake 安装 Scene C ABI 动态库、九个固定依赖许可、Scene 许可及来源/依赖清单，继续仅安装 Web helper/清单/许可而不打包 CEF。直接 Flutter CMake 配置也成功，默认编译 Scene；Cargo 开发及 CI 原生 fixture 通过显式动态库路径使用相同 ABI。
- 无旧产物的 build/native-scene-clean-validation 独立构建通过；其 C ABI fixture 在 Xvfb 通过（5.75 s）。默认 CMake 配置验证源与依赖快照，未使用参考 checkout。
- 最新 Linux release bundle 构建通过；哈希确认库/helper 等于最新产物，来源清单等于固定源码清单，九种依赖许可齐全。Scene/helper 无 RPATH/RUNPATH 和参考项目绝对路径；GUI/Scene 的 ldd 无未解析库且无 libcef 链接；CEF libraries/snapshots/paks/locales 未打包。
- 新 scripts/check-native.sh 接入 CI quality，增加必需开发库/Xvfb，保留既有 fmt、clippy、llvm-cov、脚本/manifest、Flutter 与 coverage 上传检查。脚本与 YAML 校验通过。
- 完整执行该脚本通过：191 个固定源与 9 个依赖快照验证、两个原生组件构建、C ABI/真实 CEF 回归，以及显式启用全部原生 fixture 的 Rust engine 45/library 40/GUI 32 项（分别 44.55/1.65/7.99 s）。运行中卸载测试限制在 XDG_SESSION_TYPE=x11 且无 WAYLAND_DISPLAY 的独立 Xvfb 会话，防止继承桌面后端。

这些结果完成构建/CI 接入，不代表 7.2–7.4 的三种安装包验收或 8.4 的全部 quality（尤其 coverage 上传）已经执行。未执行 push。

Scene 兼容诊断（3.5），当前 25/48：GLSL parse/link 失败改为向 C ABI 传递具体诊断，pass 的 Shader 使用 unique_ptr，临时编译 shader 使用 RAII，构造失败清理 program，驱动无日志的失败也必定拒绝。同步更新固定源码库存的三个适配文件与说明。真实 C ABI fixture 通过（7.20 s），包含未知/light 对象、缺失 shader、十次无效 vertex、无效 fragment、实际 Mesa driver 编译失败和 uniform link 失败；检查本 fixture 的 GPU 对象名字空间及 host 资源回到基线，然后有效脚本文字仍正常出帧。X11 失败替换/另一个媒体输出继续运行 fixture 通过（1.07 s），clippy warnings-as-errors 通过。完整图形矩阵及最终 bundle/native 回归仍需在后续改动后更新，未执行 push。

Scene 纹理/资源（3.1），当前 26/48：生成 TEXV/TEXB、模型、材质及 shader fixture，RGBA8/R8/RG8/DXT1/DXT3/DXT5 的 raw 与 LZ4 共 12 种情况均实际渲染红色双图层，确认纹理只读取一次。截断、像素长度不足、非法尺寸及缺失纹理拒绝并检查 native/GPU 回收；缺失纹理错误改为正确 ENOENT 和实际 materials/*.tex 路径，不再显示 Success。同步 TextureCache 的固定源 hash。Rust fixture 使用相同生成资产，实际渲染 loose、完整 PKGV、PKGV→shared texture 三种资源来源，逐字节比较完整帧并拒绝缺少 shared texture；X11 RGB visual 的 alpha 不作为颜色判断。完整 CI 原生脚本通过：C ABI、真实 CEF、engine 46/library 40/GUI 32（49.69/1.45/9.49 s），共 118 项；仍未完成图层/effect/粒子画面对照。Scene 源码更新后最终 bundle 需要重建，未执行 push。

Scene 音频接入（4.3 部分实现，仍为 26/48）：Scene ABI 升为 2，增加逐帧独立左右频谱/双声道 RMS setter；Rust renderer 传完整共享采集快照，16/32/64 shader uniforms 绑定独立声道，RMS uniform 为 VarPaper 扩展。SceneScript registerAudioBuffers 返回稳定 Float32Array left/right/average，逐帧更新，限制每 Scene 128 次注册。修复 QuickJS magic callback 类型及 engine 实例登记，实例 ID 限制在其有符号 16-bit 存储范围并复用已释放 ID；错误模块求值返回实际 JS 异常。

原生 C ABI fixture 验证频谱像素、脚本文字变化、静音独立、缺少新输入归零、非法分辨率、注册预算、双 Scene 隔离与资源回收；完整原生脚本通过（ABI 10.67 s、CEF、Rust 118 项，此时尚未增加下述新 Rust 测试）。新增私有音频 runner 在独立 PipeWire graph 中仅加载 null sink，WirePlumber 只启用 policy（无硬件 monitor），播放左 375 Hz/右 6000 Hz；真实 libpulse→FFT→SceneScript 画面测试通过（0.56 s），验证壁纸静音仍有新 PCM/频谱和文字响应，audio/EGL 计数回基线。临时配置/音频文件和私有服务退出；未修改用户音频服务。粒子/emitter、多 pass 效果和采集不可用的 GUI 诊断尚需验收，4.3 未勾选。

音频接入最终回归：频谱/level NaN/Infinity/负数与超范围输入清洗、非恒定频带 downsample、ABI 2 断言均通过。最新完整原生脚本通过：ABI 10.55 s、真实 CEF、engine 47/library 40/GUI 32（50.89/1.36/8.23 s），共 119 项；私有真实音频测试另行显式运行通过。fmt、clippy warnings-as-errors、OpenSpec strict、191 源/9 依赖 hash 和 diff 检查通过。Linux release bundle 重建通过，Scene 库哈希/ABI 2 新 symbol/来源清单匹配，GUI/Scene ldd 无缺失/CEF 链接，基础 bundle 不包含 CEF runtime。发现原有全局 tests/ 忽略规则会遗漏 C ABI fixture，已仅为两个 Scene fixture 源文件增加例外，保持测试缓存忽略。当前仍 26/48；未执行 push 或宣称完成全部 quality/安装包/桌面验收。


## 2026-10-15：描述式应用与赋值持久化（6.4、6.5）

当前 Rust Scene 范围按本 change 顶部修订执行；上文 C++/C ABI 记录均为历史证据。

- FRB 新增版本化 WallpaperSourceDto、describe/apply source 与属性更新接口，并重生成 Rust/Dart bindings。旧路径接口保留；失败事件携带 output/path/code/message，候选失败不覆盖当前赋值。
- Web 初始属性覆盖在候选首帧之前传入。更新校验声明名、类型、颜色、范围、choices 和 64 KiB 上限；更新要求源匹配且无候选替换。Scene 继续使用声明初始属性，GUI 属性编辑及 Scene 覆盖仍未交付。
- Isar 新增可选 sourceVersion/wallpaperType/projectSourceJson/propertyOverridesJson，保留 sourcePath/sourceId。新赋值保存描述，旧记录按路径恢复；替换成旧媒体时清除描述字段。失败恢复和失败候选均保留原记录。
- `/tmp/varpaper-source-persistence-tests.log`：Flutter 62 项通过。settings_store tests 实际覆盖冻结旧 Isar schema 到新 schema 的迁移、Scene/Web 描述及 Web 覆盖重开数据库后保留、旧媒体替换清除新字段；controller_restore tests 覆盖描述式恢复和失败保留。
- `/tmp/varpaper-typed-source-native.log`：Rust Scene CPU 24、GL 12、GL 3.0 拒绝 1，以及 engine 84/library 40/bridge 34 项通过。原生日志不能替代完整安装包/物理桌面验收。
- `/tmp/varpaper-source-wayland.log`：嵌套 niri Wayland 4 项通过，包含 Scene/media 失败替换、清除及重启。
- 本轮交接后 `git diff --check` 与 `cargo fmt --all -- --check` 通过。未运行完整 CI quality，未 push。

6.5 完成后按用户要求暂停，等待提交。4.5、6.7 的文档修订已开始，仍未勾选，需在后续 Task 中核对并收尾。


### 2026-10-15：按用户要求只使用新版赋值格式

本节取代上述 6.5 的旧格式兼容策略及迁移验收要求。

- 新版记录使用独立 `WallpaperSourceAssignment` Isar 集合；旧集合不读取、不迁移，旧分配需要重新应用。
- 保存接口仅接收完整 `WallpaperSourceDto`，路径和 ID 从描述写入；null 描述仅用于“所有显示器”分配下单输出显式清除。
- 恢复只调用描述式应用接口；缺失/不支持版本或不完整描述报错，不回退路径恢复。路径应用 API 本身仍可用于显式应用，不参与旧数据恢复。
- 删除旧 schema 迁移 fixture；现有媒体/项目恢复、持久化和 fake fixtures 改用新版描述。Isar 代码重新生成成功。文档/design/spec/tasks 同步新规则。
- 本次修改未运行测试，先前 62 项结果仅对应修改前。完整 CI quality 未执行，未提交或 push。按用户要求本 Task 结束后暂停。


## 2026-10-15：使用文档收尾（6.7）

- 中英 README、GUI/多屏/配置/安装页及 Workshop 说明统一五种实际类型、全部/指定输出操作和失败保留行为；明确仅使用新版赋值，旧记录不读取或迁移。
- 对照 `lib/main.dart` 的 Library 双击、详情输出按钮、恢复与保存路径，以及 `renderer.rs` 的类型分派和属性限制。用户入口未承诺 GUI 属性编辑或 Scene 覆盖。
- 对照 `web.rs`、`native/web/README.md` 的安装依赖、暂停关闭/恢复重建、系统播放监听和零输入行为；系统音频回调仅宣称 Web 支持。Scene 音频可视化、本地时钟和完整脚本仍明确 deferred。
- 对照 `scene/clock.rs` 的暂停基线和 250 ms delta 上限说明动画恢复；支持范围链接到 Rust fixture 对照表，不把历史 C++ 验证当作当前实现。
- 修正安装页旧仓库地址、三种包格式/构建产物说明和无效 Scene 文档锚点。
- 本 Task 仅修改文档和任务记录，未运行测试或重新执行历史 fixture。完成后暂停等待用户提交；未提交或 push。


## 2026-10-15：脚本、属性、音频与时间接口文档（4.5）

- `docs/src/dev/rust-scene.md` 新增接口/fixture 对照、从已有 fixture 提取的参数、运行命令及前置条件。逐项核对 `script.rs`、`script_helpers.js`、`script_timers.rs`、`runtime.rs`、`runtime_tests.rs`、`clock.rs`、`properties.rs`、`audio.rs` 和 `native/web/test_host.py` 源码。
- 明确 Scene hooks/层字段、动画秒/定时器毫秒、暂停恢复、预算、内部快照/typed arrays/uniforms；Web typed property/128 项左右音频顺序、零输入、采集恢复和 consumer 生命周期。
- 示例参数对应现有断言：Scene opacity 0.2/0.8 → 0.21/0.81；builder gain 0.25 与 opacity 0.8 → 0.2；Web speed 0.4→0.8、caption initial→updated、64×0.25 左/64×0.75 右与零输入；共享 PCM 375/6000 Hz。
- 实时 Scene 系统音频、可注入本地时钟、完整脚本/API 矩阵、GUI 属性编辑与 Scene 覆盖仍 deferred。采集诊断只声明 AudioSnapshot 字段，不虚构 GUI 状态。运行时属性更新不自动持久化。
- 修正 Web README 的属性持久化旧表述，并连接 Workshop 兼容说明。此 Task 只修改文档，未运行测试；源码参数核对不代表重新执行 fixture。完成后暂停等待用户提交，未提交或 push。


## 2026-10-15：公共控制路由修正（6.3 部分，未勾选）

- WallpaperSession 分离 manual/power 暂停原因，恢复一个原因不解除另一个。Wayland/X11 在渲染前同步电池策略，关闭 pause_on_battery 后解除电池原因；X11 初次渲染前读取供电状态。
- Flutter 托盘切换仅选择 canApplyWallpaper 的项目，不再选中 unsupported 或组件不可用项目。设置增加中英说明：循环适用于 Video/GIF，图片持续显示，Scene/Web 由各自内容控制动画。
- `cargo check --workspace --all-targets --locked --offline` 在实现改动后通过。现有 Flutter tray/controller 回归 18 项通过；此范围不包含混合类型的 battery/native renderer 验收。
- 尚未完成 6.3 要求的混合类型/电池策略回归及两后端实际暂停验证，任务继续未勾选。暂停等待用户提交；未提交或 push。


## 2026-10-15：公共控制路由完成（6.3）

- `WallpaperSession` 将手动暂停和电池暂停作为独立原因；解除一个原因不会恢复仍被另一个原因暂停的会话。Wayland/X11 在每帧前同步电池策略，配置关闭后解除电池暂停；Video、Image、GIF、Scene、Web 共用该会话控制路径。
- Flutter `cycleWallpaper` 只选择 `canApplyWallpaper` 的项目；托盘切换不会选择不支持类型或缺少 Web 组件的项目。设置页增加循环模式适用范围说明：Video/GIF 使用循环设置，Image 持续显示，Scene/Web 使用自身动画。
- `test/tray_controls_test.dart` 和 `test/controller_logic_test.dart`：19 项通过，覆盖托盘切换、暂停/恢复、静音/音量、Web 状态和不可用项目跳过。
- Rust session control fixture：5 种类型共用手动/电池暂停独立性、配置转发和音量控制，1 项通过。
- X11 混合控制 fixture：Video、Rust Scene、Web 会话的暂停、恢复、局部输出控制、音量/静音/循环配置及 Web 停止后的其他输出保留，1 项通过。
- `scripts/check-wayland.sh`：嵌套 niri Wayland 4 项通过，包含媒体/Scene 暂停恢复、失败候选保留、清除和重启。
- `cargo check --workspace --all-targets --locked --offline`、Clippy `-D warnings` 通过。Xvfb/CEF fixture 日志中的 Fontconfig warning 不影响测试结果。
- 本 Task 完成后暂停等待用户提交；未提交或 push。

## 2026-10-15：五类型界面与本地化（6.6）

- 过滤、卡片图标、预览占位及详情统一使用实际 `wallpaperType`，不受旧 category 值影响；详情展示中英类型、来源和兼容状态。缺少或损坏的预览文件/图片数据均显示类型图标与本地化占位文本。
- Web 组件缺失时详情明确提示安装入口，不能显示“可用”或触发隐式安装。不支持类型、缺失资源及其他加载错误显示本地化说明，并保留原始资产诊断；不可应用项目禁用应用操作。
- 按已确认的范围修订，GUI 属性编辑继续留待后续；本 Task 不计入属性编辑或 Scene 属性覆盖的完成证据。
- `flutter test test/shell_widgets_test.dart test/controller_logic_test.dart`：47 项通过，其中新增 20 项覆盖两种语言下的五类型过滤/详情/指定输出、兼容错误、Web 组件缺失和损坏预览。设置回归改为向上滚动查找已被 ListView 回收的音量滑块。
- `flutter analyze lib/main.dart lib/src/l10n.dart test/shell_widgets_test.dart` 无问题。未执行完整 CI quality；未提交或 push。完成本 Task 后暂停等待用户提交。

## 2026-10-15：多输出渲染调度（6.1）

- Wayland/X11 共用安全帧间隔计算：`fps_limit` 为零或过大值不会除零；事件唤醒不能绕过帧率上限。电池暂停使用统一低频调度。
- Wayland 输出使用实际 buffer scale 计算像素尺寸，resize 后更新 session 的 OutputInfo；X11 监听 RandR resource 变化并周期性检查逻辑显示器变化。Video、Rust Scene、Web 在各自输出独立首帧、暂停、恢复和尺寸更新。
- Scene/Web 暂停时保留最后一帧，但尺寸变化仍会重绘一次；Image 首帧和尺寸变化后只绘制一次，GIF 继续动画。媒体布局配置在运行时更新并覆盖 Fill/Cover/Stretch/Contain/Centre。
- 新增 `scripts/fixtures/mixed_outputs.py`、`output_control.py` 及 Rust `output_tests`。在私有三输出 Wayland（Sway headless）和 X11（Xvfb）环境验证 Video/Scene/Web 混合、单输出暂停、800×600/scale 2 resize、输出背景窗口属性、Image 静态停留、GIF 动画和 20 FPS 上限；Web 使用已安装临时 CEF 组件，不触发下载。
- `scripts/check-wayland.sh`：嵌套 Wayland backend 4 项通过。三输出 Wayland/X11 mixed fixture 各 1 项通过。`xvfb-run cargo test -p wayvid-engine --locked --offline -- --test-threads=1`：87 项通过；`cargo test -p wayvid-engine --locked --offline`：87 项通过；workspace check、workspace clippy `-D warnings`、fmt 和 `git diff --check` 通过。Fontconfig/Mesa 的软件渲染 warning 不影响结果。
- 本 Task 完成后暂停等待用户提交；未提交或 push。完整 CI quality 尚未执行。

复现混合输出：构建 `build/native-web/varpaper-web-host` 并提供已存在的扁平 CEF runtime，设置 `VARPAPER_CEF_RUNTIME=/path/to/runtime` 后执行 `python3 scripts/fixtures/mixed_outputs.py x11` 或 `python3 scripts/fixtures/mixed_outputs.py wayland`。X11 需要 Xvfb/xrandr；Wayland 需要 sway/swaymsg，始终创建私有 runtime 与 headless 输出。本次 Wayland 使用临时容器内的 Sway 1.12 / wlroots 0.20.1，GL 在主机使用 Mesa 软件渲染；临时容器和镜像已删除。Scene 仍使用已声明的 stretch 布局，未扩展后续布局范围。

首次未固定 `LIBGL_ALWAYS_SOFTWARE=1` 的全量原生回归中，一个 Web 候选在 100 次切换测试中首帧超时；固定软件 GL 后重新执行 87 项全部通过（186.11 秒）。该结果不替代 Task 8.x 的真实桌面、安装包或完整资源 soak 验收。

## 2026-10-15：统一 renderer 生命周期（6.2）

- Wayland 清除、断屏、layer closed 和 shutdown 统一经 `remove_wallpaper` 调用 session cleanup；与 X11 保持 renderer → EGL surface → native window 的释放顺序。候选或当前 renderer 的 `close()` 报错仍会释放两个 renderer 和 EGL，返回首个错误；绑定失败仍保留所有权供 Drop 重试。
- Wayland EGL 使用 swap interval 0，由引擎调度限帧，避免断屏时等待 EGL 帧回调阻塞输出事件和退出。首轮默认 swap interval 的 Wayland 重复断屏发生事件超时；调试运行未复现，改用非阻塞帧同步后四次独立运行全部通过。此处是针对阻塞风险的修正，不将未取得线程栈的首次超时归因视为已证实。
- 扩展私有三输出 fixture：Video/Scene/Web/Image/GIF 逐类断屏、重连、重新应用、单输出/全输出清除，检查 native、Scene GPU/script/sound、Web 宿主/renderer owner/program/texture/VAO、audio consumer/worker 计数回到基线；带 Video/Scene/Web 活跃会话 shutdown 时两后端在引擎线程断言 graphics/宿主所有权归零。
- 保存赋值仍由 Flutter 管理；断屏仅释放运行资源，重连按保存描述重新应用。扩展 Scene/Web controller fixture，验证项目上下文和属性保留，显式清除后再次重连不会复活赋值。
- `python3 scripts/fixtures/mixed_outputs.py x11`：1 项通过（19.60 秒）；Wayland 同一 fixture 四次通过（13.61 / 13.66 / 13.73 / 12.82 秒）。Wayland 使用临时 Sway 1.12 容器和私有 headless 输出，X11 使用私有 Xvfb，CEF 使用已有 `/tmp/varpaper-controls-runtime`；不接触用户桌面，不下载 CEF。临时 Sway 容器、镜像与命令包装器已清理。
- `scripts/check-wayland.sh`：4 项通过（65.73 秒）；常规 `cargo test -p wayvid-engine --locked --offline -- --test-threads=1`：88 项通过；Flutter `controller_restore_test.dart`：10 项通过；该文件 analyze、workspace Clippy `-D warnings`、fmt 和 `git diff --check` 通过。
- 新增 close 错误回归覆盖候选失败、当前失败、两者失败与幂等 cleanup。首次全量回归发现原有模拟 Host 测试手动构造对象未计入新计数器，已补计数并重跑。
- 修复后 Xvfb/真实 CEF 全量原生回归：88 项全部通过（184.48 秒），包括原有 100 次跨媒体/Scene/Web 切换、失败初始化、clear 和 shutdown 检查。`openspec validate add-all-wallpaper-types --strict` 通过。
- 完整 CI quality 与 Task 8.x 桌面/安装包验收另行执行。本 Task 完成后暂停供用户提交；未提交或 push。

## 2026-10-15：Linux 构建与原生产物清单（7.1）

- `native/web/bundle.cmake` 为 Flutter Linux 与独立 Web CMake 安装提供共同清单：`libexec/varpaper-web-host`、`data/runtime.json`、JSON/CEF 许可证。宿主兼任 CEF subprocess，不新增第二个 helper。Rust Scene 已随 Cargo 编入 `libwayvid_gui.so`，核对构建路径未恢复旧 Scene CMake、独立库、snapshot 或参考 checkout 依赖。
- 直接 `flutter build linux` / `flutter run -d linux` 与 `scripts/build-linux.sh` 共用 CMake 的 Web 构建目标，不再依赖脚本预先导出 helper 路径。Flutter 的 helper 产物位于其构建目录；独立脚本默认仍输出 `build/native-web`，可用 `VARPAPER_WEB_BUILD_DIR` 指定隔离目录。预构建 helper 的相邻 SDK/IPC manifest 必须匹配；缺失、不匹配、匹配三种 CMake 配置结果均已验证。
- 新增 `scripts/check-linux-bundle.py`，`build-linux.sh` 构建后自动调用。检查必需产物、manifest、ELF、动态链接、旧 Scene 库/参考绝对路径、意外捆绑的 CEF 运行文件及 helper RPATH。主程序不使用 CEF 搜索路径；仅 helper 可在未装组件时缺少 `libcef.so`。传入 `--web-runtime` 时 helper 的全部依赖也必须解析。保留 Flutter 必需的 `data/icudtl.dat`；首次检查错误地将它归为 CEF 文件，修正后重跑通过。
- 在 `/tmp/varpaper-build-clean-9lzgccel/source` 导出当前源码（不复制 `.git`、build、target、参考 checkout 或旧产物），离线解析已缓存的 pub/Cargo 依赖，从头完成 Linux release 的 Cargo/Flutter/Web helper 编译。CEF SDK 使用已有固定版本目录，仅作为 Web 编译输入；首次 Flutter 构建补齐了 Flutter SDK 自身的 Linux 引擎缓存，不下载或安装用户 CEF 组件。
- `scripts/build-linux.sh` 与随后直接 `flutter build linux --release --no-pub` 均成功。基础 bundle 的 12 个 ELF 无旧 Scene 独立库或参考路径，主程序库全部解析；提供已有 `/tmp/varpaper-controls-runtime` 后 helper 链接检查也全部通过。独立 Web 的 `Runtime` component 安装恰好产生清单中的 4 个文件，不含 Chromium 运行库。
- 反例检查拒绝额外 `libcef.so`、版本化旧 Scene 库、CEF locale、额外 ICU 文件、错误 manifest 和遗漏 helper；共 6 项按预期失败。更新构建说明、Web README 与脚本索引。
- 对干净 bundle 的实际 `libwayvid_gui.so` 执行 `scene_package_smoke_test.dart`：1 项通过（Scene 首帧、错误资产、暂停/恢复、清除），在私有 Xvfb 和空组件 data 目录下运行。使用该 bundle 的 helper 和已有 CEF runtime 执行 `native/web/test_host.py`：本地资源、Canvas/WebGL、多 browser、resize、暂停/恢复、隔离及全部子进程退出通过。
- shell 语法、Python 语法和 `git diff --check` 通过。未执行三种安装包构建或完整 CI quality；它们分别属于 7.2–7.5/8.x。本 Task 完成后暂停供用户提交；未暂存、提交或 push。


## 2026-10-15：Debian 安装包（7.2）

- `.deb` builder 复用 Linux bundle checker，分别校验打包暂存目录和实际归档解包后的目录。amd64 必须带 `libexec/varpaper-web-host`、固定 `data/runtime.json` 与 Web/Scene 许可证；宿主同时作为 CEF subprocess。主程序不能链接 CEF，基础包拒绝 CEF 库/resources/locales、旧 Scene 独立库和参考绝对路径。保留 Flutter 自身必需的 ICU 文件。
- 补充 OpenSSL、CA 证书、Fontconfig、DejaVu fallback、PulseAudio、C++/GCC runtime 以及已固定 CEF 所需系统共享库依赖；不要求安装 CEF/Chromium 软件包。运行库仍只能由设置中的显式安装获取。沿用 Flatpak/Arch 的现有处理，在复制后移除 Linux 未使用的 `libdartjni.so`，避免它的构建机 JVM 绝对 RUNPATH 和额外 Java 依赖；不修改输入 bundle。
- 复用 7.1 的干净 release bundle，执行 `VARPAPER_BUNDLE_DIR=/tmp/varpaper-build-clean-9lzgccel/source/build/linux/x64/release/bundle VARPAPER_OUTPUT_DIR=/tmp/varpaper-deb-7.2 packaging/deb/build.sh 20261015.g12345678` 成功。产物为 `/tmp/varpaper-deb-7.2/varpaper_20261015.g12345678_amd64.deb`，版本为验证用标记；未作为发布包上传。暂存与解包后的 11 个 ELF 全部通过 checker；默认仅 helper 允许缺少 `libcef.so`，提供已有 `/tmp/varpaper-controls-runtime` 后 helper 全部链接也解析成功。
- 核对归档 control 的依赖、launcher 可执行权限、desktop/icon、manifest 内容及三份 Web/Scene 许可路径；确认不存在 JNI 和 CEF 强制包依赖。五个反例（缺少 helper、缺少 manifest、捆绑 libcef、版本化旧 Scene 库、CEF locale）全部在生成归档前被拒绝。
- `VARPAPER_TEST_FLUTTER=/tmp/varpaper-flutter-ci/bin/flutter scripts/check-scene-package.sh /tmp/varpaper-deb-7.2/varpaper_20261015.g12345678_amd64.deb`：1 项通过。私有 Xvfb、空组件 data 目录下，实际包内 Rust 库完成 Scene 首帧、错误资产拒绝、暂停/恢复和清除。
- 对解包后的实际 helper 在私有 Xvfb、软件 GL、dummy audio 下运行 `native/web/test_host.py`，使用已有固定 CEF runtime：本地资源、Canvas/WebGL、独立 browser、resize、暂停/恢复、隔离和全部 subprocess 退出通过；不下载或安装组件。Fontconfig warning 不影响结果。
- `bash -n packaging/deb/build.sh` 与 `git diff --check` 通过。当前验证产物由 Arch 主机构建，control 自动记录 `libc6 (>= 2.44)`；未宣称旧 Debian/Ubuntu 安装兼容性，正式发布需在支持的最低版本基线上构建。完整发行版安装、用户组件安装/卸载和 quality 验收仍属于 8.x。本 Task 完成后暂停供用户提交；未暂存、提交或 push。


## 2026-10-15：Flatpak 安装包与 Web 沙箱（7.3）

- Flatpak manifest 增加 Zypak v2025.09（`693a71c5ffa80ec9c9ce2ae03b1ccc493c698e53`）及 BSD notice 安装，来源为 https://github.com/refi64/zypak；按其 CEF 使用说明，仅在 Flatpak 中将 Web helper 通过 `/app/bin/zypak-wrapper` 启动，传入实际组件的 `ZYPAK_CEF_LIBRARY_PATH`。GUI 不加载 Zypak/CEF；不增加 CEF runtime module 或基础包运行文件。NOTICE 记录来源与本地补丁。
- Chromium 原生嵌套 namespace/setuid sandbox 在 Flatpak 中不可用，首轮真实预检因此失败。Zypak 改用 Flatpak portal 的子沙箱，保留 sandbox 与无网络子进程策略。`zypak-cef-runtime.patch` 只读暴露已安装组件目录并传递子进程 loader 路径；调试发现 Zypak 的 EnvMap 值为 string_view，已将路径字符串保留至同步 Spawn 调用结束，修复临时字符串引用失效造成的 libcef 加载失败。未通过关闭 sandbox 绕过启动问题。
- portal 子进程不会继承测试环境变量；helper 现在主动报告 renderer client identity，fixture 不再依赖 renderer 中的主机测试变量。扩展既有 `test_host.py`，在 Flatpak 中使用 Zypak，并纳入 portal 启动、非 OS 后代的 CEF 子进程进行 crash/cleanup 检查；保留原生路径。
- builder 在复制后移除未使用 JNI，校验完整 base bundle；不遍历 Web helper 的主机 CEF 依赖闭包，避免组件运行文件混入 `/app/lib`。在 SDK 内检查全部 11 个应用 ELF、补充共享库和两个 Zypak 程序的链接，验证 mpv 使用 Platform FFmpeg ABI、Scene fallback 字体、PulseAudio、helper/manifest/licenses，拒绝额外 CEF、主机 FFmpeg/JVM/GL。共同 checker 新增显式 `--library-dir /app/lib`，不继承主机 CEF 搜索路径。
- GNOME Platform/SDK 50 提供 SDL2、PulseAudio、OpenSSL、图形库和 `/usr/share/fonts/dejavu/DejaVuSans.ttf`。曾尝试启用 mpv 自带 X11 后端而因缺少 xpresent 失败，已恢复原配置：引擎拥有 Wayland/X11 surface 与 EGL context，mpv 使用其 render API，不创建 X11 窗口。保留既有固定 libass/libplacebo/mpv modules，不复制主机 mpv/FFmpeg。
- 干净 7.1 bundle 的 Arch Rust 库要求 glibc 2.44，GNOME 50 只有 2.42；在已安装 GNOME SDK 中以缓存 Cargo 依赖离线重新编译当前 `wayvid_gui` release（编译时链接已有 GNOME mpv），替换验证输入的 Rust 库。使用固定本地 CEF SDK 重新构建当前 helper，不依赖参考 checkout。builder 的 ABI 预检覆盖主程序、全部 bundle 库与 helper；构建缓存保存在 `build/flatpak-state`，固定来源使用 `--disable-updates`，新增来源仍按 manifest 下载。
- 完整执行 `VARPAPER_BUNDLE_DIR=$PWD/build/flatpak-7.3-bundle VARPAPER_OUTPUT_DIR=/tmp/varpaper-flatpak-7.3 packaging/flatpak/build.sh 20261015.g12345678` 成功。最终 `/tmp/varpaper-flatpak-7.3/varpaper_20261015.g12345678_x86_64.flatpak` 为 21,297,928 bytes，版本为验证标记。最终完整构建日志 `/tmp/varpaper-flatpak-7.3-final-build.log`；构建/SDK 内链接检查全部通过。
- 安装该最终 bundle 到临时用户级 app installation，在实际 GNOME Platform（未使用 --devel/SDK）内关闭网络，确认 Rust engine/mpv 可加载、进程 maps 不含 libcef、整个 `/app` 不含 libcef，且 helper/manifest/Zypak 存在。
- 私有 Xvfb、Mesa/SwiftShader、dummy audio、`flatpak run --unshare=network` 下，对实际 `/app/lib/varpaper/libexec/varpaper-web-host` 运行完整既有 Web fixture，通过本地 HTML/CSS/JS/image、Canvas/WebGL、独立 browser、resize、初始/修改属性、音频/静音/音量、暂停/恢复、远程/逃逸请求拒绝、renderer crash 隔离及全部子进程退出。组件使用应用私有 data 下的独立完整副本，不执行下载；最终日志 `/tmp/varpaper-flatpak-7.3-web-verified.log`。Zypak 的 waitpid group warning 未影响断言或退出结果。原生 X11 当前 helper 同一 fixture 也通过（`/tmp/varpaper-7.3-web-native-final.log`）。
- `scripts/check-scene-package.sh` 对最终 Flatpak 解包后的实际 Rust 库执行既有 Scene fixture：1 项通过，覆盖首帧、坏资源拒绝、暂停/恢复、清除。该项在主机私有 Xvfb 中运行，区别于上述真实 Flatpak browser 验收，日志 `/tmp/varpaper-flatpak-7.3-scene.log`。
- 验证后卸载本轮用户级 app，删除唯一的临时 CEF runtime/browser-cache 副本；保留原有 app data，未触碰用户桌面/音频。`flatpak ps` 确认无本轮 app/CEF 实例，临时 origin 随卸载清理。验证包和构建缓存保留便于复查。
- shell/Python/JSON 语法、Rust fmt、共同 checker 的既有 .deb 链接回归和 `git diff --check` 通过。设置内完整组件安装/卸载、真实桌面五类型和完整 quality 仍属 8.x；此处未宣称完成它们。本 Task 完成后暂停供用户提交，未暂存、提交或 push。


## 2026-10-15：Arch 安装包（7.4）

- 更新 PKGBUILD 的壁纸类型描述，显式补充 OpenSSL、CA 证书、PulseAudio、GCC/glibc 以及固定 CEF 使用的 NSS/NSPR、ALSA、CUPS、Mesa、X11 系统库。CEF/Chromium 不作为 pacman 依赖或基础包文件。Python 仅用于构建校验。容器安装依赖和归档 metadata 断言均读取生成的 PKGBUILD，避免维护重复清单。
- recipe 复用共同 Linux bundle checker：要求 Rust engine、Web helper（兼任 subprocess）、固定 manifest 和 Scene/CEF/JSON 许可，拒绝捆绑 CEF/旧 Scene 库/参考绝对路径，检查全部 ELF。新增显式 `--manifest` 参数供隔离容器使用源清单，不依赖源 checkout 路径或继承的 LD_LIBRARY_PATH。补齐应用内图标路径，保留系统 desktop/icon 与所有根许可/NOTICE。
- Docker 内执行当前滚动 Arch 的 pacman 更新及 makepkg，检查 staging 与归档解包 payload，外层 builder 再检查解包结果。第一次归档构建及容器校验已成功，但外层与容器使用同一个 extracted 目录导致 mkdir 失败；将外层改为 archive-check 后完整重跑成功。未修改执行中的构建脚本。
- 完整命令：`VARPAPER_ARCH_USE_DOCKER=1 VARPAPER_BUNDLE_DIR=$PWD/build/flatpak-7.3-bundle VARPAPER_OUTPUT_DIR=/tmp/varpaper-arch-7.4 packaging/arch/build.sh 20261015.g12345678`。复用 7.1 干净 Flutter bundle、7.3 在 GNOME SDK 重编的当前 Rust library 和当前 helper；在 Arch 上全部动态库解析成功。最终产物 `/tmp/varpaper-arch-7.4/varpaper-20261015.g12345678-1-x86_64.pkg.tar.zst`，14,238,151 bytes，版本仅为验证标记。日志 `/tmp/varpaper-arch-7.4-build.log`。
- staging、容器内归档及主机解包的 11 个 ELF 全部通过检查；未安装组件时只允许 helper 缺少 libcef.so。额外提供已有 `/tmp/varpaper-controls-runtime` 后 helper 全部链接也成功。核对 .PKGINFO 的全部依赖、helper/manifest/三份许可、launcher、应用图标及根许可安装清单；归档 NOTICE 与当前源文件一致。
- 最终归档执行 `scripts/check-scene-package.sh`：1 项通过，实际包内 Rust library 在私有 Xvfb、空组件 data、软件 GL/dummy audio 下完成 Scene 首帧、坏资产拒绝、暂停/恢复、清除。日志 `/tmp/varpaper-arch-7.4-scene.log`。
- 对实际归档 helper 执行 `native/web/test_host.py`，使用已有固定 CEF runtime，私有 Xvfb/软件 GL/dummy audio：本地资源、Canvas/WebGL、独立 browser、resize、属性/音频、暂停/恢复、renderer crash、隔离和全部子进程退出通过。日志 `/tmp/varpaper-arch-7.4-web.log`。未下载 CEF 或连接用户桌面/音频；不将这些 host fixture 当作完整发行版安装验收。
- 更新现有构建文档和 NOTICE，核对 CEF manifest 的版本、SHA-256、官方 HTTPS 出处及 JSON 固定 revision。shell/Python 语法、git diff 检查和 OpenSpec strict validation 通过。Arch 容器自动清理，包与下载缓存保留；未在主机安装依赖。设置内组件安装/卸载、真实桌面及完整 quality 保持 8.x 未完成。本 Task 完成后暂停供用户提交，未暂存、提交或 push。


## 2026-10-15：CI 原生质量门禁（7.5）

- `quality` 已将 native gate 前置到 Flutter 检查前，并安装 PulseAudio/pactl、D-Bus、CEF helper 运行所需的 NSS/NSPR/ALSA/CUPS/GBM/X11 库及 Xvfb。新增 Python AST、JSON manifest 和 Arch recipe 语法检查。
- `scripts/check-native.sh` 现在固定软件 GL、临时 data/config/state、私有不存在的 Pulse socket，先执行 Rust Scene 解析/脚本/GL fixture，再执行私有音频 fixture、CEF helper fixture 和 workspace X11/生命周期测试。workspace 输出必须包含 X11 Scene fixture 与 Web install/uninstall/restart fixture，失败不会被“无测试执行”吞掉。
- `scripts/fixtures/scene_audio_monitor.py` 增加 `--backend pulseaudio|pipewire`。CI 使用无默认配置、无硬件发现的 PulseAudio daemon，仅创建 `module-null-sink`；检查 Scene sound PCM、mute/volume、pause/resume 和 cleanup。现有 PipeWire/WirePlumber fixture 保留用于本地环境。mpv 属性读取增加有限异步等待，避免加载事件和属性查询竞态。
- 本地 `VARPAPER_TEST_AUDIO_BACKEND=pipewire scripts/check-native.sh` 的 Rust Scene、12 项 GL、私有音频、CEF helper 和 workspace engine 88 项均通过；首次完整 workspace 在共享 target 下遇到已生成测试二进制消失（原因未证实），改用独立 `CARGO_TARGET_DIR=/tmp/varpaper-target-7.5 cargo test --workspace --locked -- --test-threads=1` 后所有 workspace crate/doc tests 通过。独立 target 重跑通过，未据此断言首次失败的根因。系统未安装 pulseaudio，因此本机未声称 CI PulseAudio backend 已执行；独立的虚拟 PulseAudio 运行曾验证 Scene sound fixture 1 项通过。
- `cargo clippy --workspace --all-targets -- -D warnings`（独立 target）和 `cargo fmt --all -- --check` 通过；shell/Python/JSON、`git diff --check` 通过。干净源码副本 `/tmp/varpaper-ci-clean-7.5-lnni13tw/source` 不含 `native/wallpaperengine`，构建不读取参考 checkout 或 Scene 动态库。Task 7.1–7.5 完成；8.x 桌面、soak、三包安装和完整 coverage 仍未完成。本 Task 完成后暂停供用户提交，未暂存、提交或 push。


## 2026-10-15：Wayland/X11 桌面验收（8.1）

- X11 私有 Xvfb 重新执行 `VARPAPER_CEF_RUNTIME=/tmp/varpaper-controls-runtime python3 scripts/fixtures/mixed_outputs.py x11`：三输出 Video/Scene/Web 首帧、多输出独立暂停、帧率上限、800×600/scale 2 resize、Image 静态帧、GIF 动画、Web 清除/重建、断屏/重连及五类型逐类生命周期均通过；Rust fixture 1 项通过（20.69 秒）。日志 `/tmp/varpaper-8.1-x11.log`。
- Wayland 私有 nested niri（独立 XDG_RUNTIME_DIR、软件 Mesa、dummy audio）执行 `scripts/check-wayland.sh`：输出枚举、native session 优先级、Scene/media 失败清除恢复、20 次重启与清理共 4 项通过（62.93 秒）。日志 `/tmp/varpaper-8.1-wayland.log`。此前使用临时 Sway headless compositor 的三输出 mixed fixture 已在 7.1 验证记录中通过；其环境和复现命令保留在该记录，未连接用户 compositor。
- 验收 fixture 在私有显示服务器中直接断言 framebuffer 尺寸、presented frame、播放状态、类型路径和 renderer/native 计数，没有生成不可复现的桌面截图；软件渲染和 Fontconfig warning 不影响断言。未发现失败项。真实物理热插拔、不同 compositor/显卡和完整发行版安装仍不由此项覆盖；8.2–8.4 保持未完成。本 Task 完成后暂停供用户提交，未暂存、提交或 push。


## 2026-10-16：五类型资源 soak（8.2）

- 扩展原生 X11 `web_texture_orientation_first_frame_and_stop_preserve_media_and_scene`，每轮依次应用 Image → Video → Scene → Web → GIF，每次成功后尝试损坏 Scene 候选并验证旧赋值保留，再暂停/恢复、resize，最后清除。共 100 轮、500 次类型应用及 500 次失败候选初始化。
- 私有 Xvfb、Mesa 软件 GL、dummy SDL、不可用的私有 Pulse socket；CEF 使用已固定并校验的外部 SDK runtime，helper 使用独立 `/tmp/varpaper-8-web-host`，不连接用户桌面/音频。测试在 `target/quality-native-8.4` 的实际 engine 测试二进制执行，指定完整测试名及 `--exact --nocapture`。
- `/tmp/varpaper-8.2-counts.log`：1 项通过（233.40 秒）。每轮断言全部计数回到基线，每 10 轮打印：native `(1, 0)`（保留测试 EGL display，无 window）、Scene `((0, 0), (0, 0), 0, 0, 0)`、Web `[0, 0, 0, 0, 0]`、audio `(0, 0)`；Web 宿主均已退出。最后 drop EGL 后 native 也回到测试前基线。无持续资源累积；这是引擎所有权计数，不是进程 RSS 或驱动显存测量。
- 此前短测试名配 `--exact` 的零项运行、未固定软件 CEF 的首帧失败，以及 SDK 被 Flutter clean 删除/运行库资源路径不完整的失败均不作为通过证据。修正隔离目录和 runtime 布局后，上述完整测试与 native gate 中同一 100 轮测试均通过。


## 2026-10-16：quality 本地验收（8.4，待当前提交 CI）

- Rust 1.88.0 `check --workspace --all-targets --locked`、workspace Clippy `-D warnings`、`cargo fmt --all -- --check` 全部通过。编译命令设置 `RUSTFLAGS=-D warnings`，使用隔离 target；日志 `/tmp/varpaper-8.4-msrv.log`、`/tmp/varpaper-8.4-clippy.log`。
- `cargo llvm-cov --workspace --ignore-filename-regex frb_generated --lcov --output-path lcov-rust.info` 通过：engine 88、library 40、GUI 34 项。普通 coverage 环境不启用原生 fixture，相关测试的提前返回不算真实原生验收；下面独立 native gate 提供该证据。日志 `/tmp/varpaper-8.4-rust-cov.log`；生成的报告另存 `/tmp/varpaper-8.4-lcov-rust.info`，不提交覆盖率产物。
- `scripts/check-native.sh` 使用外部固定 CEF SDK/helper、独立 target、Mesa 软件 GL、私有 PulseAudio（仅 null sink，无硬件发现）、Xvfb 执行成功。24 项 CPU/script、12 项真实 GL、1 项 OpenGL 3.0 拒绝、1 项真实私有音频及 Web helper fixture 通过，随后 workspace 88 + 40 + 34 项通过，包括完整五类型 100 轮 soak 及真实 uninstall/restart。日志 `/tmp/varpaper-8.4-native.log`；本机未安装 PulseAudio，使用临时提取的官方 Arch PulseAudio 二进制/模块，不修改系统服务。
- 修复 native gate 末尾的测试选择断言：原字符串 `x11_scene_fixture_tests::` 与实际模块名不符，改为完整 `engine::x11::scene_fixture_tests::textured_scene_loose_packed_shared_and_missing_resources`。重新执行整个 gate 后最终选择断言通过。
- 与 workflow 一致的全部 shell/Arch recipe 语法、三个 JSON manifest 和 Python AST 校验通过。新 Settings package fixture 纳入 CI analyze，默认不启用网络下载；未设置 opt-in 的两个 package fixture 测试均跳过。
- 原工作区 Flutter clean 因旧 build 内部分文件权限报告清理失败；未忽略该验收缺口。将当前源码（包括新 fixture，不含 build/target/参考 checkout）复制到 `/tmp/varpaper-8.4-clean-0hnqzgng/source`，完整执行 clean → pub get → workflow 指定 analyze → test --coverage，全部成功，82 项测试通过。日志 `/tmp/varpaper-8.4-clean-flutter.log`，实际 Flutter lcov 位于该副本 `coverage/lcov.info`。Flutter 3.47.4；最终原工作区 analyze 亦通过（`/tmp/varpaper-8.4-final-analyze.log`）。
- Task 8.4 保持未完成：用户已明确批准先完成本地验收、当前工作区提交推送后再检查 CI/Codecov。历史成功 run 38118275488 属于其他提交，不能证明本次改动上传成功。未暂存、提交、推送或触发远程工作流；每次 push 前仍须遵守 AGENTS.md 的完整 quality 门禁。


## 2026-10-16：设置安装/卸载的三包本地验收（8.3，部分完成）

- 按用户批准，8.3 验收范围调整为当前 Rust Scene 子集；实时 Scene 系统音频可视化及可注入本地时钟仍明确延期，不以内部手动 spectrum/单调动画时钟 fixture 冒充上述能力。Task 8.3 保持未完成。
- 新增 opt-in `scripts/fixtures/component_package_smoke_test.dart`，实际加载包内 Rust service 与 helper，使用真实 Settings widget/controller，只有存储由内存 FakeStore 替代。测试先确认组件缺失、Web 可发现但应用失败且没有创建组件目录，执行无 CEF 的 Video/Image/GIF/clear-color Scene，再点击 Install 触发官方固定归档下载、SHA/清单校验及原子安装，执行五类型与暂停/恢复，Web 活跃时点击 Uninstall，确认状态 not_installed、diskBytes=0、runtime 删除而项目文件保留。
- 复现：`VARPAPER_TEST_FLUTTER=/tmp/varpaper-flutter-ci/bin/flutter VARPAPER_TEST_COMPONENT_INSTALL=1 scripts/check-scene-package.sh PACKAGE`。私有 Xvfb/data/cache、软件 GL/dummy SDL、不可用的私有 Pulse socket；自动清理临时安装及解包目录。只在显式 opt-in 时下载 CEF。修复临时目录名称断言，并为显式 helper override 提供相邻 runtime manifest 的临时链接布局；不更改包内清单。
- 三种既有 Task 7 验证包分别成功：Arch `/tmp/varpaper-8.3-arch-settings.log`（75 秒）、Debian `/tmp/varpaper-8.3-deb-settings.log`（132 秒）、Flatpak `/tmp/varpaper-8.3-flatpak-settings.log`（128 秒），各 1 项通过。它们均使用主机动态库和测试窗口，未安装为用户桌面应用；Flatpak 解包执行也没有进入其 sandbox。
- 这些通过结果不等于完整 8.3：默认阶段没有隔离网络或抓包，未以运行断言证明零联网；Web 页面注册了属性/音频回调，但此 package fixture 没有断言回调结果。当前只覆盖音频采集不可用时仍可渲染；私有 PulseAudio 采集可用的证据来自独立 native gate，不能替代三包各自的两种音频状态。延期 Scene 能力的 GUI/资产诊断也仍需专门的 package 断言。
- 后续验收所需环境/产物：三发行包实际安装和 Flatpak sandbox 内的完整 Settings 流程；现有验证 `.deb` 在 Arch 构建并声明 `libc6 >= 2.44`，不能用它宣称支持较早 Debian/Ubuntu，需在支持的最低发行版构建基线上重新产包。此轮不放宽这些验收条件、不将 8.3 标记完成。现有 7.3 sandbox browser 通过记录仍保留，但不当作 sandbox 中设置安装器的通过证据。


## 2026-10-16：Task 8.3 package installation follow-up

- Rebuilt and installed Debian on Ubuntu 24.04 and Arch in isolated containers. Both disconnected default phases passed Video/Image/GIF/Rust Scene without CEF or a component directory. Settings install, typed Web property pixel checks, five-type pause/resume, active Web uninstall, zero disk usage, and project preservation passed with unavailable audio and with private PulseAudio null-sink audio (nonzero spectrum).
- Built the Flatpak with GNOME 50 and pinned Zypak. Its installed Platform sandbox default phase passed with `--unshare=network`, no reference checkout, no CEF in `/app`, and the four delivered types. A direct Platform service probe completed Web helper IPC. The full Settings loop remains unstable after Web uninstall: the sandbox helper intermittently exits with `ECONNRESET`/`EAGAIN`, and a sandbox download can correctly remain in `error`; this is not marked as Task 8.3 completion.
- The opt-in fixture now asserts typed Web properties and 128-band audio callback pixels, and explicitly reports live Scene system-audio visualization and injected local-clock bindings as unavailable. Flatpak portal children require `TMPDIR` under an app-visible persistent directory; parent private `/tmp` is not reusable.
- 8.3 remains unchecked pending stable Flatpak Settings install/apply/uninstall. 8.4 remains unchecked pending CI/Codecov on the submitted commit.
