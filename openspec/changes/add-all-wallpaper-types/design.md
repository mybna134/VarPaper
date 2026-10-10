# Design

## Context

See proposal.md for motivation and the four capability deltas for acceptance contracts.

当前 `WallpaperType` 包含 Video/Scene/Gif/Image，但 `WeProject::is_supported` 仅接受 Video/Scene；入口必须是实体文件，导致包内 Scene 入口被跳过。未知类型默认返回 Video。Flutter DTO 虽保留 `wallpaper_type`，分类只区分 video/scene。应用和 Isar 赋值仅携带路径，缺少项目资源根及属性。

`WallpaperSession` 持有 `MpvPlayer`、EGL surface 和 GL owner，Wayland/X11 共用此会话，已有原生资源清理及泄漏回归测试。EGL 使用桌面 OpenGL core context；参考项目要求 OpenGL 3.3，整合需验证最低上下文及 shader 兼容性。

参考项目研究基线为 `b016d7d1fdcf4e5fd2f9c9fa420a8aaa07fee02d`（本地工作树无修改），其 `ProjectParser`/`CWallpaper::fromWallpaper` 仅分派 Scene、Video、Web；Scene 具备 Package/Texture/Material/Effect 解析、CImage/CParticle/CText/CSound、QuickJS SceneScript；Web 使用 CEF windowless browser 和 GL texture 更新。参考代码存在 TODO 和不完整 API，不将它视为 Wallpaper Engine 官方全功能保证。该目录仅用于只读研究，不编辑、不作为发布依赖。

现有主规格仍含 daemon、YAML 及部分旧约束，与 README 和当前 Flutter/FRB/Isar、X11 实现不一致。进行中的 `refactor-flutter-owned-state-and-engine-boundary`、`refactor-gui-to-flutter-frb`、`add-x11-wallpaper-backend`、`fix-wallpaper-memory-leaks` 涉及同一边界；本变更新增类型能力，不复活 daemon，也不改写这些变更的历史任务。

## Goals / Non-Goals

**Goals:**
- 保持 Flutter 为配置和持久化唯一所有者，Rust 为渲染调度所有者，Wayland/X11 为 surface 所有者。
- 通过参考实现移植原生 Scene 子系统，减少重新实现专有格式的风险，并保持生命周期可测。
- 用每项功能的可再分发 fixture 验证兼容性，明确音频频谱及时间功能，而非只检查“载入成功”。

**Non-Goals:**
- Application/Wine 执行、Workshop 下载、Windows API、完整 3D/全部官方脚本 API、默认网络访问。
- 将浏览器/Scene 接管桌面窗口或使 Flutter 承担逐帧渲染。
- 重写视频 HDR/硬件解码或无关旧规格；视频参数如 hwdec、HDR 不映射到 Scene/Web。

## Decisions

### 1. 类型化源描述和稳定身份

新增共同源描述：实际类型、稳定 source id、原始路径、可选 project root/manifest/entry、shared asset roots、兼容状态、属性 schema/overrides。独立媒体维持现有路径哈希 ID；项目以 project.json 的规范路径标识，Workshop ID 仅作为元数据。扫描显式定位 project.json，并停止把项目内部资源重复当独立媒体扫描；Scene 入口由资源定位器验证，不能仅用 `Path::exists`。

FRB 添加描述式应用接口并重生成绑定；旧路径应用接口在适配层解析媒体文件、project.json、项目目录和可识别项目内旧入口，保持历史赋值可恢复。异常类型使用 unsupported 状态和 declared type，不再回退 Video。替代方案“只增加 Web 字符串”不能传递资源上下文及可靠恢复，因此不采用。

### 2. 统一 renderer 会话，保留输出所有权

将会话中的 player 替换为类型分派的 renderer 接口：初始化、resize、frame readiness/render、pause/resume、volume/mute、property update、dispose。Video/Image/GIF 仍走 mpv；Image 配置无限停留及一次绘制，GIF 保持动画及循环策略。Scene/Web 接受输出像素尺寸、FBO、GL loader 与时间信息，后端继续创建 EGL/native window 并交换 buffers。

验证当前 GL version，Scene 至少请求参考所需 OpenGL 3.3，缺少能力只使目标初始化失败。每次 render 必须恢复 framebuffer/viewport/GL state，renderer 的 GL 资源释放必须发生在其 context current 且 native window 仍存在时。候选会话准备到首帧后才替换旧会话、发成功事件并持久化；失败候选单独释放。每个 output 使用独立动态状态，资源缓存可共享不可变数据；不强行共享脚本时钟和属性。

替代方案直接启动 linux-wallpaperengine CLI 会产生第二套显示器管理、控制协议和打包依赖，无法确保事务切换，因此不采用。

### 3. 原生 Scene 兼容组件

将参考项目必要的解析器、资源定位器、渲染对象及 QuickJS 封装为仓库内的 C++ 组件，通过窄 C ABI 和 Rust RAII wrapper 集成；不导入上游 main/Steam 下载/桌面驱动。ABI 使用 opaque handles、显式 buffer 所有权、版本号及 error 返回；捕获 C++ 异常，禁止跨 FFI 展开。上游组件依赖 application context 的部分用 VarPaper adapter 提供属性、时间、输入、音频和资产信息。

资源按项目 loose files、project package、显式 shared assets 定位；包目录、压缩长度、纹理尺寸均验证边界与资源预算。保留引用的必要 upstream 内建资产并声明出处；Wallpaper Engine 安装目录资产只定位使用，不将用户/Steam 专有内容打包。SceneScript 限制执行预算和内存，禁用任意宿主 I/O。参考对象/效果逐项写入 fixture 清单；未知 required 特性报错，不能静默丢弃后宣告成功。

替代方案从头用 Rust 重写整个渲染器会同时承担格式、shader、脚本语义等风险；本次选择受控移植，解析/测试接口保持可替换。

### 4. 系统音频与时间是明确输入

进程级音频分析服务通过 PulseAudio playback monitor 获取系统输出（兼容提供 PulseAudio 接口的 PipeWire），计算 FFT/level，一份采集供多个 Scene/Web consumer 使用。输出切换/断开后重新绑定，静音或不可用产生零频谱及非致命诊断。关闭最后一个 consumer 时释放采集资源；壁纸音量控制仅作用于生成声音，默认不打开麦克风。

Scene 将频谱映射到参考实现 shader uniforms、效果/粒子和已声明脚本接口；Web 提供 `wallpaperRegisterAudioListener`。用已知低/高频信号检查频带、范围和顺序，不用随机动画伪装音频响应。

每个 renderer 独立使用 monotonic animation clock/delta，暂停停止脚本 tick 并冻结动画；恢复时重设上一帧采样点。日期/本地时间直接来自系统时钟与时区，QuickJS Date 和文字脚本能更新时钟，恢复首帧立即取当前时间；时间测试使用可注入时钟覆盖跨分钟、日期和时区变化。媒体元数据属于后续扩展，不因“时间功能”隐式承诺全部播放器集成。

### 5. CEF 离屏集成

原生 Web adapter 管理一个进程级 CEF runtime、每赋值的独立 browser/request context、浏览器 subprocess 和 bounded paint buffer。CEF 生命周期在其要求的线程执行，通过队列将 paint/状态交给 Rust engine，不让 CEF 回调直接使用其他线程的 GL context。OnPaint 上传到输出 texture，保留最后一帧以避免无新 paint 时黑屏；resize 时只接收匹配尺寸的最新帧。

项目自定义 scheme 强制根目录校验；默认拒绝远程网络、文件下载和 executable launching。注入属性 schema 对应 typed values，增加参考 Web 路径尚未覆盖的属性/audio callback bridge。pause 需停止页面动画/timers、媒体和脚本推进，并验证恢复；若 CEF API 不能完全暂停，则销毁 browser 并保留静态帧，恢复按同一描述重建，不能只停纹理上传而让脚本/音频持续运行。关闭所有 browsers 并等待 subprocess 退出后关闭 runtime。CEF renderer 崩溃使对应赋值失败且可重试。

替代方案 Flutter WebView 属于 GUI widget 且不具备现有桌面 EGL 渲染边界；采用参考项目的 CEF 路线便于验证 Canvas/WebGL。

### 6. Flutter 持久化、界面与分发

Isar 赋值新增可选描述版本、type、project manifest/root、property overrides，保留 sourcePath/sourceId 以兼容老记录和回滚。新字段为空按旧路径解析；只有已确认应用成功才保存。类型标签/过滤/预览/错误及属性编辑使用真实 type，category 不再把 Image/GIF 合并为 Video；明确呈现不能用于当前类型的设置。

native component 用 CMake 构建，Cargo build integration 和 Flutter Linux 打包共用安装清单。固定上游修订、CEF 发布及校验和，包含 subprocess、CEF resources/locales、运行库和必要 Scene 依赖；不沿用上游隐式联网下载而不校验。维护 .deb 依赖、Flatpak modules/manifest 和 Arch 配方，满足本机参考目录不存在时可构建/运行。来源与保留许可记录到现有 NOTICE；这是实施交付的一部分。

## Risks / Trade-offs

- [原生移植依赖广、context 耦合] → 先完成独立首帧 vertical slice 和 C ABI 构建，再扩展对象/效果；所有列明特性均验收后才完成整个变更。
- [CEF 增加体积、内存与辅助进程] → 延迟初始化、有界 paint 缓存、输出独立关闭、三种发行包 smoke test。
- [上游只覆盖官方功能子集] → 固定参考版本、功能 fixture 与明确 unsupported 诊断；不承诺全部 Workshop。
- [暂停/脚本资源耗尽] → 验证实际停止脚本/音频、限制 script budget、回归失败初始化及恢复路径。
- [音频监控接口不可用] → 零输入继续渲染，捕获状态可见，重连后恢复。
- [跨渲染器 GL 状态及清理泄漏] → 保持 current-context 清理顺序，拓展既有 memory/lifecycle soak。
- [主规格和进行中变更冲突] → 在实施中采用现有代码的 Flutter/FRB/Isar 所有权，变更提交说明指明旧规格待架构变更同步，不同时修改它们的规划文件。

## Migration Plan

1. 固定参考源和构建依赖，完成类型/描述适配及原生首帧链路。
2. 完成 Scene 音频/时钟及 Web 回调，接入统一会话和事件。
3. 添加兼容 Isar 字段和 FRB 绑定、界面，再迁移旧赋值的恢复解析。
4. 跑 fixture、Wayland/X11 mixed-output/lifecycle、三发行包验收和 CI quality；不得以只有扫描通过替代渲染验收。
5. 回滚程序时旧媒体字段仍可读取，新 Scene/Web 赋值由旧版本忽略/报不支持；不删除原项目及用户资源。
