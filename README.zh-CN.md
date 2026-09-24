<div align="center">

<img src="packaging/varpaper.svg" alt="VarPaper 标志" width="100" height="100">

# VarPaper

适用于 Linux Wayland 和 X11 桌面的动态视频壁纸管理器

[![许可证](https://img.shields.io/badge/license-GPL--3.0--only-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

[项目主页](https://github.com/mybna134/VarPaper) • [发布版本](https://github.com/mybna134/VarPaper/releases)

[English](README.md) | [简体中文](README.zh-CN.md)

</div>

VarPaper 是一款 Flutter 桌面应用，使用 Rust 和 mpv 作为视频播放引擎。它会扫描你添加的文件夹，以及本机已安装的 Wallpaper Engine 创意工坊内容，供你在媒体库中选择壁纸并应用到一台或所有显示器。

## 功能

- 浏览本地视频，支持预览、搜索和来源筛选。
- 为不同显示器设置不同壁纸，也可以将同一壁纸应用到所有显示器。
- 查找本机已安装的 Wallpaper Engine 创意工坊视频壁纸。创意工坊内容由 Steam 管理和下载，VarPaper 不负责下载。
- 启动时恢复各显示器之前设置的壁纸。
- 通过系统托盘暂停、继续播放或更换壁纸。
- 在应用中配置播放、外观、开机启动和电池供电时暂停等选项。

mpv 会在可用时自动使用硬件解码。Wayland 环境需要合成器支持 `wlr-layer-shell`；即使系统中存在 XWayland，Wayland 会话仍使用 layer-shell。原生 X11 会话通过 RandR 检测显示器，并使用桌面类型的壁纸窗口；窗口的堆叠行为取决于窗口管理器和桌面图标管理器。

## 安装

Linux x86_64 版本以 `.deb` 和 `.flatpak` 格式发布，可在[发布页面](https://github.com/mybna134/VarPaper/releases)下载。

Debian 或 Ubuntu 用户可下载 `.deb` 文件并运行：

```bash
sudo apt install ./varpaper_*_amd64.deb
varpaper
```

安装 Flatpak 包：

```bash
flatpak install ./varpaper_*_x86_64.flatpak
flatpak run io.github.mybna134.varpaper
```

Flatpak 使用 GNOME 50 运行时，并可访问你的主目录，以读取本地视频和 Steam 创意工坊文件。

## 使用

在 **文件夹** 页面添加视频文件夹。在 **媒体库** 中浏览或搜索，然后双击壁纸即可将其应用到所有显示器。打开壁纸详情可以将其应用到指定显示器。在 **显示器** 页面查看当前壁纸分配，在 **设置** 页面调整启动、播放和电源选项。

应用会将设置和显示器壁纸分配保存在平台的应用支持目录中的 Isar 数据库里。旧版 YAML 设置不会自动迁移。如需登录后自动恢复壁纸，请在设置中启用 **登录时启动** 和 **恢复上次壁纸**。**最小化到托盘** 和 **启动时最小化** 分别控制窗口行为。

Wallpaper Engine 创意工坊内容必须先通过 Steam 安装到本机。VarPaper 会查找这些文件夹并播放视频项目；目前不支持网页和场景项目。

## 从源码构建

需要安装 Flutter、Rust，以及 GTK、mpv、Wayland、EGL、X11、RandR、XFixes 和应用指示器所需的 Linux 开发库。构建安装包还需要 `dpkg-deb`、`flatpak-builder`，以及 GNOME 50 Flatpak 运行时和 SDK。

```bash
git clone https://github.com/mybna134/VarPaper.git
cd VarPaper
./scripts/build-packages.sh
```

脚本会将两个安装包写入 `dist/`。如需构建本地 Linux 程序包，请运行 `./scripts/build-linux.sh`。开发时可运行 `flutter run -d linux` 和 `cargo test --workspace`。

## 项目结构

```text
lib/                    Flutter 应用和设置
linux/                  Flutter Linux runner 和 Rust 构建集成
rust/                   Flutter/Rust 桥接服务
crates/wayvid-engine/    Wayland/X11 壁纸播放
crates/wayvid-library/  文件夹和创意工坊扫描、预览
packaging/              Debian 和 Flatpak 打包配置
scripts/                构建和开发脚本
```

Flutter 负责界面、设置、系统托盘和应用生命周期。Rust 服务在同一应用进程中扫描壁纸并运行播放引擎，无需单独启动守护进程。

## 许可证

VarPaper 使用 [GNU GPL v3 only](LICENSE) 许可证。上游项目归属信息见 [NOTICE](NOTICE)。保留的 [Apache-2.0](LICENSE-APACHE) 和 [MIT](LICENSE-MIT) 许可证文本仅适用于上游材料，不会替代 VarPaper 的 GPL 许可证。
