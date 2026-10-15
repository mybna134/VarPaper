<div align="center">

<img src="packaging/varpaper.svg" alt="VarPaper 标志" width="100" height="100">

# VarPaper

适用于 Linux Wayland 和 X11 桌面的动态壁纸管理器

[![许可证](https://img.shields.io/badge/license-GPL--3.0--only-blue?style=for-the-badge)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.88%2B-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![Flutter](https://img.shields.io/badge/Flutter-3.47.4-02569B?style=for-the-badge&logo=flutter)](https://flutter.dev/)

[![最新版本](https://img.shields.io/github/v/release/mybna134/VarPaper?style=for-the-badge&label=Latest%20Release&logo=github)](https://github.com/mybna134/VarPaper/releases/latest)
[![构建状态](https://img.shields.io/github/actions/workflow/status/mybna134/VarPaper/ci.yml?branch=main&style=for-the-badge&label=Build&logo=githubactions)](https://github.com/mybna134/VarPaper/actions/workflows/ci.yml)
[![Codecov](https://codecov.io/gh/mybna134/VarPaper/branch/main/graph/badge.svg)](https://codecov.io/gh/mybna134/VarPaper)

[![English](https://img.shields.io/badge/Language-English-0A66C2?style=for-the-badge)](README.md)
[![简体中文](https://img.shields.io/badge/Language-%E7%AE%80%E4%BD%93%E4%B8%AD%E6%96%87-0A66C2?style=for-the-badge)](README.zh-CN.md)

</div>

VarPaper 是一款 Flutter 桌面应用，使用 Rust 渲染引擎和 mpv 媒体播放。它会扫描你添加的文件夹，以及本机已安装的 Wallpaper Engine 创意工坊内容，供你在媒体库中选择壁纸并应用到一台或所有显示器。

## 功能

- 浏览视频、Scene、网页、图片和 GIF 壁纸，支持预览、搜索和类型／来源筛选。
- 为不同显示器设置不同壁纸，也可以将同一壁纸应用到所有显示器。
- 查找本机已安装的 Wallpaper Engine 创意工坊项目。创意工坊内容由 Steam 管理和下载，VarPaper 不负责下载。
- 启动时恢复各显示器之前设置的壁纸。
- 通过系统托盘暂停、继续播放或更换壁纸。
- 在应用中配置播放、外观、开机启动和电池供电时暂停等选项。

mpv 会在可用时自动使用硬件解码。Wayland 环境需要合成器支持 `wlr-layer-shell`；即使系统中存在 XWayland，Wayland 会话仍使用 layer-shell。原生 X11 会话通过 RandR 检测显示器，并使用桌面类型的壁纸窗口；窗口的堆叠行为取决于窗口管理器和桌面图标管理器。

## 安装

Linux x86_64 版本以 `.deb`、`.flatpak` 和 Arch `.pkg.tar.zst` 格式发布，可在[发布页面](https://github.com/mybna134/VarPaper/releases)下载。

`.deb` 在 Ubuntu 24.04 上构建，需要 glibc 2.39 及以上版本、`libmpv.so.2` 和其他声明的系统库。在兼容的 Debian 系发行版上，下载后运行：

```bash
sudo apt install ./varpaper_*_amd64.deb
varpaper
```

在 Arch Linux 上，下载 `.pkg.tar.zst` 包并使用 pacman 安装：

```bash
sudo pacman -U ./varpaper-*-x86_64.pkg.tar.zst
```

安装 Flatpak 包：

```bash
flatpak install ./varpaper_*_x86_64.flatpak
flatpak run io.github.mybna134.varpaper
```

Flatpak 使用 GNOME 50 运行时，并可访问你的主目录，以读取本地视频和 Steam 创意工坊文件。

Niri 出于安全原因会隐藏沙盒 Flatpak 应用的 `wlr-layer-shell` 协议，因此 Flatpak 版无法在 Niri 会话中设置壁纸。请在 Niri 中使用原生 VarPaper 构建；修改 Flatpak socket 权限无法恢复被合成器过滤的协议。

## 使用

在 **文件夹** 页面添加媒体或项目文件夹。在 **媒体库** 中浏览或搜索，然后双击壁纸即可将其应用到所有显示器。打开壁纸详情可以将其应用到指定显示器。在 **显示器** 页面查看当前壁纸分配，在 **设置** 页面调整启动、播放和电源选项。

应用会将设置和显示器壁纸分配保存在平台的应用支持目录中的 Isar 数据库里。旧版 YAML 设置不会自动迁移。如需登录后自动恢复壁纸，请在设置中启用 **登录时启动** 和 **恢复上次壁纸**。**最小化到托盘** 和 **启动时最小化** 分别控制窗口行为。

Wallpaper Engine 创意工坊内容必须先通过 Steam 安装到本机。VarPaper 支持视频项目和已实现的 Rust Scene 子集：图像图层、效果、动画、基础精灵粒子、文字、声音及基础脚本。高级粒子、系统音频可视化和完整时钟／脚本兼容留待后续实现；遇到未支持的必需功能会明确报错。网页项目使用可选的 Web 壁纸组件。详见 [Scene 支持范围](docs/src/dev/rust-scene.md)。

图片壁纸持续显示，GIF 通过媒体渲染器播放动画。Scene 需要 OpenGL 3.3，目前采用拉伸布局。硬件解码和 HDR 是媒体播放设置，各类型的限制见[使用指南](docs/src/user-guide/gui.md)。

网页壁纸需在 **设置 → Web 壁纸支持** 中主动安装组件。启动、扫描、应用和恢复均不会自动下载。网页使用本地资源，外部请求和下载被阻止；暂停保留最后画面并关闭页面，继续时重新创建，页面临时状态可能丢失。卸载停止网页播放并删除组件文件，保留项目和已保存的壁纸分配。

Web 支持项目初始属性和系统播放音频回调。通过服务 API 提交的 Web 属性覆盖可随分配保存，图形属性编辑器尚未实现。音频采集不可用时提供零输入并自动重试；Scene 实时系统音频可视化和本地日期／时钟绑定仍待后续实现。Application 项目不会执行，详见[项目兼容说明](docs/src/features/workshop.md)。

## 从源码构建

需要安装 Flutter、Rust，以及 GTK、mpv、Wayland、EGL、X11、RandR、XFixes 和应用指示器所需的 Linux 开发库。构建安装包还需要 `dpkg-deb`、`flatpak-builder`，以及 GNOME 50 Flatpak 运行时和 SDK。

```bash
git clone https://github.com/mybna134/VarPaper.git
cd VarPaper
./scripts/build-packages.sh
```

脚本会将三种安装包写入 `dist/`。如需构建本地 Linux 程序包，请运行 `./scripts/build-linux.sh`。开发时可运行 `flutter run -d linux` 和 `cargo test --workspace`。

## 项目结构

```text
lib/                    Flutter 应用和设置
linux/                  Flutter Linux runner 和 Rust 构建集成
rust/                   Flutter/Rust 桥接服务
crates/wayvid-engine/    Wayland/X11 壁纸播放
crates/wayvid-library/  文件夹和创意工坊扫描、预览
packaging/              Debian、Flatpak 和 Arch 打包配置
scripts/                构建和开发脚本
```

Flutter 负责界面、设置、系统托盘和应用生命周期。Rust 服务在同一应用进程中扫描壁纸并运行播放引擎，无需单独启动守护进程。

## 许可证

VarPaper 使用 [GNU GPL v3 only](LICENSE) 许可证。上游项目归属信息见 [NOTICE](NOTICE)。保留的 [Apache-2.0](LICENSE-APACHE) 和 [MIT](LICENSE-MIT) 许可证文本仅适用于上游材料，不会替代 VarPaper 的 GPL 许可证。
