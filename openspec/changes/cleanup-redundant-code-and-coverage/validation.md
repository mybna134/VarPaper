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
