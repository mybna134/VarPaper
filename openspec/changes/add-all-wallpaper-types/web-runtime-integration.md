# Web runtime 线程集成决策（待确认）

## 已验证的约束

固定 CEF 135.0.17 的 `include/cef_app.h` 要求 `CefInitialize` / `CefShutdown` 在 main application thread 调用，并规定初始化失败后只能退出。VarPaper 的 `spawn_engine` 将 renderer 生命周期运行在 Rust 工作线程，GTK/Flutter runner 持有应用主线程。不能直接将 CEF 生命周期移植到 renderer 的创建/销毁调用。

下载的 linux64 minimal 分发包通过官方 SHA-1 校验，本地 SHA-256 为 `24ac1980e62be7b1aea2c337d6bd4bc770dccd8b37e47c6db94b397f13ec4dc6`。原始下载和解包只存在于 /tmp 和 build，尚未接入发布清单。

## 推荐：一个共享 Web 宿主进程

由 Rust engine 管理一个 `varpaper-web-host`，在该进程 main 中初始化/关闭 CEF。每个输出拥有独立 browser/request context。CEF renderer/GPU 子进程仍使用单独的 subprocess executable。GTK runner 不执行 CEF 调用。

宿主通过受限本地 IPC 接收带 request/browser ID 的创建、resize、属性、频谱、音量、暂停/恢复和销毁命令。尺寸与消息大小在两端验证；paint 帧带 browser ID、generation、width/height、序号，使用有上限的共享缓冲区，Rust 仅在其 GL current 线程上传最新匹配帧。首帧提交仍由现有 candidate session 控制。项目资源通过校验后的 descriptor/root 和共享资源范围加载，不让页面获得 IPC/宿主文件权限。

暂停销毁 browser 并保留已上传纹理，恢复按 descriptor 重建。最后一个 Web renderer 释放后，宿主关闭全部 browser、执行 CefShutdown 并退出；断开/崩溃会使该宿主的 Web 赋值收到关联错误，媒体和 Scene 输出继续工作。包含显式重启、EOF、输出删除、100 次切换和遗留子进程验收。

这需要补充 browser host 的 IPC、崩溃恢复和安装清单任务，是原设计未明确的运行边界。确认后才调整 design/specs/tasks，避免将 CEF 主线程假装为任意工作线程。

## 可选：GTK/Flutter 主线程内共享 CEF

在 Linux runner main 初始化 CEF，GTK 主循环驱动其任务队列，Rust renderer 通过桥接排队 browser 操作和取 paint。CEF 初始化失败按其契约会影响整个 GUI 进程；CEF runtime 的初始化/退出要与 GTK、FRB engine shutdown 协调，Rust 引擎独立测试另需主线程宿主。此方案不增加一个浏览器宿主进程，但更改 runner 的启动/退出边界。

## 当前实现状态

项目发现、原生 Scene ABI、媒体 renderer 适配、GL 能力检测和候选首帧提交已通过测试。SceneScript、共享音频采集、完整 Scene 图形兼容、Web、界面持久化、安装包与综合验收仍未完成。此文档不代表 Web 已实现或原设计已改变。
