# Tasks

2026-10-15 按用户要求收窄当前交付；高级功能移至 deferred.md，那里未完成项不计为已实现。Scene 功能契约沿用 `add-all-wallpaper-types` 的 Scene/project/runtime delta specs；历史 C++ 测试不计为 Rust 验收。各项实现、测试与文档同步交付，下面的最终验收只检查跨模块集成。

## 1. Rust 核心与测试资产

- [x] 1.1 建立 engine 内 scene/ 模块、类型化 Scene 输入及资源读取接口，复用 ProjectResources；锁定所需 Rust/第三方 binding 依赖并核对 Rust 1.88/许可，用 Cargo check 和依赖/MSRV 检查验证，不导入上游 C++ 类。
- [x] 1.2 将旧 C ABI fixture 的生成 TEX/JSON/GLSL/PCM 资产及预期迁为 Rust fixture generator，并迁出私有音频 Python runner；验证同资产可重建、无 Workshop 专有文件且测试文件未被 gitignore 漏掉。
- [x] 1.3 实现 Rust 最小 clear-color Scene 与 render-thread/current-context GPU owner；真实 GL 验证首帧像素、缺失 OpenGL 3.3、partial 初始化失败与重复释放计数，执行测试不得加载 Scene 自有动态库。
- [x] 1.4 更新开发文档的 Rust 模块、资源/context 所有权和最小首帧命令，实际执行示例验证。

## 2. 资源、纹理、材质和 shader

- [x] 2.1 实现 Rust Scene/model/material JSON 解析、依赖关系/循环与 required 特性诊断，复用 loose/PKGV/shared 资源读取；以完整、循环、缺失 shared、路径越界和坏字段 fixture 验证资产级错误及失败回收。
- [x] 2.2 实现 TEXV/TEXI/TEXB 和 RGBA/R/RG/DXT1/3/5 的 raw/LZ4 解码与有界分配；迁移格式边界测试，验证截断、非法维度、解压长度和总预算均在分配前拒绝。
- [x] 2.3 实现 context 所有的纹理上传/缓存，用 12 种格式实际 GL 像素和双图层只读一次 fixture 验证；逐字节对照 loose/PKGV/shared 相同画面及 native/GPU 回收。
- [x] 2.4 用 Rust 实现 shader 源定位、include/宏/combo 和 GLSL 转换，处理依赖循环/缺失及资源预算；以参考 shader fixture 验证展开结果和真实 GL 编译，不使用复制的 C++ GLSLContext。
- [x] 2.5 实现 Rust shader/program/材质 pass、uniform 和 FBO owner，恢复 framebuffer/viewport/GL state；用无效 vertex/fragment、link/driver 失败和成功像素 fixture 验证完整诊断及 GPU 基线。
- [x] 2.6 文档记录已验证纹理/shader 支持、预算和失败行为，逐项关联新 Rust fixture 并核对仍未通过项不能宣称兼容。

## 3. 图层、效果、动画与声音

- [x] 3.1 实现 Rust image/camera/layer transforms、裁剪和混合；生成固定图层 fixture，采用项目显式 size，祖先隐藏时隐藏子树；对照参考同尺寸/资产/时间的截图（统一图片方向），无已确认语义差异的案例容差 ≤2，size/父可见性差异单独记录并验证项目参数预期。
- [x] 3.2 实现多 pass effects 和材质参数更新；固定效果 fixture 验证 pass 顺序、ping-pong FBO、输出像素/GL 状态及失败 cleanup，对照参考画面。
- [x] 3.4 实现 Rust text/font 布局、栅格和 fallback；真实 GL 验证文字可见、缺失字体 fallback、有界文本分配及清除资源回收。
- [x] 3.5 实现 Rust sound 对象的解码/播放控制与 stream/device/worker owner；通过已知 PCM 验证实际 volume/mute、pause/resume、清除停止声音和 partial sound 加载失败回收。
- [x] 3.6 按已实现 Rust 子集更新图形/声音兼容矩阵和生成 fixture 命令，验证全部声明对应新 Rust 通过结果；未知 required 模式必须有清晰错误，不能静默忽略。

## 4. SceneScript、音频与时间

- [x] 4.1 接入 JavaScript VM Rust binding，建立每赋值独立 runtime/context 与 init/update/project property/layer APIs；用属性动画、双 Scene 隔离及脚本异常 fixture 验证，移除 C++ ScriptEngine/EngineObject 语义封装。
- [x] 4.6 文档区分已实现基础宿主/动画时钟、内部音频快照接口与后续完整脚本/实时音频/本地时钟能力，不将后续设计宣传为当前支持。

## 5. session 切换与旧链路删除

- [x] 5.1 让 renderer factory 直接创建 Rust Scene，接入 Wayland/X11 事件、现有 stretch/resize/fps、pause/mute 与初始 property 输入（其他布局和实时属性编辑移入后续）；用媒体/Scene/Web 混合输出验证首帧提交、失败候选保留旧路径与其他输出继续运行。
- [x] 5.2 替换原动态库依赖的 Rust Scene/X11/lifecycle fixture，移除 Scene library path 查找和所有 vp_scene_* 加载/调用；在未设置 Scene 环境变量且无独立 Scene 库时执行完整 Rust 集成测试。
- [x] 5.3 删除 native/wallpaperengine 自有 C++ Scene、adapter、ABI、专用 vendored 快照/库存验证及 scripts/build-scene.sh；用架构检查确认已删除路径且执行代码不再含旧库/ABI 依赖，生成 fixture/音频 runner 已迁出。
- [x] 5.4 移除 Flutter/Linux CMake、build-linux/check-native、Cargo 集成和 CI 中的 Scene CMake/library/manifest 安装与调用；新增 Rust Scene fixture 门禁并保留 CEF/media 检查，核对工作流不引用已删除脚本且真正运行新测试。
- [x] 5.5 修正 .deb/Flatpak/Arch 的 Scene 依赖和产物清单，按实际使用保留 Web/mpv/第三方依赖；构建各包并检查不带旧 Scene 库/快照，动态链接和实际 Scene smoke test 通过。
- [x] 5.6 更新 NOTICE、README/构建/兼容文档和旧 Scene 许可分发规则，保留实际参考来源和第三方许可证；检查文档命令、许可清单及历史 C++ 证据与 Rust 完成状态明确区分。

## 6. 集成验收与变更衔接

- [ ] 6.1 在支持的 Wayland/native X11 环境执行 Rust Scene 与媒体/Web 混合输出、resize、pause、clear、hotplug/restart；记录环境/截图/失败项，执行至少 100 次跨类型/失败创建循环验证 GPU/script/audio/browser/native 计数无持续增长。
- [ ] 6.2 从干净输出目录构建 release bundle，检查 ELF/文件清单无自有 Scene 库、ABI、参考绝对路径或未解析依赖；在无参考 checkout 的三种安装包环境验证 Scene 可用、Web 默认无 CEF/无隐式下载，保存项目/属性不被迁移删除。
- [ ] 6.3 执行 .github/workflows/ci.yml quality 全部检查，包括 fmt/clippy/llvm-cov、脚本/manifest、Flutter clean/pub get/analyze/test coverage 和新的 Rust Scene/CEF fixture；修复并重跑失败项，条件触发的 coverage 上传亦成功，每次 push 前保持门禁。
- [ ] 6.4 将通过的 Rust fixture/构建/验收证据对应到 add-all-wallpaper-types 的相关 Scene 任务；核对只勾选完全满足原描述的项目，并验证本迁移的任务全部完成后仍不误报原变更整体完成。
