# Design

## 2026-10-15 用户调整后的当前范围

用户要求先删除 WallpaperEngine 的所有 C++ 代码，并先移除 Rust 尚未实现的功能，后续再实现。本节取代此前“全功能完成后才能删除”的顺序与范围要求。

当前交付为现有 Rust Scene 子集及旧链路删除：资源/package/shared、纹理/shader、图层和多 pass 效果、动画纹理、基础 sprite 粒子、文字、once/loop 声音、基础 SceneScript 与单调动画时钟。项目初始属性可读，GUI 实时属性编辑仍不提供。Scene 当前采用 stretch 渲染，不承诺其他布局。

高级粒子（rope/trail、控制点、子粒子及缺失 initializer/operator）、完整脚本 API/预算验收、系统 playback monitor→Scene 可视化、音频粒子调制、可注入 wall-clock/timezone 与实时属性界面移出当前交付，保留在 deferred.md。现有内部 AudioSnapshot/typed-array 接口保留，应用 Scene 不启动音频采集，不宣称实时音频响应。未知必需对象/格式继续明确报错；不静默删除项目必需内容，不删除用户项目或保存赋值。

发行包、真实桌面及 quality 验收仍按实际执行记录，未执行项目不能勾选；当前删除工作不宣称整个五类型变更完成。

## Context

动机与范围见 proposal.md。现有 `crates/wayvid-engine/src/scene.rs` 是 libloading/C ABI owner，`renderer.rs` 查找 `libvarpaper_scene.so` 并把 EGL、资源回调与音频输入传给 C++。`native/wallpaperengine` 包含适配的上游 Scene、QuickJS/GLSL 编译依赖快照、CMake、ABI 测试；Flutter CMake、Linux 构建脚本与 CI 原生脚本均引用它。engine 已有 Rust GL/EGL、renderer/session、共享 monitor/FFT；library 已实现有预算的 project/package/shared 资源定位。

研究参考固定为 linux-wallpaperengine `b016d7d1fdcf4e5fd2f9c9fa420a8aaa07fee02d`。只读研究其格式、GLSL 宏/材质及脚本语义，不编辑参考 checkout，不将其路径作为构建或运行依赖。参考实现仍有 TODO，迁移不能将旧静默忽略视为已支持。

`add-all-wallpaper-types` 已调整为 Rust Scene，但功能矩阵尚未完成。本迁移提供其 Scene 实现与清理证据；界面、FRB、持久化的新功能及五类型综合交付仍由原变更追踪。旧 C++ ABI/像素测试是可复用预期和历史记录，不证明 Rust 实现完成。本变更按用户新范围更新 add-all-wallpaper-types 的 Scene delta spec，本目录沿用 `skip_specs: true`，验收以修订后的 Scene/project/runtime delta specs 为准。

## Goals / Non-Goals

**Goals:**
- Scene 业务逻辑由 Rust engine 所有，调用路径不再经过自有 C++ Scene 库。
- 交付已实现并验证的对象/格式、效果、基础脚本属性和单调时间范围，逐项验证并给出真实兼容诊断。
- 保持 EGL/output/session 所有权、首帧事务、跨赋值隔离和释放顺序。
- 完整删除旧实现、装载、构建、安装、资产库存与失效文档，独立构建可复现。

**Non-Goals:**
- 重写 mpv、EGL、Chromium/CEF 或 JavaScript VM 等第三方底层实现。
- 改变 Web 可选安装策略、用户持久化格式，或扩展 Application/Windows 专有 API 支持。
- 保留 C++ fallback 或复制 C++ 后套 Rust FFI。

## Decisions

### 1. engine 内的 Rust Scene 模块

将 `scene.rs` 替换为 `scene/mod.rs` 及按解析/model、texture/cache、shader、material/pass、objects/effects、script、clock 分层的 Rust 模块。输入为项目描述/资源读取 trait、EGL render context、输出尺寸/FBO、属性与音频快照；直接实现共同 `WallpaperRenderer` 契约。library 继续负责目录/package/shared 的边界和有界读取，engine 负责资源的语义解析与渲染。

CPU 不可变资产可缓存；GPU 句柄受 context/thread owner 约束，禁止跨 context 分享可变 GPU 状态。脚本、属性、随机状态、时钟及播放控制均按赋值隔离。避免把 engine 变成上游 application context 的模拟器，继续使用当前 VarPaper 输出调度。

### 2. 纹理、材质、shader 与对象

Rust 解析 TEXV/TEXI/TEXB、RGBA/R/RG/DXT1/3/5、LZ4 和动画帧，明确长度、维度、帧数、解压与总缓存预算。缓存只在成功解析/上传后提交。GL allocation/program 编译链接采用 Rust owner 和显式 current-context close；失败路径释放临时 shader/program/texture/FBO/VAO/buffer。

Rust 负责 GLSL 源定位、include/宏/combo、uniform/材质参数、语法转换与 pass 组织。OpenGL driver 负责最终编译/链接；如需第三方编译工具，通过受控 binding 使用，不能保留上游 CPass/GLSLContext 的 C++ 实现。保留失败日志及资产/pass 名称，不能以字符串替换掩盖不支持语法。固定 fixture 与参考同资产/尺寸/时间的帧对照，差异须有明确容差与原因。

Rust 对象层实现相机、变换、混合/裁剪、多 pass 效果、animated texture、可注入随机种子的粒子、文字/字体 fallback 和声音控制。解码、字体、音频设备可通过第三方库 Rust binding；加载、流/线程、静音/音量与关闭策略由 engine 所有。未知 required 对象、shader 或脚本 API 应在候选阶段产生兼容错误。

用户已明确确认图层语义：显式 `size` 决定渲染尺寸，随后应用 `scale`；缺省尺寸才回退到纹理/model。祖先 `visible: false` 隐藏整个子树。参考固定提交使用纹理尺寸覆盖显式 size、只检查子层自身 visibility 的行为不作为 Rust 兼容目标。截图比较统一为 top-origin RGB；无这些差异的生成案例保持逐像素容差 ≤2，受这两项已确认语义影响的案例单独记录差异并用项目参数的固定预期验收，不以增大容差掩盖差异。

### 3. Rust SceneScript 宿主

使用 JavaScript VM 的 Rust binding（可沿用 QuickJS 底层 VM），删除复制的 C++ EngineObject/ScriptEngine。VM 安装的属性、layer、timer、audio 和 clock APIs 在 Rust 定义并显式限定宿主访问；不开放文件系统、shell、网络和通用模块加载。

每赋值独立 runtime/context，所有回调仅在 render thread 执行。初始预算沿用已测试的 32 MiB VM 内存、512 KiB stack、20 ms 执行期限，128 次 audio buffer registration 与 1024 个 timer 上限；异常、中断和 pending jobs 都纳入失败与释放检查。用户已确认将编译基线统一为 Rust 1.88：现有 image 0.25.10 已要求该版本，旧 manifest 的 1.75 声明不再成立。统一 workspace/crate 声明并增加 CI minimum-toolchain check；Rust 依赖选型及锁定版本核对 1.88 与许可证，不能隐式继续提高 MSRV。

支持 init/update、项目属性与已声明 layer binding；`registerAudioBuffers(16/32/64)` 返回稳定 Float32Array left/right/average，逐帧更新且按赋值隔离。timer 使用赋值 animation clock，暂停停止 tick/声音；后续 Date/local-time 将从可注入系统时钟获得，恢复首帧取当前日期且 delta 不含整个暂停时长。

### 4. 后续设计：直接复用 Rust 音频服务

Scene 从现有 AudioConsumer 得到 64 左 + 64 右频带及双声道 RMS，Rust 计算 32/16 下采样并直接绑定 shader/效果/粒子和脚本。`g_AudioSpectrum{16,32,64}{Left,Right}` 保留独立声道；已有 RMS 扩展保持明确命名与文档。消除 `vp_scene_set_spectrum` 中转，缺失/过期输入归零并产生非致命状态。

壁纸生成声音 mute/volume 与 playback monitor 独立，最后一个 consumer 关闭释放采集。移植私有 PipeWire/null sink 测试 runner 至 Rust fixture 目录；真实音频测试必须显式开启并只连接临时音频服务，不操作用户设备。

### 5. 清理与构建切换

按用户新顺序，先删除旧 C++ 和构建链路，再直接接入已有 Rust 子集。开发期间 C++ 资产只作为对照；本次立即删除整个自有 Scene C++ 子系统和独立库，不保留隐藏 fallback 或可选编译开关。

删除 Scene-specific CMake/Flutter 安装项、`build-scene.sh`、库路径环境变量、库存校验与仅被其使用的快照/许可安装规则；修改 native check 直接执行 Rust GL fixture，保留 CEF 原生测试。底层第三方依赖和 CEF 所需 CMake/C++/SDL 等按实际使用保留。清理 NOTICE 中已删除的分发说明，保留研究/移植相关归属和仍使用的第三方许可，不能一概删除法律来源。

检查代码、脚本、CI、发行包和运行 ELF 均无旧 Scene 库/symbol/path 依赖；历史 OpenSpec 验证记录可明确标注为历史，不应机械删除。已有 build 目录可能含旧库，验收必须使用全新输出目录/干净 bundle，确认打包脚本不会误带旧产物。

## Risks / Trade-offs

- [格式/shader 语义差异] → 生成可再分发 fixture，逐项参考帧对照，错误不能降级为空白成功。
- [VM binding/MSRV/底层依赖变化] → 锁定兼容版本、核对 Cargo/MSRV/许可，在干净环境验证；性能和错误边界采用真实 fixture。
- [partial GPU/音频/脚本初始化泄漏] → 错误注入、线程/current-context 检查、至少 100 次生命周期循环及资源基线。
- [删除依赖误伤 Web/mpv] → 按实际使用删除 Scene 专用项，保持真实 CEF 和媒体回归。
- [原变更与迁移重复计数] → 原变更任务只有获得 Rust 实现的通过证据才勾选；本变更完成不等于整个五类型交付完成。

## Migration Plan

1. 将旧生成资产、坏输入、预期像素和资源计数迁为 Rust fixture；整理旧依赖引用与可复用 Rust 接口。
2. 实现 Rust Scene 核心及首帧，完成资源/纹理/shader/pass/object、脚本、音频/时钟模块与测试，切换 renderer/session。
3. 立即删除 C++ Scene、库装载与专用构建/包清单，更新开发文档/NOTICE/CI。
4. 用干净 bundle、三种安装包和 quality 检查验收；迁移证据关联原变更的 Scene 任务，其其他任务保持独立。
5. 回滚使用版本控制中的上一完整应用版本，不提供运行时 C++ fallback；任何迁移/回滚均不删除用户项目、共享资产或已保存赋值。
