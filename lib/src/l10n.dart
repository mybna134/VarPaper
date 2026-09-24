import 'package:flutter/widgets.dart';

class WayvidLocalizations {
  const WayvidLocalizations(this.locale);

  final Locale locale;

  bool get isChinese => locale.languageCode == 'zh';

  static WayvidLocalizations of(BuildContext context) =>
      Localizations.of<WayvidLocalizations>(context, WayvidLocalizations) ??
      const WayvidLocalizations(Locale('en'));

  String text(String english) => isChinese ? _zh[english] ?? english : english;

  static const Map<String, String> _zh = {
    'Library': '壁纸库',
    'Folders': '文件夹',
    'Monitors': '显示器',
    'Settings': '设置',
    'About': '关于',
    'Show labels': '显示文本',
    'Hide labels': '隐藏文本',
    'Search wallpapers': '搜索壁纸',
    'Close search': '关闭搜索',
    'More': '更多',
    'Workshop wallpapers': 'Workshop 壁纸',
    'Local wallpapers': '本地壁纸',
    'Scene': '场景',
    'Video': '视频',
    'No wallpapers found. Add a folder to begin.': '没有找到壁纸，请先添加文件夹。',
    'Rescan wallpapers': '重新扫描壁纸',
    'Close details': '关闭详情',
    'By': '作者',
    'Apply to all monitors': '应用到所有显示器',
    'Apply to': '应用到',
    'Unapply from all monitors': '从所有显示器取消应用',
    'Unapply from': '取消应用于',
    'Path': '路径',
    'No library folders configured.': '尚未配置壁纸文件夹。',
    'Scan folder': '扫描文件夹',
    'Add folder': '添加文件夹',
    'No displays detected.': '没有检测到显示器。',
    'Active': '运行中',
    'Clear wallpaper': '清除壁纸',
    'Appearance': '外观',
    'Theme': '主题',
    'Theme color': '主题色彩',
    'Color style': '配色风格',
    'Tonal spot': '柔和',
    'Fidelity': '忠实',
    'Content': '内容',
    'Neutral': '中性',
    'Vibrant': '鲜艳',
    'Expressive': '灵动',
    'Add custom color': '添加自定义颜色',
    'Remove custom color': '移除自定义颜色',
    'Custom color': '自定义颜色',
    'Hex color': '十六进制颜色',
    'Enter a 6-digit hex color': '请输入 6 位十六进制颜色',
    'Add color': '添加颜色',
    'Reset palette': '重置调色板',
    'Cancel': '取消',
    'System': '跟随系统',
    'Light': '浅色',
    'Dark': '深色',
    'Language': '语言',
    'English': 'English',
    '中文': '中文',
    'Minimize to tray': '最小化到托盘',
    'Start minimized': '启动时最小化',
    'Playback': '播放',
    'Volume': '音量',
    'Loop mode': '循环播放',
    'Power': '电源',
    'Pause on battery': '电池供电时暂停',
    'Pause on fullscreen applications': '全屏应用运行时暂停',
    'Launch at login': '登录时启动',
    'Restore last wallpaper': '恢复上次壁纸',
    'Project website': '项目主页',
    'Report an issue': '报告问题',
    'An animated wallpaper engine for Linux': '适用于 Linux 的动态壁纸引擎',
    'Flutter UI · Rust engine · flutter_rust_bridge service facade':
        'Flutter UI · Rust 引擎 · flutter_rust_bridge 服务门面',
    'Show VarPaper': '显示 VarPaper',
    'Previous wallpaper': '上一张壁纸',
    'Next wallpaper': '下一张壁纸',
    'Pause': '暂停',
    'Resume': '继续',
    'Mute': '静音',
    'Unmute': '取消静音',
    'Quit': '退出',
  };

  static const delegate = _WayvidLocalizationsDelegate();
}

class _WayvidLocalizationsDelegate
    extends LocalizationsDelegate<WayvidLocalizations> {
  const _WayvidLocalizationsDelegate();

  @override
  bool isSupported(Locale locale) =>
      const ['en', 'zh'].contains(locale.languageCode);

  @override
  Future<WayvidLocalizations> load(Locale locale) async =>
      WayvidLocalizations(locale);

  @override
  bool shouldReload(_WayvidLocalizationsDelegate old) => false;
}
