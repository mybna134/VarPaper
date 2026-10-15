# Rust Scene 迁移验证记录

2026-10-12，Linux x86_64 / Xvfb / Mesa。

- 用户确认最低编译版本为 Rust 1.88，workspace/library/engine/GUI 声明已统一，CI 新增 minimum-toolchain check。现有 image 0.25.10 已要求 1.88，不将旧 1.75 声明视为真实基线。
- 新 Rust Scene 资源接口复用 ProjectResources；scene/model/runtime/gpu 已建立，依赖锁定 rquickjs 0.6.2、glam 0.25.0、ab_glyph 0.2.32 和现有 image 0.25.10。Cargo +1.88.0 check --workspace --all-targets --locked --offline 实际通过（42.65 s）。新增依赖许可 MIT / MIT OR Apache-2.0 / Apache-2.0；QuickJS 底层 MIT，未引入上游 C++ Scene 类。
- Rust 首帧真实 GL fixture 通过（0.16 s）：检查 clear-color RGB、viewport/scissor/color-mask/clear-color 恢复、失败 resize 保留旧 target、partial texture 分配失败回收及 20 次创建/释放 GPU/EGL 基线。测试直接创建 RustSceneRenderer，未设置/加载自有 Scene 动态库。
- Mesa GL 3.0 的同 fixture 实际执行通过（0.09 s），明确拒绝缺少 OpenGL 3.3 并回收 surface。

当前仅完成 1.1、1.3。应用 renderer factory 仍使用待替换的旧适配，Rust 图层/script/effect 与 C++ 构建删除尚未完成；新 Rust 测试不能替代完整功能矩阵或三包/真实桌面验收。未 push。


## 2026-10-13 — Rust 纹理与生成资产

- `cargo +1.88.0 test -p wayvid-engine scene::texture::tests --locked --offline`：最初 4 项通过；补充动画和分配边界后，`cargo test -p wayvid-engine scene:: --locked --offline` 的 6 项 texture 测试全部通过。12 个 raw/LZ4 格式组合解码为相同 RGBA；覆盖 DXT1 透明、DXT3/5 alpha、非整块边缘、TEXB1/3/非视频 4、PNG、动画、每个截断位置、损坏长度与 64/256 MiB 分配预算。尚未完成纹理上传和图层 GL 像素验证（2.3）。
- Rust fixture generator 已提供项目、双图层/model/material JSON、GLSL、TEX、PCM WAV 和新目录导出。两次生成/导出逐字节相同，拒绝覆盖目录；无 Workshop 文件。`git check-ignore` 对 Rust fixture 和迁移 runner 均未输出匹配。
- 私有音频 runner 已移至 `scripts/fixtures/scene_audio_monitor.py`，去掉旧 Scene 库环境变量，改用未来的 Rust runtime consumer fixture，并检查恰好执行一项测试；Python 语法检查通过。实际 monitor→Rust Scene 音频验证仍未实现，不计为任务 4.4 通过。
- 开发文档新增 Rust 模块、context/surface owner 和最小首帧命令。私有 Xvfb GL 首帧测试通过（0.26 s），OpenGL 3.0 拒绝测试通过（0.06 s）；新增调用方 tiny unpack PBO/row layout 的成功 resize 和状态恢复验证。测试无旧 Scene 动态库环境变量。
- 本次 Scene 单元测试输出的其他旧 native/gated 测试未显式启用，不能将其零执行通过计为迁移验收。

当前任务为 5/32：1.1、1.2、1.3、1.4、2.2。应用 factory 切换、完整 Scene 对象/脚本以及旧 C++ 删除均仍待完成。


## 2026-10-14 — Rust 资源图与真实纹理画面

- `cargo +1.88.0 test -p wayvid-engine scene:: --locked --offline`：20 项列出通过，其中真正执行的 CPU fixture 覆盖资源图、纹理、shader 展开、生成资产；未显式启用的 GL/旧 ABI 音频测试不可算作集成验证。
- 资源图类型化 image/model/material/pass、对象依赖与保留的动态绑定；验证两个 image 共享单一 model/material/texture，循环/缺失依赖、坏字段、越界路径、symlink 越界、loose/PKGV/shared 和缺失 shared 诊断。先完成 CPU 资源校验再创建 GPU 候选。
- `env -u WAYLAND_DISPLAY XDG_SESSION_TYPE=x11 VARPAPER_TEST_X11=1 xvfb-run -a cargo test -p wayvid-engine scene::runtime::tests:: -- --nocapture --test-threads=1`：3 项实际 GL fixture 通过（0.30 s）。包含 clear/状态/失败 resize、12 种 raw/LZ4 TEX 的双图层真实采样、读取一次与 GPU 共享、30 次坏 vertex/fragment/link 创建释放，以及 loose/PKGV/shared 完整帧逐字节一致。嵌套 `.h` includes（含无扩展名形式）、combo 默认和 float4/texSample2D/saturate 均在 Mesa 上实际编译并采样。
- 新 program/shader owner 失败时释放 GL 对象，分别验证 shader/program/mesh/texture/FBO/EGL 计数回到基线。GL guard 恢复 program/VAO/array buffer、16 个 texture units、active texture、blend/depth/cull、viewport/framebuffer 及 unpack 状态。
- Rust shader expansion 已建立 source/include/宏/combo/预算与 required-module 诊断；source budget、include cycle/depth、坏路径/未知模块、注释边界有 CPU fixture。完整 shader metadata/default texture 与 material uniform 参数仍在实现，2.4/2.5 尚未勾选。
- `cargo clippy -p wayvid-engine --all-targets --locked --offline -- -D warnings` 已通过（1.83 s）；修复新代码 Clippy 项并将旧 audio binding 的手写 NUL 字符串改为适合 Rust 1.88 的 C 字面量。

当前任务 7/32（新增 2.1、2.3）。Rust 仅直接 fixture 支持空场景和基本单 pass image；尚未接入应用 factory。effects、animated texture、text、sound、VM/音频/时间、构建清理与发行验收继续保持未完成，不能将 fixture 结果当作完整 Scene 可用。


### Shader/pass 与独立 Rust 门禁

- `scripts/check-rust-scene.sh` 已实际通过：15 项纯 Rust CPU fixture（明确跳过旧 ABI 与 gated runtime），4 项私有 Xvfb/Mesa GL fixture（0.60 s），1 项 GL 3.0 拒绝 fixture（0.07 s）。脚本清除旧 Scene 库和用户 Wayland/audio 测试环境，检查真实执行项数；不会加载 C++ Scene 动态库、CEF 或用户音频设备。
- shader/pass 新增 reflected uniform 类型/长度、有界元数据、跨 stage combo、默认 sampler 资源、`require`/`requireany` 条件与缺失 active sampler 诊断。实际 GL 验证 float/vec3 显式材质参数和默认值、vec4/mat3/int/bool/float array 默认值、条件纹理两条分支，以及 program 分配失败回收。坏 uniform 在候选阶段失败。
- 兼容记录见 `docs/src/dev/rust-scene.md`，逐项对应生成 fixture，明确单 pass image 的通过范围及 effects/动画/script/audio/session 等待办。构建文档原 C++ 链路仍标为待删除历史路径；尚未修改应用 factory。
- 最新 `cargo +1.88.0 check --workspace --all-targets --locked --offline` 通过（1.70 s）。Rust Scene 单包 Clippy 通过；全工作区 Clippy 后续通过（4.46 s）。纹理 CPU fixture 准确计数为 6 项（其中一项遍历 12 个 raw/LZ4 组合），已纠正前一条记录的计数。

当前进度 10/32：第 1、2 组任务完成；接下来继续图层/效果/动画/文字/声音。完整 Scene、应用接入、旧 C++ 删除、安装包与桌面验收均未完成；未 push。


### 图层迁移进展（3.1 尚未完成）

- 加入 typed camera、auto projection、父图层索引、top-down 坐标、弧度旋转、fullscreen/alignment、logical texture UV 裁剪及 image 颜色/alpha/brightness uniform。GPU 创建前校验几何输入，父查找使用 id 索引，避免逐层扫描全部对象。
- `scripts/check-rust-scene.sh` 最新通过：15 CPU + 5 真实 GL + 1 GL 3.0 拒绝 fixture。新增 GL 图层测试检查非对称位置和旋转矩形、父 scale/origin/visibility、alpha 0.5 混合蓝背景与纹理绿色 padding 裁剪；GPU/program/mesh 计数恢复基线。
- 尚未获得这些新图层案例的参考截图对照，故保持 3.1 未勾选；效果、动画、文本、声音及应用 factory 仍未完成。当前仍为 10/32。


### 参考固定提交真实截图对照 — 语义待确认

- 原参考 checkout 的固定提交 `b016d7d1fdcf4e5fd2f9c9fa420a8aaa07fee02d` 保持未修改。参考程序在 `/tmp/varpaper-scene-reference-src` 与独立 build 目录编译；六个子模块使用已锁定同 revision 的现有快照，argparse `d924b84eba1f0f0adf38b20b7b4829f6f65b6570` 与 MimeTypes `8990d6a796bd9904ec3d41e5c150a1ffb48fe32e` 下载固定提交归档，复用原 CEF SDK。程序不是 VarPaper 的 Scene 动态库。
- Rust fixture 可通过 `VARPAPER_TEST_SCENE_REFERENCE_OUTPUT` 导出相同生成项目和 Rust PNG。外部参考程序在私有 Xvfb 中以 64×64 window 渲染，使用 silent/no-audio-processing/noautomute、dummy SDL、私有不存在的 Pulse socket；未使用桌面音频设备或用户项目。
- 首次原始 PNG 比较未通过：crop 最大差 255 / 超过 2 的像素 4080；parent-hidden 128 / 32；parent-visible 128 / 160；rotation 255 / 144。参考 window 截图的上下方向亦与标准 top-origin Rust PNG 不同，不能将未经方向规范化的统计视为最终像素误差。
- 查证参考 `CImage::getSize` 在纹理存在时返回纹理 real width/height，后续帧忽略输入 object size；`CImage::render` 只检查当前 image.visible，没有继承父 group.visible。父 origin/scale 则实际生效。Rust 当前使用项目 size，并继承父可见性。
- 已请求用户选择遵循项目参数还是严格保持参考这些现有行为；未擅自修改 OpenSpec 对照规则或宣称 reference parity。3.1 保持未勾选，迁移进度 10/32。


### 等待语义选择期间的检查

- 最新 Rust 1.88 全工作区/all-targets/locked/offline check 通过（3.14 s）。fmt --check、工作区 clippy -D warnings（3.10 s）、git diff --check 均通过。
- `cargo test --workspace --locked --offline` 通过：engine 67、library 40、GUI 32，共 139 项。仍需注意其中 gated native/desktop/audio/browser 项未显式启用，不能把总项数视为这些集成路径已验收。
- 独立 Rust Scene 门禁再次实际通过：15 CPU（0.04 s）、5 GL（0.69 s）、1 GL 3.0 拒绝（0.08 s）。测试文件/脚本未被 gitignore 忽略；参考原 checkout 的 git status 仍为空。
- 未执行 Flutter/CEF/三包/桌面全量 quality 验收，未 push。后续依赖语义选择的对照与实现保持等待，不默认把时间流逝当作同意。


## 用户确认后的图层验收

用户确认使用项目 size，并继承祖先 visible=false。设计/任务与原 Scene delta spec 已同步明确。公共案例采用参考项目支持的字符串向量（参考数组 angles 不产生旋转），window PNG 上下方向规范化后，common-rotation、common-parent-alpha 两例 RGB 最大误差均为 0，超过容差 2 的像素均为 0。其余四例分别记录已确认的 size/父可见性差异，不增大容差冒充一致。可重建命令与比较程序为 scripts/fixtures/compare_scene_reference.py；JSON 报告带固定提交与参考二进制 SHA256。参考程序仍仅用于独立验证，无应用构建依赖。3.1 完成，当前 11/32。

## Rust 多 pass 与 effects 验收

迁移进度 12/32。新增真实 GL fixture `rust_scene_multipass_pingpong_parameters_and_failure_cleanup` 和 `rust_scene_effect_files_named_targets_copy_bindings_overrides_and_cleanup`：连续两次 gain、ping-pong FBO、效果 JSON、命名 RGBA FBO、copy、纹理 binding、实例 constant overrides、visible=false、事务性材质参数更新、viewport 恢复以及各十次创建/销毁的 GPU/program/mesh 基线均通过。坏 target、未知 command、零 scale 与注入 allocation 失败不会遗留资源。reference 仅支持的 copy 已实现，其未支持的 swap 会明确拒绝。

`/tmp/varpaper-rust-effects-reference-comparison-v2/comparison.json` 记录相同生成资产：common-effects / common-parent-alpha / common-rotation 的 maximum RGB error 全为 0。参考固定提交和二进制 SHA 与前一节一致；common-effects 同时按项目预期验证整幅图像（中心 4×4，red=32，蓝色背景）。关闭的效果使用有效资产，避免参考 parser 在跳过效果前加载缺失文件的行为干扰截图比较。size/父可见性案例仍按用户确认单独验收。

Rust Scene 门禁通过 15 个 CPU、7 个实际 GL、1 个 GL 3.0 拒绝测试；engine all-targets clippy 无警告。全仓 OpenSpec strict 校验存在四个其他变更的既有 delta 问题，不计为本迁移通过或阻塞，未修改无关变更。应用 factory 与旧 C++ 删除仍未完成，以上不代表完整 Scene 交付。

## 动画纹理与粒子基础（3.3 尚未完成）

动画时钟支持 monotonic 注入，暂停冻结 animation time，恢复边沿重置基线，帧间隔上限 250 ms。TEX 动画选择上传 image，向 shader 提供 atlas translation/rotation；signed/rotated frame rectangle 在分配前验证四角边界。实际 GL fixture 验证两张 image、图集三帧（含旋转）、长暂停/恢复及十次 GPU 基线。

Rust 粒子模块提供有界定义、固定种子 SplitMix64、可注入时间步、基础随机 initializer、movement/angular movement/fade/change，独立 simulation 与 sprite VAO/VBO owner。CPU 测试验证确定性、边界和零步冻结；Rust GL fixture 验证粒子可见/移动、五次相同种子完整帧一致、父 visible=false 隐藏、暂停冻结及 resume、native/GPU/program/mesh/particle owner 回收。动画纹理测试验证连续帧、循环、图集旋转/平移和非法时长拒绝。trail/rope、control points/children、剩余 initializer/operator、音频调制与高级 shader 布局仍由解析器明确报告 unsupported required，并列入 3.6 兼容矩阵；不静默丢弃。

新增 CPU 验证通过：粒子测试 2 项、纹理测试 7 项，包含 unknown renderer/control point/initializer/operator 错误和动画循环边界。沙箱内 Xvfb 不可连接；获准在沙箱外执行后，Rust GL 门禁通过。高级粒子模式仍未迁完，3.3/3.6 保持未完成。

Rust VM 测试 5 项通过，新增 `registerAudioBuffers` 的 16/32/64 注册和 128 次上限、左右声道均值更新、非有限值归零、独立 VM 隔离。原执行/内存/I/O/timer 测试继续通过；pending jobs 和 stack 等完整验收仍待补充，4.2 保持未完成。

4.3 部分实现：Rust runtime 每帧显式消费 AudioSnapshot，缺少新输入归零，向 image/effect/particle shader 绑定 16/32/64 stereo bands 和 RMS uniforms，脚本 Float32Array 原地更新。新增真实 GL fixture 验证非恒定频带下采样、独立左右声道、RMS、缺少新输入/非法输入归零和 GPU cleanup。`scripts/check-rust-scene.sh` 实际通过 24 CPU + 12 GL + 1 GL 版本拒绝 fixture；未启动桌面音频采集。完整 effect/particle/script audio 响应、private monitor 与 session 接入仍待完成，进度保持 15/32。

当前门禁实际通过 18 CPU / 9 GL / 1 GL 3.0 拒绝。engine all-targets clippy 通过；Rust 1.88 workspace/all-targets check 通过（5.63 s）。整体进度仍为 12/32，应用 factory 与旧 C++ 删除尚未完成。

## Rust text/font 验收（3.4）

实际 GL fixture `rust_scene_text_font_fallback_layout_budget_visibility_and_cleanup` 通过：文字 coverage 可见，多行文本，missing font 与默认 fallback 完整帧一致，项目内字体与相同系统字体完整帧一致，动态 text 候选更新、非法更新保留旧 texture、空文字清除、父隐藏、glyph/pointsize/canvas/path/坏字体诊断及十次关闭后的 GPU/program/mesh 基线。字形布局/coverage 使用 ab_glyph，由 Rust 所有，纹理上传复用 Rust TextureUpload。字体 cache 64 MiB、glyph 数量 4096、文本 64 KiB、canvas 4096×4096 和 glyph raster work 有界，Scene 总纹理 cache 256 MiB。3.4 完成，进度 13/32；SceneScript/时钟输入及尚未迁移的粒子/声音仍待完成。

## Rust sound 对象验收（3.5）

`python3 scripts/fixtures/scene_audio_monitor.py --sound` 实际通过（最终 Rust test 2.06 s）：独立临时 PipeWire/Pulse/WirePlumber policy/null sink，只播放 Rust Scene 自有的 375 Hz PCM，检查 decoder samplerate=48000、mpv volume/mute/pause 属性及真实 monitor RMS；调低音量明显衰减，mute/pause/clear 接近零，resume 恢复。三次创建/播放/清除的 stream/GPU/program 与最后 consumer/worker 基线恢复；第二个损坏音频导致 partial sound 创建失败并释放已加载 stream。

Rust sound owner 使用 audio-only libmpv、memfd 资产存储，禁止读取配置/加载 scripts，默认 paused/muted，controls 和关闭生命周期在 engine。once/loop 已验证；最多 64 streams、单音频 64 MiB、共享资源图 256 MiB，未知 mode 明确拒绝。测试体只允许 `varpaper-audio-check-*` 的 private PULSE_SERVER，未操作用户音频设备。volume 的 monitor 衰减遵循 mpv/Pulse 的 volume 行为，没有宣称百分比到 RMS 线性映射。Rust 1.88 workspace/all-targets check 通过，workspace clippy 通过。进度 14/32。

## Rust SceneScript 核心（4.1）

Rust rquickjs runtime/context 按赋值隔离，binding 作为独立 ES module 保留 init/update exports。初始 project properties 可在 init 前提供，后续属性更新进入 engine.userProperties；thisLayer、thisScene.getLayer、Vec2/3/4 和带 seed 的 script property builder 已接入。实际 GL fixture 验证 0.25/0.75 两份 Scene 的不同 opacity 像素，暂停期间属性编辑不推进脚本、恢复首帧更新；脚本从 A 到 BBBB 真实改变文字画面，VM/GPU/program/mesh 回到基线。CPU 验证 module hooks、init 属性、seed、图层向量写入、双 VM 状态隔离和异常回收。4.1 完成，进度 15/32。

另已有 Rust timer scheduler 的 cancellation/reentry/1024 注册预算、32 MiB/512 KiB/20 ms 限制与死循环/内存/I/O 探测；4.2 的完整 pending jobs/audio registration 验证仍未完成，不提前勾选。timer/audio/date 完整 API 矩阵与 private FFT/script 仍待实现，应用工厂仍使用旧 adapter。


## 2026-10-15 — 用户要求先删除 C++、交付已有 Rust 子集

- 删除整个 native/wallpaperengine（自有 adapter/ABI/CMake、上游 C++、专用快照、旧测试和库存校验）、scripts/build-scene.sh，以及本地旧 build/native-scene 输出树。
- scene/mod.rs 仅保留 Rust 模块。renderer factory 直接构造 RustSceneRenderer，用 ProjectResources 和初始项目属性；旧动态库路径查找、ABI 调用和 Scene 音频 consumer 已删除。Scene 不宣传实时系统音频或 GUI 属性控制。
- Scene 资源发现允许应用，详细 required 特性检查在 Rust renderer 创建时执行；Web 保留可选组件要求。X11 资源 fixture 改用 Rust 自生成资产，删除旧库 gating。
- Linux CMake、build-linux/check-native 与 CI 已切换；Web 共用 JSON 快照迁至 native/web/vendor，SHA-256 与原固定值一致。JSON MIT 许可随 helper 安装；保留 GPL 参考来源许可。打包脚本拒绝含旧 Scene 库的脏 bundle。
- 高级功能移至 deferred.md，未完成项没有计为实现完成；Scene delta spec、README、构建与兼容文档同步修订。
- 本轮实际通过：cargo check --workspace --all-targets --locked --offline；cargo clippy --workspace --all-targets --locked --offline -- -D warnings；构建/打包脚本 bash -n；Flatpak/依赖 JSON 解析；迁移 OpenSpec validate。源码检索确认 crates/rust/native/linux/scripts/.github 不再包含旧 Scene ABI/环境变量/构建路径。
- 此前的 Rust CPU/GL 通过记录仍为此前证据。本轮未运行测试、构建发行包或执行真实桌面验收；quality 全部检查未执行，未 push。


## 2026-10-15 — 切换后集成验收

- Rust Scene 门禁实际通过：24 CPU、12 真实 GL、1 OpenGL 3.0 拒绝；全部从 Rust 创建，无 Scene 动态库或参考 checkout。
- X11 资源 fixture 实际验证 loose/PKGV/shared 像素及 missing shared 失败；Scene 首帧提交、坏 shader 候选保留旧赋值和独立媒体继续渲染通过。修正旧 shader 错误断言以检查完整 anyhow chain，并在 Wayland/X11 WallpaperFailed 和 bridge 错误中保留完整原因。
- CEF fixture 的 first frame 改为等待测试页面预期像素，避免初始化透明帧造成假失败；固定软件 Vulkan ICD 指向临时 runtime。初次本地旧 SDK 缺少 libvulkan.so.1，重新下载并校验固定归档后，完整 SDK 的软件渲染通过。
- scripts/check-native.sh 最终实际通过（去除诊断用 strace）：24 CPU + 12 GL + 1 GL3.0、CEF 原生 Canvas/WebGL/音频/隔离/cleanup，以及 engine 82、library 40、GUI bridge 32 项测试。gated 桌面/真实声音测试未启用不能计入。
- 新扩展的 X11 混合 fixture 同时创建 Media/Scene/Web 输出，验证 Scene 首帧、resize、pause/resume/volume 和 StopWeb 后 Media/Scene 继续保留及 GL/audio 资源回到基线。X11 单独 suite 11 项通过（Web gated 在该单独命令未启用；真实三类型 fixture 在 check-native 中启用并通过）。
- cargo llvm-cov --workspace 实际通过并生成 /tmp/varpaper-rust-coverage.info；cargo +1.88.0 check --workspace --all-targets --locked --offline、fmt 和 workspace clippy 实际通过。沙箱中 IPC/Xvfb 失败的检查均在获批后于沙箱外重跑，未将失败记录算作通过。
- 本地 Flutter 3.50.0-0.1.pre 完成 clean、offline pub get、指定源 analyze 和 58 项 test --coverage；版本不同于 CI 的 3.47.4，因此暂不计为完整 quality 门禁通过。未上传 Codecov，未 push。
- 干净 Linux release bundle 已构建，ELF 无旧 Scene 依赖；Debian 与 Arch 包实际构建并检查文件清单，含 Rust bridge、Web helper、JSON MIT 和研究 GPL 许可，无旧 Scene 库/库存与 CEF runtime。SDL2、DejaVu 字体依赖补齐；Flatpak 同时检查 helper 的底层库依赖。三种安装环境 Scene smoke/真实 Wayland 与整体发行验收仍待完成。
- 3.6、5.1、5.2、5.4、5.6 完成；父变更按 Rust 实际证据勾选 2.1/2.2/2.4/2.6/3.1/3.2/3.5/3.6。高级粒子、完整脚本/实时音频/本地时钟条目仍未勾选；不宣称整个五类型交付完成。


### 精确 Flutter 版本、三种包与包内 Scene smoke

- 只读复用本机 Flutter git 对象，在 /tmp/varpaper-flutter-ci 建立独立 3.47.4 checkout，下载其固定 engine/Dart 缓存；未切换用户原 Flutter checkout。使用 3.47.4 完成 offline pub get、指定源和新增 fixture 的 analyze（无问题）、58 项 test --coverage（全通过，覆盖率 /tmp/varpaper-flutter-coverage.info）。此前 clean 在本地 SDK 执行；CI 上传/完整远端 quality 尚未执行，6.3 继续未完成。
- 原生宿主构建的 Rust bridge 引用 GLIBC_2.44，而 GNOME 50 提供 2.42，首次 Flatpak 导出后链接检查明确失败。新增下载前 glibc 检查，已实际验证清晰失败。未把该失败算作成功。
- 在现有 maxxing/compiler-dev 容器（glibc 2.39）安装开发库，以只读源树/registry 和 Rust 1.88 重新编译同一 Rust bridge，max GLIBC 要求降至 2.39。复制 Linux bundle 至 build/flatpak-compatible-bundle，仅替换其中 Rust bridge；以 VARPAPER_BUNDLE_DIR 指定该兼容 bundle，Flatpak 重建、GNOME 运行时 ldd/FFmpeg ABI 检查和导出全部通过。无复制 host libc 的绕过。
- 实际产物：dist/varpaper_20261014.gf2f0b356_amd64.deb、dist/varpaper-20261014.gf2f0b356-1-x86_64.pkg.tar.zst、dist/varpaper_20261014.gf2f0b356_x86_64.flatpak（20,765,688 bytes）。Debian/Arch 文件清单无旧 Scene 库、快照库存和 CEF runtime；Flatpak 正常导出且 source bundle 同样无旧 Scene。
- 新 scripts/check-scene-package.sh 支持三种包，提取到私有临时目录（Flatpak 使用独立 OSTree 仓库，不安装到用户系统），从真实包内加载 Rust .so，经 FRB 启动 X11 engine。三个包均实际通过：Scene 首帧提交、pause/resume/config、坏模型资产的同步 apply_failed 和异步 wallpaper_failed 诊断、清除事件以及 engine shutdown。测试匹配规范化 project.json 路径；monitor discovery 的 currentWallpaper 不是赋值持久化接口，未将其作测试契约。
- scripts/fixtures/scene_package_smoke_test.dart 生成自有资产；包装脚本只在私有 Xvfb 启用。当前 smoke 使用宿主系统库，不等于三种系统内安装验收；Flatpak 另有 GNOME SDK 内动态链接检查。真实 Wayland/安装环境验收仍在 6.1/6.2。
- CI 的三个 package verification 步骤均接入该脚本，增加 ostree/Xvfb 工具与显式 fallback 字体依赖；质量检查增加新脚本语法与 Dart fixture analyze。NOTICE/许可继续保留真实参考来源。
- 5.5 完成，当前 23/27。6.1、6.2、6.3、6.4 仍未完成；未 push，未误报整体变更完成。

- 补充检查：从最终 Flatpak 提取 Web helper，在 GNOME 50 runtime 下实际执行 ldd；X11/SDL2/C++/libc 依赖均解析，只有按可选组件策略排除的 libcef.so 未安装。Flatpak 构建脚本同步加入 helper 检查，并只允许这一项可选缺失。

### 100 次跨类型替换与失败候选回收

- 扩展 X11 的 Media/Scene/Web 混合 fixture，执行 100 轮 Media → Rust Scene → Web，每次赋值均等待首帧提交，随后拒绝一个无效 Scene 候选并核对原路径保持、没有提交事件；每种类型执行 pause/resume 和 resize。累计 300 次成功赋值和 300 次失败候选拒绝。
- 每轮销毁 session 后检查 EGL context/surface、Scene texture/FBO、shader/program、particle、script、sound 和共享音频服务计数恢复基线；共享 Web host 不再有强引用，其 Drop 执行退出/终止并回收子进程。生成 Scene 为 image fixture，未包含脚本或声音对象，这些计数只验证没有意外创建；具体脚本/声音所有权仍由各自专用 fixture 证明。
- 沙箱内首次运行无法连接 Xvfb，未计为通过。获准在沙箱外重跑 scripts/check-native.sh 后整体成功：24 CPU + 12 真实 GL + 1 GL3.0 拒绝、真实 CEF fixture，以及 engine 82、library 40、bridge 32 项；engine suite 耗时 175.75 s。日志 /tmp/varpaper-native-100-cycles.log。
- 新增只在测试编译的 Scene 资源汇总与 Web host 存活查询。fmt --check、workspace/all-targets clippy -D warnings、Rust 1.88 workspace/all-targets locked/offline check、git diff --check 均通过。构建文档中仍声称工厂使用旧 adapter 的过期句子已修正。
- 此结果仍是私有 Xvfb 验收，不覆盖真实 Wayland/native X11 的 hotplug/restart 或三种系统的安装环境；6.1/6.2 保持未勾选。完整远端 quality/coverage 上传及最终衔接仍待完成，当前 23/27，未 push。

### 嵌套 niri 的真实 Wayland/EGL 集成

- 新增 scripts/check-wayland.sh 和 scripts/fixtures/wayland_session.py。在私有 Xvfb 中运行 niri 26.04（8ed0da4），使用独立 mode=0700 的 XDG_RUNTIME_DIR、空配置、软件 GL 和 dummy SDL；不使用 --session、不导入全局环境，也不连接用户桌面的 Wayland socket。退出时回收 niri 与临时目录，测试 subprocess 设 300 s 上限。
- 首次沙箱内运行因 XOpenDisplayFailed 失败；获批在沙箱外重跑后成功。原有 backend 三项实际通过（63.34 s），包含输出发现和 20 次 engine 重启 × 5 次媒体 apply/clear，不把 gated 跳过算作执行。
- 新增 nested_wayland_scene_media_failure_clear_and_restart，仅在隔离 runner 的显式开关下运行。生成 Rust image Scene，三次完整 engine 生命周期各执行 Media → Scene → Media → Scene：逐次等待首帧提交，pause/resume，提交无效 Scene 并等待 WallpaperFailed，再查询 GetStatus 确认有效赋值仍在；clear 后 EGL context/surface 为 (1,0)，随后 shutdown/join。
- 新 runner 最终实际通过 backend 全部 4 项（70.19 s）；日志 /tmp/varpaper-wayland-scene.log。此处执行的是实际 niri layer-shell、Wayland EGL 和 Rust renderer 路径，不是仅协议 mock，也不等于物理显示器的热插拔验收。
- workspace/all-targets clippy -D warnings、Rust 1.88 locked/offline check、fmt、shell/Python 语法和 diff 检查通过。CI 脚本语法门禁已覆盖新增 shell 脚本；CI 未安装 niri，也未宣称远端执行了这项可选验收。
- 6.1 的 Wayland Scene/media 首帧、失败保留、clear 和 engine restart 已补充；混合多输出/Web、物理 hotplug、resize/截图以及三种安装环境仍待验收。6.1–6.4 继续未勾选，进度 23/27，未 push。
