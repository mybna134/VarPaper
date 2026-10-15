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
