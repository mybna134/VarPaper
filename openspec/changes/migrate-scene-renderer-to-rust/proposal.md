# Proposal

## 2026-10-15 用户调整后的当前范围

用户要求先删除 WallpaperEngine 的所有 C++ 代码，并先移除 Rust 尚未实现的功能，后续再实现。本节取代此前“全功能完成后才能删除”的顺序与范围要求。

当前交付为现有 Rust Scene 子集及旧链路删除：资源/package/shared、纹理/shader、图层和多 pass 效果、动画纹理、基础 sprite 粒子、文字、once/loop 声音、基础 SceneScript 与单调动画时钟。项目初始属性可读，GUI 实时属性编辑仍不提供。Scene 当前采用 stretch 渲染，不承诺其他布局。

高级粒子（rope/trail、控制点、子粒子及缺失 initializer/operator）、完整脚本 API/预算验收、系统 playback monitor→Scene 可视化、音频粒子调制、可注入 wall-clock/timezone 与实时属性界面移出当前交付，保留在 deferred.md。现有内部 AudioSnapshot/typed-array 接口保留，应用 Scene 不启动音频采集，不宣称实时音频响应。未知必需对象/格式继续明确报错；不静默删除项目必需内容，不删除用户项目或保存赋值。

发行包、真实桌面及 quality 验收仍按实际执行记录，未执行项目不能勾选；当前删除工作不宣称整个五类型变更完成。

## Why

现有 Scene 路径在 Rust engine 中动态加载自有 C++ Scene 库，纹理、材质、shader、效果和脚本实际由移植的 linux-wallpaperengine 子系统实现，与用户要求的 Rust engine 实现不符。按当前用户确认的 Rust 子集范围，将逻辑迁移到 Rust 并删除旧实现和构建链路，统一渲染、错误和资源所有权。

## What Changes

- 在 `crates/wayvid-engine` 中用 Rust 实现 Scene 模型、纹理缓存、材质/pass、shader 转换、图层/效果/粒子、文字/声音控制、SceneScript 宿主、音频与时钟绑定。
- 复用现有 Rust 项目/package/shared 资源定位、共同 renderer/session、EGL、共享 playback monitor/FFT；保持首帧成功后提交、失败保留旧输出与 current-context 清理规则。
- 删除 `native/wallpaperengine` 的 C++ Scene 源、兼容 adapter、独立库、C ABI、专用 vendored 快照和库存校验，移除 Rust 动态加载及 Cargo/Flutter/CMake/scripts/CI/打包中的旧依赖。
- 将生成资产、像素对照、恶意格式、脚本预算及资源回收验证迁为 Rust fixture；既有 C++ 通过记录仅保留为历史证据，不能计为 Rust 实现通过。
- 保留 Video/Image/GIF 的 mpv 行为和可选 Chromium/CEF Web 宿主、官方下载安装及卸载策略；本变更专门替换 Scene，不能删除仍被其他功能使用的原生库或构建工具。
- **BREAKING（内部集成）**：移除 `vp_scene_*`、自有 `libvarpaper_scene.so` 和 Scene 库路径环境变量；壁纸项目格式、用户源路径、属性及保存赋值不因内部实现迁移而重置。

## Capabilities

### New Capabilities

无。当前行为范围修订在 `add-all-wallpaper-types` 的 delta specs 中维护，本目录 `skip_specs: true`，不新建重复规格。

### Modified Capabilities

按用户新范围同步收窄 `add-all-wallpaper-types` 的 `scene-wallpaper-rendering`、`wallpaper-project-loading`、`wallpaper-runtime` Scene 行为契约。尚未实现的 Scene 功能移入后续待办，不以本迁移宣布全部五类型支持完成。

## Impact

- `crates/wayvid-engine/src/scene.rs`、新增 `scene/` 模块、`renderer.rs`、session 与 X11/Wayland fixture：替换 FFI Scene owner，Rust 实现和测试。
- `crates/wayvid-library` 项目资源接口：复用边界与校验，必要适配不得改变稳定身份或安全预算。
- `native/wallpaperengine/`、`scripts/build-scene.sh`、`scripts/build-linux.sh`、`scripts/check-native.sh`、`linux/CMakeLists.txt`、CI、三种安装包配方：删除旧 Scene 构建/装载/安装/校验并接入 Rust 检查。
- engine Cargo 依赖与锁文件、NOTICE、开发文档：核对 Rust 依赖、底层 FFI 许可、参考来源和实际发布清单。
- 与 `add-all-wallpaper-types` 协作：本变更拥有 Scene 迁移与旧链路清理；原变更继续拥有五类型界面/FRB/持久化、Web 与综合交付。相关任务只按新 Rust 证据勾选。
