# 当前项目架构

Wayvid 是一个 Flutter-first Linux 桌面应用。Flutter 位于仓库顶层，负责 UI、状态管理、持久化、托盘和 engine 单实例；Rust 作为同进程 FRB service，负责扫描、预览、Wayland surface 和 MPV 播放。

## 总体结构

~~~text
Flutter application
├── Riverpod
│   └── WayvidController
├── Isar Community
│   ├── AppSettingsRecord
│   └── WallpaperAssignmentRecord
├── window_manager
├── tray_manager
└── flutter_rust_bridge
    └── Rust service (rust/)
        ├── scan_folder / scan_workshop
        ├── load_preview
        ├── refresh_monitors
        └── create/update/stop/apply/pause/resume engine

Rust service
├── wayvid-library
│   ├── FolderScanner
│   ├── WorkshopScanner
│   └── ThumbnailGenerator
└── wayvid-engine
    ├── Wayland output/layer-shell
    ├── EGL/OpenGL
    └── libmpv
~~~

关键约束：

- Flutter 是唯一的设置持有者，设置保存在 Isar，不再读取或写入 YAML。
- Flutter 发起扫描请求，Rust 返回内存中的扫描 DTO；Rust 不保存 SQLite 库。
- engine 创建时必须收到完整 EngineConfigDto；配置修改通过 update_engine_config 下发。
- wayvid-library 和 wayvid-engine 是并列 crate，二者不能直接依赖或交互。
- Rust 不包含 CLI、Unix Socket IPC、Rust 托盘或 Rust 单实例实现。

## 目录

~~~text
.
├── lib/
│   ├── main.dart                         Flutter 页面与 Riverpod controller
│   ├── src/storage/settings_store.dart   Isar schema、设置模型和 repository
│   └── src/bridge_generated.dart/         FRB 生成的 Dart API
├── rust/
│   ├── src/bridge.rs                      Flutter service facade
│   ├── src/engine.rs                      engine 单进程内生命周期 wrapper
│   └── src/frb_generated.rs               FRB 生成的 Rust glue
├── crates/
│   ├── wayvid-engine/
│   │   └── src/                          Wayland/EGL/MPV 播放引擎
│   └── wayvid-library/
│       └── src/                          扫描、Workshop 和预览
├── linux/                                 Flutter Linux runner、CMake/Cargokit
├── pubspec.yaml                           Flutter 依赖
├── Cargo.toml                             Rust workspace
└── flutter_rust_bridge.yaml              FRB 生成配置
~~~

## Flutter 模块

| 文件 | 作用 |
| --- | --- |
| lib/main.dart | 应用启动、窗口初始化、Riverpod provider、页面、扫描/预览/engine 操作 |
| lib/src/storage/settings_store.dart | Isar collection、默认设置、读写、壁纸 assignment、Linux autostart |
| lib/src/storage/settings_store.g.dart | Isar Community 生成的 collection adapter |
| lib/src/bridge_generated.dart/bridge.dart | FRB 生成的 service、DTO 和事件类型 |
| lib/src/bridge_generated.dart/frb_generated*.dart | FRB runtime/native binding |
| linux/CMakeLists.txt | Flutter runner、Cargokit Rust service、Isar/tray 插件和 bundle |

### 状态流

~~~text
Isar.open
    ↓
SettingsStore.load
    ↓
WayvidController (Riverpod ChangeNotifierProvider)
    ├── 保存 UI/播放/电源/文件夹设置到 Isar
    ├── 将 EngineConfigDto 下发给 Rust
    ├── 请求 Rust 扫描并合并内存列表
    └── 缓存 PreviewDto，避免卡片重建时重复请求
~~~

tray_manager 直接在 Flutter 端创建菜单和处理显示/退出；关闭窗口且启用“最小化到托盘”时只隐藏窗口。

## Rust service 与 FRB API

rust/src/bridge.rs 是唯一的 Flutter/Rust 应用服务门面：

| API | 功能 |
| --- | --- |
| initialize | 返回 Workshop 可用性和 engine 状态 |
| scan_folder | 扫描本地目录并返回 WallpaperDto |
| scan_workshop | 扫描 Steam Workshop 并返回 WallpaperDto |
| load_preview | 静态图片返回安全源路径；视频/GIF 返回预览字节 |
| refresh_monitors | 查询当前显示器 |
| create_engine | 使用完整配置创建同进程 engine |
| update_engine_config | 向已运行 engine 更新配置 |
| apply_wallpaper | 传递源路径给 engine |
| pause / resume / clear_wallpaper | 控制播放 |
| poll_events | 返回 engine 状态和错误事件 |

FRB 生成方式：

~~~bash
flutter_rust_bridge_codegen generate --config-file flutter_rust_bridge.yaml
~~~

## wayvid-library

wayvid-library 只负责发现和预览，不拥有应用状态：

- scanner.rs：本地目录递归扫描、文件类型判断、Workshop 结果的扫描模型。
- model.rs：WallpaperItem、WallpaperMetadata、SourceType、WallpaperType。
- workshop.rs：Steam library、Wallpaper Engine project.json 和 Workshop 项目扫描。
- thumbnail.rs：图片/GIF/视频的内存预览生成。

此 crate 没有 SQLite、rusqlite、持久化 schema，也不依赖 wayvid-engine。

## wayvid-engine

wayvid-engine 只接收 engine 配置和源路径：

- engine/command.rs：EngineCommand、EngineEvent、EngineConfig。
- engine/mod.rs：Wayland event loop、surface 创建、输出枚举和渲染循环。
- engine/session.rs：每个输出的 EGL/MPV session，支持热切换源路径和配置更新。
- wayland/output.rs：输出状态及尺寸/位置。
- egl.rs：EGL display/context/window/surface 生命周期。
- mpv.rs：libmpv 初始化、locale、解码、HDR、布局、播放控制。
- types.rs：engine 私有输出、布局、HDR、tone mapping 和硬件解码类型。

创建流程：

~~~text
Flutter Isar settings
        ↓ EngineConfigDto
Rust bridge::create_engine
        ↓ EngineConfig
EngineController
        ↓ spawn_engine
Wayland + EGL + MPV thread
~~~

## 已移除的边界

以下代码和依赖已经从当前 workspace 移除：

- crates/wayvid-core
- crates/wayvid-ctl
- Rust ipc_server.rs
- Rust single_instance.rs
- Rust tray.rs
- wayvid-library 的 SQLite database module
- Rust YAML settings/autostart manager

因此当前应用不再提供 CLI、Unix Socket IPC、旧 YAML/SQLite 配置迁移。Cargo.toml workspace 只保留 wayvid-engine、wayvid-library 和 Flutter 的 rust service。
