# Validation

## Task 1.1 — 2026-10-16 覆盖率基线

- 源码提交：`092f1e1c375d2bb8ab1e251caa452520189de818`。开始时唯一未跟踪项为本 change 目录，业务代码工作区干净。
- 执行时间（UTC）：`2026-10-16T12:31:32Z`；平台 Linux 7.2.8-arch1-2，x86_64。
- 工具链：Rust/Cargo 1.97.1，cargo-llvm-cov 0.9.1；Flutter 3.47.4（9584c6713b），Dart 3.13.3。项目本地 Puro 环境为 varamusic，其版本与 workflow 固定的 Flutter 3.47.4 一致。
- Rust 命令设置 `RUSTFLAGS=-D warnings`，与 quality 一致；只把输出报告路径改到临时目录。未启用任何 VARPAPER_TEST_* opt-in。

```sh
RUSTFLAGS='-D warnings' cargo llvm-cov --workspace --ignore-filename-regex frb_generated --lcov --output-path /tmp/varpaper-cleanup-baseline-hyewzuit/lcov-rust.info
puro flutter test --coverage
```

- 两条命令退出码均为 0。Rust engine 88、library 40、service 39 项测试返回成功；Flutter 86 项通过。未运行本轮无关的 clean、Clippy 或完整 quality，不据此声明 push 门禁通过。
- Rust 以上 167 项中，源码检查确认有 39 项在当前无 opt-in 环境下提前返回；它们不构成真实显示/GPU/音频/CEF 验收。普通 Wayland 内存协议 fixture 实际执行，无需连接桌面。具体测试名与 gate 保存在 coverage-baseline.json 的 test_results.rust_opt_in_early_returns。
- Flutter 命令默认只执行 test/，scripts/fixtures 下两个 package opt-in fixture 未被选择。native helper Python fixture 也不属于此次命令执行范围。

### 统计口径

- 原始值：按 LCOV 的 SF/DA 统计，以仓库相对路径及行号去重，同一行计数相加，计数大于零为覆盖。只统计报告实际包含的文件，不把报告未收集的文件当作已覆盖。Rust 报告包含 llvm-cov 发出的测试代码；生成 FRB 文件已由原有命令排除。
- 过滤后值：在同一份报告上应用现有 codecov.yml ignore，用 Python fnmatchcase 匹配仓库相对路径。没有新增排除；这是本地行覆盖率比较视图，不等同于远程 Codecov 对 region/branch/partial line 的处理。
- 两个 flag 分别记录；不使用 carryforward 历史报告，也未上传或查询远程 Codecov。用户报告的约 64% 尚未验证其提交与指标口径。
- 各 flag 按未覆盖行数降序列出前十个非排除热点；不足十个时列出全部。

| Flag | 口径 | 覆盖行 / 总行 | 未覆盖行 | 行覆盖率 |
| --- | --- | ---: | ---: | ---: |
| rust | 原始 | 7265 / 14941 | 7676 | 48.62% |
| rust | 现有过滤后 | 6574 / 10854 | 4280 | 60.57% |
| flutter | 原始 | 1512 / 2656 | 1144 | 56.93% |
| flutter | 现有过滤后 | 1475 / 1525 | 50 | 96.72% |

### rust 热点

| 文件 | 覆盖 / 总行 | 未覆盖 |
| --- | ---: | ---: |
| `crates/wayvid-engine/src/web.rs` | 152 / 594 | 442 |
| `crates/wayvid-engine/src/scene/draw.rs` | 0 / 431 | 431 |
| `crates/wayvid-engine/src/scene/assets.rs` | 466 / 823 | 357 |
| `crates/wayvid-engine/src/scene/runtime.rs` | 0 / 348 | 348 |
| `crates/wayvid-engine/src/scene/program.rs` | 0 / 283 | 283 |
| `crates/wayvid-engine/src/scene/gpu.rs` | 0 / 266 | 266 |
| `rust/src/bridge.rs` | 643 / 907 | 264 |
| `crates/wayvid-engine/src/scene/text.rs` | 0 / 231 | 231 |
| `crates/wayvid-engine/src/scene/sound.rs` | 4 / 221 | 217 |
| `rust/src/engine.rs` | 115 / 303 | 188 |

### flutter 热点

| 文件 | 覆盖 / 总行 | 未覆盖 |
| --- | ---: | ---: |
| `lib/main.dart` | 1206 / 1238 | 32 |
| `lib/src/storage/settings_store.dart` | 219 / 237 | 18 |
| `lib/src/l10n.dart` | 11 / 11 | 0 |
| `lib/src/preview_cache.dart` | 39 / 39 | 0 |

### 结果与限制

- 过滤后 Flutter 已达 96.72%，仅 50 行未覆盖；Rust 为 60.57%，未覆盖热点主要集中在 Web 与 Scene GPU 路径。draw/runtime/program/gpu/text 的零覆盖来自未启用图形执行路径，不能据此认定其无用。
- 这表明清理应以实际调用关系为依据；仅删除 Wayland 遗留代码不会直接改善现有过滤后的覆盖率，因为 wayland/** 本已被排除。
- 原始日志、两份 LCOV 与完整临时汇总位于 `/tmp/varpaper-cleanup-baseline-hyewzuit/`，可能随系统清理丢失。提交的 coverage-baseline.json 保存基线元信息、报告 SHA-256、配置/lockfile SHA-256、所有文件计数和环境测试列表，供后续重跑比较。
- 新生成的 Flutter lcov 已复制到上述目录；仓库原有受跟踪 coverage/lcov.info 还原为本轮开始时的 HEAD 版本，不提交生成的覆盖率产物。
- 同一工具链与锁文件下重跑同一命令，再按上述 SF/DA 和原有 ignore 汇总即可比较；异步/系统相关行可能有小幅波动，应记录并解释。

## Task 1.2 — 调用与契约证据

本轮仅作审计，以下删除/合并结论供后续 Task 实施。所有权字段不因 dead_code 标注或低覆盖率而删除。搜索覆盖 crates、rust、lib、test、scripts、examples、docs、packaging、README，并单独阅读现有 change 的契约；生成代码只视作生成导出引用，不能据此认定生产调用。

| 候选 | 定义和调用证据 | 契约/生命周期证据 | 结论与后续验证 |
| --- | --- | --- | --- |
| `WallpaperSession::render_frame` | `engine/session.rs:389` 仅返回 `Ok(())`；仓库无调用。Wayland 主循环调用 `render_frame_to_surface`，X11 调用 `render_frame_to_x11_window` | 函数注释明确已不用；`WallpaperSession` 仍为 crate 导出，真正渲染入口承担首帧及清理行为 | **删除空入口**；同步过时注释，workspace all-targets check 与 Wayland 协议/生命周期测试验证，保留实际渲染函数 |
| 独立 `wayland/layer_shell.rs` | `LayerSurface`、`WallpaperState`、builder 和配置的构造/使用均限于本文件；`wayland/mod.rs` 与 crate root 仅重导出。README/docs/examples/scripts 无消费者 | 实际主引擎使用 `LayerSurfaceInfo`、原生 `ZwlrLayerSurfaceV1` 及 `EngineState`，其 `Drop` 与内存协议测试独立存在 | **删除仓库内无消费者的封装及专属导出**；记录公开 Rust 导出移除，保留主引擎路径。外部下游不在此仓库可证明范围，若出现明确消费者先保留 |
| `smithay-client-toolkit` | engine Cargo.toml 直接依赖；源码 `smithay_client_toolkit` 仅在上述独立封装出现 | 主引擎依赖原生 wayland-client/protocols 与 calloop | **随封装删除专属依赖**；检查 lockfile、cargo tree 与 all-targets check，避免连带删除主引擎依赖 |
| `OutputManager` 本体 | EngineState 持有；生产调用 add/remove、update geometry/mode/scale、mark_ready、get、output_names、ready_outputs | `GetOutputs`/`GetStatus`、输出创建、热插拔依赖该状态；协议测试也使用 ready_outputs | **保留并补测**，不整体删除 wayland 目录 |
| `OutputManager::output_at` | 忽略坐标，只返回 HashMap 首个输出；仓库无调用 | 未见命中坐标的现有生产入口；当前实现不能保证坐标语义 | **删除无消费者占位方法**，不借清理引入新命中测试功能 |
| `OutputManager::{get_mut,get_all_ready}` | 定义外未发现接收者为 OutputManager 的调用；engine/wayland_tests:340 的 get_mut 是 layer_surfaces HashMap，不能混为引用 | 实际读取走 get/ready_outputs，更新走专用 update 方法 | **删除未使用辅助方法**；用编译和输出协议测试验证 |
| `OutputManager::{len,is_empty}` | 生产无调用，现有 new 测试使用；无占位行为 | 属于结构查询而非无效分支 | **保留并在输出状态测试使用**；不为覆盖率删除有意义的微小查询接口 |
| `PendingOutput::{name,done}` | `engine/mod.rs:593` 的 name 只初始化；done 初始化并在 Done 事件写 true，无读取；struct 级 dead_code 标注掩盖这些字段 | global 已作为 HashMap key 与事件 user data；输出 ready 状态由 OutputManager 管理 | **删除冗余字段与相应写入**，缩小/移除掩盖标注；协议 Done/热插拔测试验证 |
| `PendingOutput.wl_output`、`OutputState.wl_output` | pending 对象的 Drop 调用 release；output state 中 handle 在 `apply_wallpaper_to_output` 克隆以定向 layer surface | pending 必须在 renderer/native surface 清理后释放；protocol tests 检查 destructor 次数 | **保留所有权字段与 Drop**；绝不能按“字段未直接计算”删除 |
| `LayerSurface.config` | 独立封装中的 dead_code 字段；构造时配置已用于设置 surface | 该字段属于整个无消费者封装 | **随封装删除**，无需在主引擎新增等价存储 |
| `FileWatcher.debouncer` | scanner.rs:442 附近持有 Debouncer，构造结束前调用 `watcher().watch`；之后不主动读取 | 持有 watcher 才能继续收到文件事件，drop 会停止监听 | **保留并核查多余 dead_code 标注**；增加 watcher 释放/事件测试，不能删除字段 |
| `path.is_file()` 后 `path.exists()` | scanner callback 在 Any 事件内二次读取文件状态；也有外层不存在文件处理 | 文件可在两次查询之间消失，内层 Deleted 分支不是静态不可达 | **合并判断但保留删除竞态语义**；若不能证明等价则保留该分支并补测试，不直接删 Deleted |
| `FileEvent::Created` 与 wildcard | 当前内部 callback 不发 Created，但 process_event 接受它；FileEvent 是 library 公开导出，AsyncFileWatcher 暴露 event_sender；DebouncedEventKind 为外部库类型 | 外部注入 Created 有效；wildcard 为未来枚举变体提供 fallback | **保留有效公开事件和前向兜底**，若多个匹配臂语义相同可合并，不能仅凭内部无生产者删除 |
| `WayvidService::apply_wallpaper` | bridge.rs:415；lib/main.dart 生产调用新版 source 入口，但 package smoke fixtures 与 web_lifecycle_tests 仍调用 path 入口；FRB 生成绑定导出两者 | add-all-wallpaper-types Task 6.4 与 design 明确要求保留旧路径解析适配；不读旧存储格式是另一项决定 | **保留公共入口、合并内部调度**；验证 path/source 同行为、typed properties 与失败事件 |
| 两个 FRB 入口锁内外 `validate_source` | path 入口和 source 入口均在准备后拿 service 锁，再验证组件状态；uninstall 也获取同锁停 Web | 可用性在准备与获取锁之间变化，锁内复查保护卸载竞态 | **保留时序并抽共同应用流程**；不能将重复调用视为全冗余；用 boundary/lifecycle tests 验证 |
| `EngineController::apply_wallpaper` | engine.rs:142 为带空 properties 的转接；bridge path 入口和 engine tests 使用 | 真实处理已集中到 apply_with_properties；薄封装保留调用语义 | **随边界合并审查**，仅在消费者全部迁移后删内部 wrapper，不改变有效入口 |
| Flutter fake 的 path 转接 | test/support/fakes、controller_restore_test、tray_controls_test 的 source fake 调用 path fake；生产 main.dart:342/381 直接调 applyWallpaperSource | path fake 只收路径，可能遗漏 typed properties/type/source context | **对齐真实 source 调用并补断言**，保留测试所需失败模拟和恢复语义 |
| Scene/Web 零/低覆盖路径 | filtered 热点 draw/runtime/program/gpu/text/sound/web；真实 renderer 和生产 apply 路径引用 | 对应真实五类型能力、GPU/音频生命周期与安全限制；本次普通 coverage 未启用原生环境 | **保留并针对可隔离逻辑补测**；不能因当前报告为 0 而移除模块 |

### 可复查的搜索命令

```sh
rg -n 'render_frame\(|render_frame_to_surface|render_frame_to_x11_window' crates rust lib test scripts
rg -n 'LayerSurface|WallpaperState|smithay_client_toolkit' crates rust lib test scripts docs examples README* --glob '!**/*generated*'
rg -n 'output_at|get_all_ready|ready_outputs|output_names' crates rust lib test scripts
rg -n 'PendingOutput|pending\.(name|done)|wl_output' crates/wayvid-engine/src/engine
rg -n 'FileWatcher|FileEvent::|event_sender|debouncer' crates rust lib test scripts
rg -n 'apply_wallpaper\(|apply_wallpaper_source\(|applyWallpaper\(|applyWallpaperSource\(' crates rust lib test scripts --glob '!**/*generated*'
rg -n '6\.4|旧路径|路径应用接口' openspec/changes/add-all-wallpaper-types/{tasks,design}.md
```

### 本轮收尾

- Task 1.1 与 1.2 完成；没有删除业务代码，没有修改 coverage 配置或 CI，也没有恢复此前移除的检查步骤。
- 下一步 Task 2 才实施引擎遗留代码清理；本轮按用户“每个 Task 完成后暂停”的要求在 Task 1 后停止。未暂存、提交或 push。

## Task 2.1 — 引擎遗留实现清理

- 删除只返回成功的 `WallpaperSession::render_frame`；真实 Wayland/X11 渲染入口保留。
- 删除 418 行无仓库消费者的 `wayland/layer_shell.rs`（含独立 WallpaperState、builder/config）、对应模块和 crate root 重导出。更新 crate 架构图与 wayland 模块文档，指向实际 engine 原生 surface 所有权。
- 移除仅供该封装使用的 smithay-client-toolkit；通过离线 Cargo check 更新 lockfile，清理专属传递依赖，未升级仍需使用的依赖。`cargo tree -p wayvid-engine --depth 1 --locked --offline` 确认主引擎 calloop 0.14 / calloop-wayland-source 0.4 / 原生 Wayland 依赖保留。
- `cargo check --workspace --all-targets --offline` 通过（9.80 秒），日志 `/tmp/varpaper-cleanup-task2-check.log`。
- `cargo test -p wayvid-engine engine::wayland_tests:: --lib --offline` 实际执行 3 项内存协议测试并全部通过（0.20 秒），日志 `/tmp/varpaper-cleanup-task2-protocol.log`；包含 stale configure/frame/closed、移除/重连、输出 release 与 surface destructor 次序、20 轮生命周期，无 GPU 或桌面连接。
- 重复搜索源码/文档/示例未发现被删符号的残留消费者。本次公开 Rust 导出确有移除，不承诺仓库外下游源兼容；Flutter/FRB 接口无变动。

## Task 2.2 — 输出辅助方法与冗余字段

- 删除 `OutputManager::{get_mut,get_all_ready,output_at}`。保留生产读取/更新方法及有效的 len/is_empty 查询。
- 删除 PendingOutput 只初始化的 name、只写不读的 done 字段及其写入，移除 struct 级 allow(dead_code)。global id 仍为 pending_outputs 的 key，输出完成状态仍由 OutputManager.ready 管理。
- 保留 PendingOutput.wl_output 及 Drop：该对象在 renderer 和 surface 清理后发送 release。保留 OutputState.wl_output：实际 layer-shell surface 创建需要用它指定输出。所有权字段不是冗余状态。
- 新增 3 项测试，全部实际执行通过：
  - `wayland::output::tests::unknown_output_updates_leave_manager_empty`：空 manager 对未知输出更新/mark_ready/remove 均无副作用。
  - `engine::wayland_tests::output_manager_ready_updates_replacement_and_removal`：借用内存协议 fixture 的真实 WlOutput proxy 验证注册尚未 ready、geometry/mode/scale 更新、输出独立性、重复 ready、同名替换重置与移除/重复移除。
  - `engine::wayland_tests::protocol_output_updates_become_visible_after_done`：直接派发实际 wl_output 事件处理器，确认 geometry/current mode/scale 在 Done 后发布；Preferred 非 Current mode 不覆盖当前分辨率，OutputAdded 携带更新后信息。
- `cargo test -p wayvid-engine --lib --locked --offline`：91 项返回成功（新增上述 3 项；原有环境 opt-in 测试仍提前返回，不作为 GPU/桌面验收）。日志 `/tmp/varpaper-cleanup-task2-engine.log`，测试执行 0.51 秒。现有 3 项协议生命周期测试在清理字段后再次通过。
- 最终 `cargo check --workspace --all-targets --locked --offline`、`cargo clippy --workspace --all-targets --locked --offline -- -D warnings`、`cargo fmt --all -- --check`、`git diff --check` 通过；`cargo +1.88.0 check --workspace --all-targets --locked --offline` 通过，日志 `/tmp/varpaper-cleanup-task2-msrv.log`。
- 按 lockfile 的 (name, version) 比较，删除 12 项旧封装专属 package 版本，新增 0 项，没有升级仍在使用的依赖。
- 本 Task 改动位于现有 Codecov 排除的原生模块，未声称 filtered coverage 已提高；全量 coverage 前后比较仍由 Task 5 执行。本轮未运行完整 quality/Flutter 检查，未上传 Codecov。
- Task 2.1、2.2 完成后暂停；下一项为 Task 3.1 边界逻辑合并。未暂存、提交或 push。

## Task 3.1 — 共享 Rust 应用调度

- 两个公共 FRB 应用入口分别准备路径或 typed source 后，统一调用 `apply_prepared_source`。保留 service 锁内的组件可用性复查、输出等待、首帧确认与错误映射；卸载继续使用同一把锁。路径入口仍传空 properties，source 入口传准备后的 typed properties。
- 删除消费者已迁移完毕的内部 `EngineController::apply_wallpaper` 薄包装，测试改用 `apply_with_properties`。公共 path/source 接口签名未变，无需重新生成 bindings。
- 新增 boundary 测试验证无效 source、伪造身份/版本、缺失 Web 组件和卸载期间复查；失败时不启动引擎、不产生应用事件、不创建组件目录。
- 新增 X11 实际应用测试验证两个入口都确认首帧；损坏 PNG 返回 apply_failed/失败事件，不覆盖原有赋值。扩展实际 Web 生命周期 fixture，验证字符串、布尔值、数字属性准备，以及卸载失败、重试、重装和重启后保留用户 source。

## Task 3.2 — 扫描事件合并与所有权

- 回调只过滤一次扩展名，将 is_file/exists 两次观测交给生产使用的纯分类函数。`(false, true)` 忽略，`(false, false)` 删除，`(true, false)` 对 Any 删除、对 AnyContinuous/未来种类保持原 Modified 兜底，`(true, true)` 修改。保留两次文件查询之间删除的竞态语义。
- 保留公开 Created 事件和 AsyncFileWatcher 注入 API；保留 Debouncer 所有权字段，删除多余 allow(dead_code)，添加持有目的说明。
- 新增确定性分类测试及真实临时目录 watcher 测试：修改、删除、忽略 txt、Created 处理，以及释放 watcher 后通道断开。
- 初次 watcher 测试因解构使用 `..` 导致 debouncer 保留到作用域结束而失败；改为显式析构、drop debouncer/scanner 后重跑通过。此修正仅影响测试生命周期。定向 2 项通过，日志 `/tmp/varpaper-cleanup-task3-scanner.log`。

## Task 3.3 — 路径边界与组件恢复补测

- project 新增 4 项测试，覆盖路径规范化、缺失根/文件、目录/非法名、共享根去重与非法根、稀疏超限资源、缺失/损坏 manifest、缺失/越界入口、非法预览、package 缺项/截断/重复规范化路径/坏 header/非法 UTF-8。
- Web component 新增 3 项测试，覆盖已安装时幂等、忙碌安装/卸载与取消、卸载阻塞播放、卸载失败释放状态后重试、损坏/超限安装记录、缺失记录项及越界记录路径。使用微型合成归档，不下载官方 runtime。
- project 与 Web component 生产实现均未修改。保留路径安全、资源预算、坏记录拒绝、取消、busy 互斥、卸载清理与失败恢复分支；这些是有效行为。GPU/Web/音频模块及所有权资源继续保留。

### 执行结果

- `cargo test -p wayvid-library --lib --locked --offline`：46 项通过，日志 `/tmp/varpaper-cleanup-task3-library.log`。
- `cargo test --workspace --locked --offline`：engine 91、library 46、service 44，共 181 项返回成功，doc-tests 通过；普通运行中的原生 opt-in 测试仍提前返回。日志 `/tmp/varpaper-cleanup-task3-workspace.log`。
- `cargo clippy --workspace --all-targets --locked --offline -- -D warnings`：通过。首次检查发现合并后内部 apply_wallpaper 包装已无生产消费者，删除包装并迁移测试后重跑通过。日志 `/tmp/varpaper-cleanup-task3-clippy.log`。
- `cargo +1.88.0 check --workspace --all-targets --locked --offline`：通过，日志 `/tmp/varpaper-cleanup-task3-msrv.log`。
- 使用本机已有 pinned CEF SDK 构建 helper：`VARPAPER_WEB_BUILD_DIR=/tmp/varpaper-cleanup-web-host scripts/build-web-host.sh`，通过；日志 `/tmp/varpaper-cleanup-task3-web-build.log`。
- 在私有 Xvfb、软件渲染和临时 XDG 目录中运行全部 service 测试：44 项返回成功（29.86 秒），实际执行 X11 首帧/失败候选和 CEF 生命周期；Wayland 专项及官方归档 opt-in 未启用。Fontconfig 输出字体配置警告，未导致失败。日志 `/tmp/varpaper-cleanup-task3-native-final.log`。执行命令如下；临时目录由本轮准备，CEF runtime 为已有 SDK 的独立目录链接，未使用用户桌面或用户音频服务：

```sh
env -u WAYLAND_DISPLAY \
  XDG_SESSION_TYPE=x11 LIBGL_ALWAYS_SOFTWARE=1 SDL_AUDIODRIVER=dummy \
  VARPAPER_TEST_CEF_SOFTWARE=1 VARPAPER_TEST_X11=1 \
  VARPAPER_WEB_HOST=/tmp/varpaper-cleanup-web-host/varpaper-web-host \
  VARPAPER_CEF_RUNTIME=/tmp/varpaper-cleanup-native-hvv8bkhg/runtime \
  VARPAPER_TEST_CEF_SDK="$PWD/build/cef/cef_binary_135.0.17+gcbc1c5b+chromium-135.0.7049.52_linux64_minimal" \
  VK_ICD_FILENAMES=/tmp/varpaper-cleanup-native-hvv8bkhg/runtime/vk_swiftshader_icd.json \
  XDG_DATA_HOME=/tmp/varpaper-cleanup-native-hvv8bkhg/data \
  XDG_CONFIG_HOME=/tmp/varpaper-cleanup-native-hvv8bkhg/config \
  XDG_STATE_HOME=/tmp/varpaper-cleanup-native-hvv8bkhg/state \
  PULSE_SERVER=unix:/tmp/varpaper-cleanup-native-hvv8bkhg/no-pulse \
  xvfb-run -a cargo test -p wayvid_gui --lib --locked --offline -- --test-threads=1 --nocapture
```

### 同口径 Rust 覆盖率中间结果

沿用基线工具链、命令与现有 ignore。基于 HEAD `2dc69c1` 加本轮未提交改动运行：

```sh
RUSTFLAGS='-D warnings' cargo llvm-cov --workspace --ignore-filename-regex frb_generated --lcov --output-path /tmp/varpaper-cleanup-task3-lcov.info
```

日志 `/tmp/varpaper-cleanup-task3-coverage.log`；报告 SHA-256、工作区状态、所有文件计数和生产新增覆盖行保存于 `coverage-task3.json`。数据为 Task 2 + Task 3 相对 Task 1 基线的累计差异。

| Rust 范围 | 基线覆盖 / 总行 | 当前覆盖 / 总行 | 基线 → 当前未覆盖 |
| --- | ---: | ---: | ---: |
| 原始 | 7265 / 14941（48.6246%） | 7664 / 14977（51.1718%） | 7676 → 7313 |
| 现有 ignore 过滤后 | 6574 / 10854（60.5675%） | 6947 / 11129（62.4225%） | 4280 → 4182 |

| 文件 | 基线覆盖 / 总行 | 当前覆盖 / 总行 | 未覆盖减少 |
| --- | ---: | ---: | ---: |
| library/project.rs | 347 / 354 | 471 / 473 | 5 |
| library/scanner.rs | 490 / 578 | 597 / 643 | 42 |
| rust/bridge.rs | 643 / 907 | 673 / 905 | 32 |
| rust/engine.rs | 115 / 303 | 115 / 303 | 0 |
| rust/web_component.rs | 753 / 900 | 864 / 993 | 18 |

- 上表包含 llvm-cov 发出的测试代码行，覆盖行增加不能全部归为生产代码。直接比较未修改的生产部分，project 新覆盖 5 行（40、70、116、217、244），Web component 新覆盖 18 行（132、133、135–139、141–145、212、213、392、533、541、552）；两者无原已覆盖生产行退化。Web 对比排除 cfg(test) SDK helper 与 tests。
- 普通覆盖率运行未启用原生 opt-in；上面的独立 Xvfb/CEF 测试不计入本次覆盖率。因此保留 engine/图形路径未覆盖行，不将其认定为无用。
- 本轮未修改 Flutter、coverage 配置或 CI，也未恢复已移除步骤；Flutter 未重跑，完整 quality 与双方覆盖率收尾仍由 Task 5 执行。远程 Codecov 未验收。
- Task 3.1–3.3 完成后按用户要求暂停，下一组为 Task 4。未暂存、提交或 push。

## Task 4.1 — Flutter typed source 调用与无用分支清理

- 共享 FakeService 直接记录 `(source, output)`，不再将 typed source 转发到路径入口；路径入口明确报错，防止 controller 测试意外退回路径调用。保留字符串调用日志供现有暂停/路由断言使用。
- restore 与 tray 的局部 fake 直接实现 source 入口；Web 恢复失败按 wallpaperType 判断。托盘断言完整 source 与实际保存记录一致，并核对目标输出和轮换顺序。
- 新增受 Completer 控制的应用确认测试，逐一验证 video/scene/web/image/gif 的版本、身份、类型、项目上下文和字符串/布尔/数字 overrides 原样传递；确认返回前不更新显示或持久化赋值，失败候选保留旧赋值，迟到的 WallpaperApplied 事件不覆盖确认后的状态。
- 删除 apply 两处分支中 sourcePath/sourceId 的重复赋值：随后执行的 setSourceDescription 已设置全部字段。保留成功确认后才保存的顺序、所有输出/单输出两种持久化语义。
- 删除恒返回 false 的 anotherInstance getter 及唯一依赖它的次实例提示分支。仓库搜索仅发现这两处引用，提示无可达显示路径；当前架构不依赖此占位状态。未删除有效错误提示、组件诊断或取消逻辑。
- 初次四文件定向测试 43 项通过；加入恢复边界测试后，controller_restore/controller_logic/tray_controls/settings_store/shell_widgets 五文件共 74 项通过，日志 `/tmp/varpaper-cleanup-task4-targeted-final.log`。

## Task 4.2 — 新格式校验、恢复与组件状态补测

- 新增存储测试：逐项损坏 version/id/type/overrides 必须拒绝读取，未知版本必须拒绝设置；失败更新不覆盖已有 Isar 记录；显式清除将 source 全部字段清空。没有增加旧格式转换或兼容。
- 使用运行时构造的 FpsLimitPatch 验证 u32 最大值、空 patch 保留原值、unlimited 清除与实际数据库往返。原有有限数值/范围错误测试继续保留。
- 新增恢复测试：缺失文件跳过应用但保留持久化赋值；文件存在但描述不完整时报告错误且保留记录；修复为有效描述后可恢复同一 source。
- 新增组件状态测试：等待中的安装拒绝重复操作，动作结束后读取真实 downloading 状态，事件轮询刷新状态，取消失败释放 pending 并反馈错误，再次取消成功恢复 not_installed。保留安装/取消/卸载各自入口与错误分支。
- 新增反向轮换测试：当前 id 不在列表时从最后一项开始；保留空列表、不可用项目和输出差异处理。
- SettingsStore.open、真实 autostart 文件操作及原生 main 启动仍保留；这类平台入口未在本组测试中执行。未改用户 HOME 中的文件；没有为覆盖率删除这些功能。

### Flutter 验证与中间覆盖率

- `puro flutter test --coverage`：92 项全部通过（基线 86 项，本组新增 6 项），日志 `/tmp/varpaper-cleanup-task4-coverage.log`，报告 `/tmp/varpaper-cleanup-task4-lcov-flutter.info`。
- 按当前 CI quality 的范围运行分析，通过：

```sh
puro flutter analyze lib/main.dart lib/src/l10n.dart lib/src/storage/settings_store.dart test scripts/fixtures/scene_package_smoke_test.dart scripts/fixtures/component_package_smoke_test.dart
```

日志 `/tmp/varpaper-cleanup-task4-analyze-scoped.log`。此前无参数 `puro flutter analyze` 额外报告已有 preview_cache collection lint、生成 Isar 文件 experimental API 警告，以及新测试 if 缺大括号；新测试已修正，未手改生成代码或调整分析排除。无参数分析的其他现有问题不作为已通过声明。

- 6 个本组改动 Dart 文件的 `puro dart format --output=none --set-exit-if-changed ...` 通过，`git diff --check` 通过。
- 工具链与基线一致（Flutter 3.47.4 / Dart 3.13.3），沿用 SF/DA 合并与原有 ignore 口径。`coverage-task4.json` 保存报告 SHA-256、HEAD/工作区状态、逐文件计数和新增覆盖行映射。

| Flutter 范围 | 基线覆盖 / 总行 | 当前覆盖 / 总行 | 基线 → 当前未覆盖 |
| --- | ---: | ---: | ---: |
| 原始 | 1512 / 2656（56.9277%） | 1513 / 2649（57.1159%） | 1144 → 1136 |
| 现有 ignore 过滤后 | 1475 / 1525（96.7213%） | 1475 / 1518（97.1673%） | 50 → 43 |
| main.dart | 1206 / 1238 | 1202 / 1231 | 32 → 29 |
| settings_store.dart | 219 / 237 | 223 / 237 | 18 → 14 |

- 按基线源码与当前源码相同文本块映射，新增覆盖 main 的状态轮询 2 行（原 319–320，现 318–319）、store 的坏描述/未知版本拒绝和运行时 FPS patch 构造 4 行（83、97、439、440）。没有原已覆盖的保留行退化。main 删除 7 个可执行统计行，其中 6 行原已覆盖、1 行原未覆盖，因此过滤后覆盖行总数持平；未将全部比例提升归因于新增测试。
- 生成报告已复制至临时目录；仓库受跟踪 coverage/lcov.info 恢复为本组开始时的 HEAD 版本。本组没有修改生成文件、coverage ignore、Codecov 门槛或 CI，也没有恢复已移除步骤。
- Task 4.1–4.2 完成后按用户要求暂停。总进度 9/11，剩余 Task 5 完整 quality 与 Rust/Flutter 最终比较。本组没有执行 Flutter clean/pub get、重跑 Rust 检查或上传远程 Codecov；不将本组结果称为完整 quality 验收。未暂存、提交或 push，保留此前 Task 3 的工作区改动。

## Task 5.1 — 当前 quality 的完整本地验收

本轮对 HEAD `2dc69c1` 加 Task 3/4 未提交改动执行当前 `.github/workflows/ci.yml` quality 中的全部本地代码检查。Rust 命令均设置 `RUSTFLAGS='-D warnings'`、`RUST_BACKTRACE=1`，无新增原生测试 opt-in。现有 Rust 1.97.1 / cargo-llvm-cov 0.9.1 / Flutter 3.47.4（9584c6713b）/ Dart 3.13.3 与基线一致；MSRV 使用已安装的 Rust 1.88.0。

| 命令 | 结果 | 日志 `/tmp/` 下文件 |
| --- | --- | --- |
| `cargo +1.88.0 check --workspace --all-targets --locked` | 通过 | varpaper-cleanup-task5-msrv.log |
| `cargo fmt --all -- --check` | 通过 | varpaper-cleanup-task5-fmt.log |
| `cargo clippy --workspace --all-targets -- -D warnings` | 通过 | varpaper-cleanup-task5-clippy.log |
| `cargo llvm-cov --workspace --ignore-filename-regex frb_generated --lcov --output-path /tmp/varpaper-cleanup-task5-lcov-rust.info` | engine 91、library 46、service 44，共 181 项返回成功；生成报告成功 | varpaper-cleanup-task5-rust-coverage.log |
| `puro flutter clean` | 重跑后完整通过 | varpaper-cleanup-task5-flutter-clean-final.log |
| `puro flutter pub get` | 通过；锁文件未变 | varpaper-cleanup-task5-flutter-pub-final.log |
| 当前 CI 范围的 `puro flutter analyze`（完整参数见 Task 4） | No issues found | varpaper-cleanup-task5-flutter-analyze-final.log |
| `puro flutter test --coverage` | 92 项全部通过 | varpaper-cleanup-task5-flutter-coverage-final.log |

### clean 的环境修复

首次 clean 返回 0，但日志显示 build 下旧 root 所有权构建目录删除失败，因此未将该轮当作完整 clean 验收。将原 build 整体通过同一文件系统 rename 保留至忽略目录 `target/varpaper-quality-preserved-p8_9hr95/build`，没有修改所有权或删除其中产物；随后完整重跑 clean/pub get/analyze/test coverage，clean 日志无删除失败。旧 CEF SDK/打包产物现保存在该路径，后续若复用 Task 3 SDK fixture，应使用保留目录中的 SDK 路径或恢复该构建目录。路径另存于 `/tmp/varpaper-cleanup-task5-preserved-build.txt`。

### 范围核对

- 当前 quality 与基线 workflow SHA-256 相同；已删除的 package/manifest 与 native fixtures 两步没有恢复。packages job 中已有发布打包校验不属于本次删除/恢复范围。
- `codecov.yml` SHA-256 与基线相同，project auto/1%、patch 80%/1%、现有 ignore 和两个 flag 的 carryforward 均未调整。没有新增生产代码排除。
- 本地已具备所需工具和系统依赖，未在 Arch 主机执行 Ubuntu apt 安装步骤，也未调用 GitHub Actions 的上传步骤。本轮未提交或 push，当前工作区版本没有对应远程 CI/Codecov 报告，**远程上传与两个 flag 尚未验收**。
- 普通 Rust coverage 里的显示、GPU、音频、CEF/归档等 opt-in 测试仍可能提前返回；181 个成功返回不代表这些原生路径全部执行。此前 Task 2 内存 Wayland 协议测试与 Task 3 私有 Xvfb/CEF 实际执行结果仍为定向证据。

## Task 5.2 — 最终覆盖率与清理结果

最终报告元信息、源码 diff SHA-256、配置/锁文件 SHA-256、逐文件计数、命令和报告 SHA-256 已写入 `coverage-final.json`。原始报告位于 `/tmp/varpaper-cleanup-task5-lcov-rust.info` 与 `/tmp/varpaper-cleanup-task5-lcov-flutter.info`；受跟踪的 coverage/lcov.info 恢复为开始时 HEAD 版本。基线仍为 `092f1e1`，最终数据包含本 change 全部 Task 的累计改动。

| flag / 范围 | 基线覆盖 / 总行 | 最终覆盖 / 总行 | 覆盖率变化 | 未覆盖变化 |
| --- | ---: | ---: | ---: | ---: |
| Rust 原始 | 7265 / 14941 | 7664 / 14977 | 48.6246% → 51.1718% | 7676 → 7313（-363） |
| Rust 现有过滤后 | 6574 / 10854 | 6947 / 11129 | 60.5675% → 62.4225% | 4280 → 4182（-98） |
| Flutter 原始 | 1512 / 2656 | 1512 / 2649 | 56.9277% → 57.0781% | 1144 → 1137（-7） |
| Flutter 现有过滤后 | 1475 / 1525 | 1475 / 1518 | 96.7213% → 97.1673% | 50 → 43（-7） |

- 两个 flag 按相同 SF/DA 合并与原有 ignore 的本地行口径分别不下降，过滤后未覆盖手写行均减少。Flutter 最终原始数据比 Task 4 少 1 个生成桥接文件覆盖行，过滤后的手写数据完全一致；最终 Rust 数据与 Task 3 一致。
- 手写 Rust/Dart 源码 diff 相对基线共新增 1006 行、删除 544 行，**包含新增/扩展测试**，不是净删除 544 行。Cargo.lock 另新增 13、删除 142 行；package 版本集合移除 12 项，新增 0 项，无保留依赖升级。pubspec.lock 未变。
- 主要删除项：418 行无消费者 layer-shell 封装及其导出/专属依赖、空 render_frame、三个无消费者 OutputManager 方法、PendingOutput 的 name/done 冗余状态、失去消费者的内部应用包装、Flutter 重复字段赋值和恒假次实例提示。
- 主要合并项：Rust path/source 应用调度共享锁内校验和首帧确认；scanner 共享文件事件分类；Flutter fake 直接记录 typed source，消除 source→path 转接。
- Rust 测试从基线 167 增至 181，Flutter 从 86 增至 92。新增覆盖不能全归于生产代码，因为 llvm-cov 包含 Rust 测试体。已单独核对未修改生产代码：project 新覆盖 5 行、Web component 新覆盖 18 行；Flutter main 新覆盖 2 行、store 新覆盖 4 行，具体行号见 Task 3/4 与相应 JSON。
- 保留的重要低覆盖路径仍包括 Scene GPU/Web/音频、真实平台启动/autostart、资源预算/路径安全、超时/重试/取消，以及 Debouncer/协议对象所有权。它们有实际职责；当前未启用环境不能成为删除依据。未用原始比例变化或本地过滤结果冒充 Codecov 远程计算。
- Task 5.1、5.2 完成，本 change 实施进度 **11/11**。本轮只更新验收材料与任务状态，未新增业务代码改动，未暂存、提交、push 或归档。
