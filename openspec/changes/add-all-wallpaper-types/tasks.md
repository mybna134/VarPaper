# Tasks

## 1. 类型与项目发现

- [x] 1.1 扩展 model/DTO 的五种实际类型及 unsupported declared type、项目源描述和兼容状态，移除未知类型回退 Video；以序列化及 DTO 测试验证各类型无信息丢失。
- [x] 1.2 在 workshop/scanner 发现本地及已安装 Workshop project.json，保留独立媒体 ID、去重项目并排除项目内部素材；用重叠扫描、大小写类型、Web/Application 和坏 manifest fixture 验证。
- [x] 1.3 实现目录/package/shared-assets 资源定位和包内入口验证，校验 traversal、symlink、包长度与解压预算；以 loose/packed Scene 和恶意路径/损坏包测试验证。
- [x] 1.4 修正项目预览选择及类型 fallback，不对 Scene/Web 进行视频缩略图提取；通过现有 thumbnail/preview tests 验证失败缓存及非阻塞行为。
- [x] 1.5 在现有 docs/导入说明中记录项目目录、共享资源和不支持类型；对照扫描 fixture 检查文档示例。

## 2. 原生组件与 renderer 边界

- [x] 2.1 将参考修订 b016d7d1fdcf4e5fd2f9c9fa420a8aaa07fee02d 的必要 Scene 组件固定到仓库内原生构建，排除上游桌面/main 管理；用不包含参考绝对路径的独立构建验证，并更新 NOTICE/依赖来源。
- [x] 2.2 建立 C ABI 与 Rust RAII wrapper，明确 handles/buffers/error/thread ownership，捕获 C++ 异常；用重复创建释放及注入初始化失败的 ABI 测试验证。
- [x] 2.3 抽取共同 renderer 接口并适配 mpv Video/Image/GIF，统一配置能力标记；用既有 mpv/session 回归及 Image 无限停留、GIF 动画循环 fixture 验证。
- [x] 2.4 为 Scene 增加 OpenGL 3.3 能力检测/上下文要求，保留媒体上下文兼容；用真实/测试 GL 上下文验证可用首帧及缺失能力的目标级错误。
- [x] 2.5 增加候选首帧后提交赋值及关联成功/失败事件，保持 GL current 清理顺序；以失败替换保留旧赋值、初始化失败资源回收测试验证。
- [x] 2.6 在现有开发文档记录 renderer/C ABI 所有权和本机构建命令；验证示例构建及最小 Scene 首帧可执行。

## 3. Scene 图形与声音

- [ ] 3.1 接入 Scene package/texture/model/material 解析及纹理缓存；以包内/外纹理、压缩格式、缺失共享资源和非法尺寸 fixture 验证。
- [ ] 3.2 接入 image objects、camera/layer transforms、混合和多 pass effects；以固定画面截图及 reference 同 fixture 对比验证层次、裁剪、透明度和效果。
- [ ] 3.3 接入 animated texture 与 particle 更新；以固定随机种子/时间步检查连续帧变化和暂停冻结。
- [ ] 3.4 接入 text/font 和 sound objects；以文本可见、字体 fallback、音量/静音及清除声音的 fixture 验证。
- [ ] 3.5 对 unsupported required objects/shaders 返回兼容诊断，避免上游静默忽略；用未知对象/无效 shader fixture 验证首帧失败及其他输出继续运行。
- [ ] 3.6 在现有 docs/兼容说明维护上述功能与 fixture 对照表；逐项检查每个声明均有通过的 fixture，不使用不可再分发 Workshop 内容作提交资产。

## 4. SceneScript、属性、音频可视化与时间

- [ ] 4.1 接入 QuickJS 初始化/update hooks、项目属性和 layer bindings，加执行/内存预算与禁止任意 host I/O；以属性动画、脚本异常、超时和隔离 fixture 验证。
- [ ] 4.2 实现共享 PulseAudio/PipeWire-Pulse playback monitor、FFT/level、consumer 生命周期；用已知低/高频 PCM 验证频带，模拟断线/输出切换/零输入验证恢复及计数回归。
- [ ] 4.3 将 spectrum/level 连接到 Scene shader、效果/粒子和支持的 script APIs；以音频可视化 Scene 验证实际响应、壁纸静音不影响其他系统音频响应。
- [ ] 4.4 提供 monotonic animation time/frame delta 及系统 Date/local-time 文字绑定；通过注入时钟测试跨分钟/日期/时区和暂停恢复首帧正确时钟、无动画大跳步。
- [ ] 4.5 在现有 docs/兼容说明记录支持的脚本属性/音频接口、时钟和采集不可用行为；验证对应 fixture 及其示例参数。

## 5. Web 离屏渲染

- [x] 5.1 集成固定版本 Chromium/CEF SDK、共享 Web 宿主进程、subprocess 和可选运行组件清单（CEF 在宿主 main 初始化，主 GUI 不链接 libcef），建立线程正确的 runtime/browser 生命周期；用独立 Web 首帧及退出无遗留 subprocess 验证。
- [x] 5.2 建立项目 scheme、根目录校验和默认外部请求/下载拦截；用相对 HTML/CSS/JS/image、目录逃逸和远程请求 fixture 验证正常加载及明确拒绝原因。
- [x] 5.3 将 bounded OnPaint buffer 上传至输出 texture，处理尺寸/scale 变化和无新 paint 保留帧；以 Canvas/WebGL 动画及 resize fixture 检查帧变化、方向、无黑屏和缓存预算。
- [ ] 5.4 注入 wallpaperPropertyListener.applyUserProperties 与 wallpaperRegisterAudioListener，连接属性和共享 spectrum 服务；通过初始/修改属性和已知音频 fixture 验证回调类型、顺序与零输入。
- [ ] 5.5 实现实际暂停页面脚本/动画/音频及恢复、mute/volume，并处理 renderer crash；以 timer/音频计数 fixture 验证暂停无推进、恢复可用、崩溃不影响 GUI/其他输出。
- [ ] 5.6 在现有 docs/兼容说明记录 Web 支持、离线限制、暂停重建可能丢失页面临时状态及错误重试；检查文档行为与 fixture 一致。

- [ ] 5.7 实现官方 HTTPS 固定版本/架构清单、SHA-256 校验、有界安全解包和原子安装；用成功、损坏归档/校验、路径逃逸、断网、磁盘不足、取消/重试及重启检测 fixture 验证，默认启动/扫描/应用均不得下载。
- [ ] 5.8 实现组件 uninstall：阻止新 Web 请求、取消下载、停止 browser/宿主/subprocess/audio consumer 后删除运行库、归档及组件临时/缓存数据；用运行中卸载、失败重试、空间回收及再次安装 fixture 验证，保留用户项目/保存属性和其他类型输出。
- [ ] 5.9 将组件 availability/version/ABI 校验接入发现、应用和保存赋值恢复；用缺失/损坏/不匹配组件和卸载后重启验证明确状态、无隐式下载、旧输出/赋值不被失败候选覆盖。

## 6. 桌面输出与界面持久化

- [ ] 6.1 将所有 renderers 接入 Wayland/X11 输出事件、resize、layout 和 fps scheduling；以混合 Video/Scene/Web 多输出测试验证独立控制、缩放、首帧及背景层位置。
- [ ] 6.2 将清除、切换、断屏、重连和 shutdown 的统一 renderer cleanup 接入两后端；扩展既有 wayland/native lifecycle 测试验证 native/GPU/browser/audio consumer 计数回到基线。
- [ ] 6.3 将 tray、battery pause 和配置更新路由到所有支持类型；扩展 tray/controller tests 验证暂停/恢复及明确的类型专用设置行为。
- [ ] 6.4 扩展 FRB 描述式应用/属性/错误接口并重生成 Rust/Dart bindings，保留旧路径解析适配；以 bridge tests 验证各类型描述、旧项目入口和失败事件。
- [ ] 6.5 为 Isar 赋值新增可选版本/type/project/property 字段并保留 sourcePath/sourceId；扩展 settings_store/controller_restore tests 验证老媒体恢复、新 Scene/Web 属性恢复、失败候选不覆盖存储。
- [ ] 6.6 更新五类型过滤/标签/详情/预览、属性编辑及兼容错误的中英文本；用 widget/l10n tests 验证 Web/Image/GIF 不再归为 Video、错误可见及输出选择。
- [ ] 6.7 更新现有 README 中仅视频和不支持 Scene/Web 的说明及 docs/使用页；逐项对照五类型操作、音频可视化和时钟行为。

- [x] 6.8 在设置添加中英“Web 壁纸支持”组件管理，提供状态、版本、空间占用、安装/进度/取消/重试/卸载；widget/controller tests 验证状态来源真实、组件缺失时的 Web 安装入口和卸载播放提示。

## 7. 安装包依赖与构建

- [ ] 7.1 将原生 renderer/Web 宿主与 helper 产物清单接入 Cargo/Flutter Linux 构建与 scripts/build-linux.sh；干净构建并检查不存在参考绝对路径和运行时未解析库。
- [ ] 7.2 更新 .deb 依赖与文件安装，仅包含 Web 宿主/helper/固定组件清单及 Scene 所需库，不捆绑或强制依赖 libcef/resources/locales；构建包并检查动态链接、辅助进程和资源路径完整。
- [ ] 7.3 更新 Flatpak modules/manifest、原生依赖及可选 Chromium 组件运行配置，基础 bundle 不包含 CEF 运行库；构建 bundle 并在 sandbox 内验证 browser 启动、本地资源可读和退出回收。
- [ ] 7.4 更新 Arch 配方与运行依赖（CEF 不作为强制依赖或基础包文件）；构建并检查安装清单/运行库，更新现有构建文档与 NOTICE 后核对固定版本及出处。
- [ ] 7.5 将 C ABI/native fixture 检查接入 CI quality 并保持既有检查；核对工作流包含新原生组件的构建、测试与资产验证。

## 8. 综合验收

- [ ] 8.1 在支持的 Wayland 和原生 X11 桌面执行五类型/mixed-output、pause、resize、hotplug 和 restart 验收，记录 fixture/环境/截图及失败项；全部场景通过后才标记完成。
- [ ] 8.2 执行至少 100 次跨 Video/Scene/Web/Image/GIF 切换、清除及失败初始化 soak，记录 native/GPU/browser/audio 计数与进程退出，验证无持续资源累积。
- [ ] 8.3 在未安装参考 checkout 的 .deb、Flatpak、Arch 测试环境验证默认无 CEF 且不联网，随后通过设置安装组件执行五类型、Scene 音频可视化/时钟及 Web 回调，再卸载并验证空间回收的 smoke test；记录音频采集可用/不可用两种结果。
- [ ] 8.4 运行 `.github/workflows/ci.yml` quality 全部检查：fmt、clippy、Rust llvm-cov、shell/manifest 校验、Flutter clean/pub get/analyze/test coverage 以及新增 native 检查；记录全部结果，修复并重跑失败检查，每次 push 前保持该门禁，CI 条件触发的 coverage 上传也必须成功。
