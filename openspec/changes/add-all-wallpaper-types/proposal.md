# Proposal

## Why

VarPaper 当前仅具备 mpv 媒体播放路径：Scene 虽能入库却无法正确渲染，Web 被扫描器跳过，Image/GIF 的界面分类也被合并为 Video。参考本机 linux-wallpaperengine 的 Video、Scene、Web 实现补齐类型支持，使用户能在同一壁纸库中应用不同类型的本地和 Workshop 壁纸。

## What Changes

- 完整保留 Video、Scene、Web、Image、GIF 类型，增加项目目录、入口文件、资源定位及可播放状态，禁止未知类型回退为 Video。
- 支持已安装 Workshop 和用户添加目录中的 Wallpaper Engine 项目；保留现有媒体文件扫描和稳定身份。
- 在现有 Wayland/X11 桌面输出上增加 Scene 渲染，覆盖参考实现的资源包、纹理、材质/效果、图像、粒子、文字、声音及 SceneScript 功能；明确支持系统音频频谱可视化、动画时间与本地日期/时钟，并通过兼容样例明确边界。
- 增加 CEF 离屏 Web 渲染，支持本地 HTML/CSS/JavaScript、Canvas/WebGL、相对资源、项目属性及音频接口。
- 各类型统一应用、暂停/恢复、静音/音量、切换、清除、显示器缩放及重启恢复；界面提供准确类型、预览和错误原因。
- 将新增运行库及辅助进程纳入 .deb、Flatpak、Arch 打包、测试和来源声明。
- 按用户确认，Application 可执行程序壁纸不在本次范围内；明确显示不支持，不尝试运行。类型覆盖不等于对全部 Workshop 内容或 Windows 专有功能无条件兼容。

## Capabilities

### New Capabilities

- `wallpaper-project-loading`: 壁纸类型、项目发现、资源解析、兼容状态及安全加载。
- `scene-wallpaper-rendering`: Wallpaper Engine Scene 资源与渲染、动画、脚本、属性、音频响应。
- `web-wallpaper-rendering`: 离屏浏览器壁纸、本地资源、网页属性和音频接口。
- `wallpaper-runtime`: 多类型运行调度、桌面输出、生命周期、界面呈现及持久化恢复。

### Modified Capabilities

无。现有 `video-playback` 的视频行为保持；本次新增能力扩展类型覆盖。`gui-integration`/`config-management` 仍描述旧 daemon/YAML 架构，需由进行中的架构变更协调，本次不以恢复旧架构满足它们。

## Impact

- `crates/wayvid-library/src/{model,scanner,workshop,thumbnail}.rs`：项目模型、发现、预览。
- `crates/wayvid-engine/src/engine/{command,session,mod,x11}.rs`、`egl.rs`：可替换渲染会话、Scene/Web 渲染与输出集成。
- `rust/src/{bridge,engine}.rs`、FRB 生成绑定：类型化源描述、状态和错误。
- `lib/main.dart`、本地化、`lib/src/storage/settings_store.dart`：类型过滤、详情、属性编辑、赋值兼容及恢复。
- 新增原生 Scene/CEF 适配组件和构建依赖；参考目录仅为研究来源，发布构建不依赖该绝对路径。
- `linux/`、`packaging/`、`scripts/`、CI、测试、现有文档和 `NOTICE`：原生集成、分发、回归验证与出处。
