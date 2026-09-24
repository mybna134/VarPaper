import 'package:isar_community/isar.dart';
import 'package:path_provider/path_provider.dart';

import 'dart:io';

import '../bridge_generated.dart/bridge.dart';

part 'settings_store.g.dart';

const defaultThemeColor = 0xff009688;
const defaultColorSchemeVariant = 'tonalSpot';
const supportedColorSchemeVariants = {
  'tonalSpot',
  'fidelity',
  'content',
  'neutral',
  'vibrant',
  'expressive',
};

@collection
class AppSettingsRecord {
  Id id = 0;

  int windowWidth = 1200;
  int windowHeight = 800;
  bool minimizeToTray = true;
  bool startMinimized = false;
  String theme = 'system';
  int themeColor = defaultThemeColor;
  List<int> customThemeColors = [];
  String colorSchemeVariant = defaultColorSchemeVariant;
  String language = 'en';
  String renderer = 'auto';
  bool sidebarCollapsed = false;
  bool detailPanelVisible = true;

  double volume = 0;
  int? fpsLimit;
  String? preferredMonitor;
  bool loopMode = true;
  bool shuffle = false;
  String layout = 'fill';
  String hwdec = 'auto';
  bool mute = true;
  double startTime = 0;
  double playbackRate = 1;
  String hdrMode = 'auto';
  String toneMappingAlgorithm = 'hable';
  double toneMappingParam = 1;
  String toneMappingMode = 'hybrid';
  bool toneMappingComputePeak = true;

  bool autostartEnabled = false;
  bool restoreLastWallpaper = true;
  bool pauseOnBattery = false;
  bool pauseOnFullscreen = true;
  int? batteryFpsLimit;
  List<String> libraryFolders = [];
}

@collection
class WallpaperAssignmentRecord {
  Id id = Isar.autoIncrement;

  @Index(unique: true)
  late String output;
  late String sourcePath;
  String? sourceId;
}

class GuiSettingsDto {
  const GuiSettingsDto({
    required this.windowWidth,
    required this.windowHeight,
    required this.minimizeToTray,
    required this.startMinimized,
    required this.theme,
    required this.themeColor,
    required this.customThemeColors,
    required this.colorSchemeVariant,
    required this.language,
    required this.renderer,
    required this.sidebarCollapsed,
    required this.detailPanelVisible,
  });

  final int windowWidth;
  final int windowHeight;
  final bool minimizeToTray;
  final bool startMinimized;
  final String theme;
  final int themeColor;
  final List<int> customThemeColors;
  final String colorSchemeVariant;
  final String language;
  final String renderer;
  final bool sidebarCollapsed;
  final bool detailPanelVisible;

  GuiSettingsDto copyWith({
    int? windowWidth,
    int? windowHeight,
    bool? minimizeToTray,
    bool? startMinimized,
    String? theme,
    int? themeColor,
    List<int>? customThemeColors,
    String? colorSchemeVariant,
    String? language,
    String? renderer,
    bool? sidebarCollapsed,
    bool? detailPanelVisible,
  }) => GuiSettingsDto(
    windowWidth: windowWidth ?? this.windowWidth,
    windowHeight: windowHeight ?? this.windowHeight,
    minimizeToTray: minimizeToTray ?? this.minimizeToTray,
    startMinimized: startMinimized ?? this.startMinimized,
    theme: theme ?? this.theme,
    themeColor: themeColor ?? this.themeColor,
    customThemeColors: customThemeColors ?? this.customThemeColors,
    colorSchemeVariant: colorSchemeVariant ?? this.colorSchemeVariant,
    language: language ?? this.language,
    renderer: renderer ?? this.renderer,
    sidebarCollapsed: sidebarCollapsed ?? this.sidebarCollapsed,
    detailPanelVisible: detailPanelVisible ?? this.detailPanelVisible,
  );
}

class PlaybackSettingsDto {
  const PlaybackSettingsDto({
    required this.volume,
    required this.fpsLimit,
    required this.preferredMonitor,
    required this.loopMode,
    required this.shuffle,
    required this.layout,
    required this.hwdec,
    required this.mute,
    required this.startTime,
    required this.playbackRate,
    required this.hdrMode,
    required this.toneMappingAlgorithm,
    required this.toneMappingParam,
    required this.toneMappingMode,
    required this.toneMappingComputePeak,
  });

  final double volume;
  final int? fpsLimit;
  final String? preferredMonitor;
  final bool loopMode;
  final bool shuffle;
  final String layout;
  final String hwdec;
  final bool mute;
  final double startTime;
  final double playbackRate;
  final String hdrMode;
  final String toneMappingAlgorithm;
  final double toneMappingParam;
  final String toneMappingMode;
  final bool toneMappingComputePeak;

  PlaybackSettingsDto copyWith({
    double? volume,
    int? fpsLimit,
    bool clearFpsLimit = false,
    String? preferredMonitor,
    bool? loopMode,
    bool? shuffle,
    String? layout,
    String? hwdec,
    bool? mute,
    double? startTime,
    double? playbackRate,
    String? hdrMode,
    String? toneMappingAlgorithm,
    double? toneMappingParam,
    String? toneMappingMode,
    bool? toneMappingComputePeak,
  }) => PlaybackSettingsDto(
    volume: volume ?? this.volume,
    fpsLimit: clearFpsLimit ? null : (fpsLimit ?? this.fpsLimit),
    preferredMonitor: preferredMonitor ?? this.preferredMonitor,
    loopMode: loopMode ?? this.loopMode,
    shuffle: shuffle ?? this.shuffle,
    layout: layout ?? this.layout,
    hwdec: hwdec ?? this.hwdec,
    mute: mute ?? this.mute,
    startTime: startTime ?? this.startTime,
    playbackRate: playbackRate ?? this.playbackRate,
    hdrMode: hdrMode ?? this.hdrMode,
    toneMappingAlgorithm: toneMappingAlgorithm ?? this.toneMappingAlgorithm,
    toneMappingParam: toneMappingParam ?? this.toneMappingParam,
    toneMappingMode: toneMappingMode ?? this.toneMappingMode,
    toneMappingComputePeak:
        toneMappingComputePeak ?? this.toneMappingComputePeak,
  );
}

class PowerSettingsDto {
  const PowerSettingsDto({
    required this.pauseOnBattery,
    required this.pauseOnFullscreen,
    required this.batteryFpsLimit,
  });

  final bool pauseOnBattery;
  final bool pauseOnFullscreen;
  final int? batteryFpsLimit;

  PowerSettingsDto copyWith({
    bool? pauseOnBattery,
    bool? pauseOnFullscreen,
    int? batteryFpsLimit,
  }) => PowerSettingsDto(
    pauseOnBattery: pauseOnBattery ?? this.pauseOnBattery,
    pauseOnFullscreen: pauseOnFullscreen ?? this.pauseOnFullscreen,
    batteryFpsLimit: batteryFpsLimit ?? this.batteryFpsLimit,
  );
}

class SettingsDto {
  const SettingsDto({
    required this.gui,
    required this.playback,
    required this.autostartEnabled,
    required this.restoreLastWallpaper,
    required this.power,
    required this.libraryFolders,
  });

  final GuiSettingsDto gui;
  final PlaybackSettingsDto playback;
  final bool autostartEnabled;
  final bool restoreLastWallpaper;
  final PowerSettingsDto power;
  final List<String> libraryFolders;

  factory SettingsDto.defaults() => SettingsDto(
    gui: const GuiSettingsDto(
      windowWidth: 1200,
      windowHeight: 800,
      minimizeToTray: true,
      startMinimized: false,
      theme: 'system',
      themeColor: defaultThemeColor,
      customThemeColors: [],
      colorSchemeVariant: defaultColorSchemeVariant,
      language: 'en',
      renderer: 'auto',
      sidebarCollapsed: false,
      detailPanelVisible: true,
    ),
    playback: const PlaybackSettingsDto(
      volume: 0,
      fpsLimit: null,
      preferredMonitor: null,
      loopMode: true,
      shuffle: false,
      layout: 'fill',
      hwdec: 'auto',
      mute: true,
      startTime: 0,
      playbackRate: 1,
      hdrMode: 'auto',
      toneMappingAlgorithm: 'hable',
      toneMappingParam: 1,
      toneMappingMode: 'hybrid',
      toneMappingComputePeak: true,
    ),
    autostartEnabled: false,
    restoreLastWallpaper: true,
    power: const PowerSettingsDto(
      pauseOnBattery: false,
      pauseOnFullscreen: true,
      batteryFpsLimit: null,
    ),
    libraryFolders: [],
  );

  EngineConfigDto toEngineConfig() => EngineConfigDto(
    volume: playback.volume,
    fpsLimit: playback.fpsLimit,
    loopPlayback: playback.loopMode,
    layout: playback.layout,
    hwdec: playback.hwdec,
    mute: playback.mute,
    startTime: playback.startTime,
    playbackRate: playback.playbackRate,
    hdrMode: playback.hdrMode,
    toneMappingAlgorithm: playback.toneMappingAlgorithm,
    toneMappingParam: playback.toneMappingParam,
    toneMappingMode: playback.toneMappingMode,
    toneMappingComputePeak: playback.toneMappingComputePeak,
    autoPlay: true,
    pauseOnBattery: power.pauseOnBattery,
  );

  SettingsDto apply(SettingsPatch patch) => SettingsDto(
    gui: gui.copyWith(
      windowWidth: patch.windowWidth,
      windowHeight: patch.windowHeight,
      minimizeToTray: patch.minimizeToTray,
      startMinimized: patch.startMinimized,
      theme: patch.theme,
      themeColor: patch.themeColor,
      customThemeColors: patch.customThemeColors,
      colorSchemeVariant: patch.colorSchemeVariant,
      language: patch.language,
      renderer: patch.renderer,
      sidebarCollapsed: patch.sidebarCollapsed,
      detailPanelVisible: patch.detailPanelVisible,
    ),
    playback: playback.copyWith(
      volume: patch.volume,
      loopMode: patch.loopMode,
      mute: patch.mute,
      clearFpsLimit: patch.fpsLimit?.isUnlimited ?? false,
      fpsLimit: patch.fpsLimit?.value,
    ),
    autostartEnabled: patch.autostartEnabled ?? autostartEnabled,
    restoreLastWallpaper: patch.restoreLastWallpaper ?? restoreLastWallpaper,
    power: power.copyWith(
      pauseOnBattery: patch.pauseOnBattery,
      pauseOnFullscreen: patch.pauseOnFullscreen,
    ),
    libraryFolders: patch.libraryFolders ?? libraryFolders,
  );
}

class SettingsPatch {
  const SettingsPatch({
    this.windowWidth,
    this.windowHeight,
    this.minimizeToTray,
    this.startMinimized,
    this.theme,
    this.themeColor,
    this.customThemeColors,
    this.colorSchemeVariant,
    this.language,
    this.renderer,
    this.sidebarCollapsed,
    this.detailPanelVisible,
    this.volume,
    this.loopMode,
    this.mute,
    this.fpsLimit,
    this.pauseOnBattery,
    this.pauseOnFullscreen,
    this.autostartEnabled,
    this.restoreLastWallpaper,
    this.libraryFolders,
  });

  final int? windowWidth;
  final int? windowHeight;
  final bool? minimizeToTray;
  final bool? startMinimized;
  final String? theme;
  final int? themeColor;
  final List<int>? customThemeColors;
  final String? colorSchemeVariant;
  final String? language;
  final String? renderer;
  final bool? sidebarCollapsed;
  final bool? detailPanelVisible;
  final double? volume;
  final bool? loopMode;
  final bool? mute;
  final FpsLimitPatch? fpsLimit;
  final bool? pauseOnBattery;
  final bool? pauseOnFullscreen;
  final bool? autostartEnabled;
  final bool? restoreLastWallpaper;
  final List<String>? libraryFolders;
}

class FpsLimitPatch {
  const FpsLimitPatch({this.value, this.isUnlimited = false});
  const FpsLimitPatch.unlimited() : value = null, isUnlimited = true;

  final int? value;
  final bool isUnlimited;
}

class SettingsStore {
  SettingsStore(this.isar);

  final Isar isar;

  static Future<SettingsStore> open() async {
    final directory = await getApplicationSupportDirectory();
    final isar = await Isar.open(
      [AppSettingsRecordSchema, WallpaperAssignmentRecordSchema],
      directory: directory.path,
      name: 'wayvid',
    );
    return SettingsStore(isar);
  }

  Future<SettingsDto> load() async {
    final record = await isar.appSettingsRecords.get(0);
    if (record == null) return SettingsDto.defaults();
    return _fromRecord(record);
  }

  Future<void> save(SettingsDto settings) async {
    final record = _toRecord(settings);
    await isar.writeTxn(() => isar.appSettingsRecords.put(record));
  }

  Future<void> syncAutostart(bool enabled) async {
    final home = Platform.environment['HOME'];
    if (home == null || home.isEmpty) return;
    final file = File('$home/.config/autostart/wayvid.desktop');
    if (!enabled) {
      if (await file.exists()) await file.delete();
      return;
    }
    await file.parent.create(recursive: true);
    final executable = Platform.resolvedExecutable.replaceAll('"', '\\"');
    await file.writeAsString(
      '[Desktop Entry]\n'
      'Type=Application\n'
      'Name=Wayvid\n'
      'Exec="$executable"\n'
      'X-GNOME-Autostart-enabled=true\n',
    );
  }

  Future<void> saveAssignment({
    required String output,
    required String sourcePath,
    String? sourceId,
  }) async {
    final current =
        await isar.wallpaperAssignmentRecords
            .filter()
            .outputEqualTo(output)
            .findFirst() ??
        (WallpaperAssignmentRecord()..output = output);
    current.sourcePath = sourcePath;
    current.sourceId = sourceId;
    await isar.writeTxn(() => isar.wallpaperAssignmentRecords.put(current));
  }

  Future<Map<String, WallpaperAssignmentRecord>> loadAssignments() async {
    final records = await isar.wallpaperAssignmentRecords.where().findAll();
    return {for (final record in records) record.output: record};
  }

  Future<void> removeAssignment(String output) async {
    await isar.writeTxn(
      () => isar.wallpaperAssignmentRecords.deleteByOutput(output),
    );
  }

  Future<void> removeAllAssignments() async {
    await isar.writeTxn(() => isar.wallpaperAssignmentRecords.clear());
  }

  Future<void> saveDefaultAssignment({
    required String sourcePath,
    String? sourceId,
  }) async {
    await isar.writeTxn(() async {
      await isar.wallpaperAssignmentRecords.clear();
      await isar.wallpaperAssignmentRecords.put(
        WallpaperAssignmentRecord()
          ..output = 'all'
          ..sourcePath = sourcePath
          ..sourceId = sourceId,
      );
    });
  }

  AppSettingsRecord _toRecord(SettingsDto value) => AppSettingsRecord()
    ..id = 0
    ..windowWidth = value.gui.windowWidth
    ..windowHeight = value.gui.windowHeight
    ..minimizeToTray = value.gui.minimizeToTray
    ..startMinimized = value.gui.startMinimized
    ..theme = value.gui.theme
    ..themeColor = value.gui.themeColor
    ..customThemeColors = value.gui.customThemeColors.toList()
    ..colorSchemeVariant = value.gui.colorSchemeVariant
    ..language = value.gui.language
    ..renderer = value.gui.renderer
    ..sidebarCollapsed = value.gui.sidebarCollapsed
    ..detailPanelVisible = value.gui.detailPanelVisible
    ..volume = value.playback.volume
    ..fpsLimit = value.playback.fpsLimit
    ..preferredMonitor = value.playback.preferredMonitor
    ..loopMode = value.playback.loopMode
    ..shuffle = value.playback.shuffle
    ..layout = value.playback.layout
    ..hwdec = value.playback.hwdec
    ..mute = value.playback.mute
    ..startTime = value.playback.startTime
    ..playbackRate = value.playback.playbackRate
    ..hdrMode = value.playback.hdrMode
    ..toneMappingAlgorithm = value.playback.toneMappingAlgorithm
    ..toneMappingParam = value.playback.toneMappingParam
    ..toneMappingMode = value.playback.toneMappingMode
    ..toneMappingComputePeak = value.playback.toneMappingComputePeak
    ..autostartEnabled = value.autostartEnabled
    ..restoreLastWallpaper = value.restoreLastWallpaper
    ..pauseOnBattery = value.power.pauseOnBattery
    ..pauseOnFullscreen = value.power.pauseOnFullscreen
    ..batteryFpsLimit = value.power.batteryFpsLimit
    ..libraryFolders = value.libraryFolders.toList();

  SettingsDto _fromRecord(AppSettingsRecord value) => SettingsDto(
    gui: GuiSettingsDto(
      windowWidth: value.windowWidth,
      windowHeight: value.windowHeight,
      minimizeToTray: value.minimizeToTray,
      startMinimized: value.startMinimized,
      theme: value.theme,
      themeColor: value.themeColor == 0 ? defaultThemeColor : value.themeColor,
      customThemeColors: value.customThemeColors,
      colorSchemeVariant:
          supportedColorSchemeVariants.contains(value.colorSchemeVariant)
          ? value.colorSchemeVariant
          : defaultColorSchemeVariant,
      language: value.language,
      renderer: value.renderer,
      sidebarCollapsed: value.sidebarCollapsed,
      detailPanelVisible: value.detailPanelVisible,
    ),
    playback: PlaybackSettingsDto(
      volume: value.volume,
      fpsLimit: value.fpsLimit,
      preferredMonitor: value.preferredMonitor,
      loopMode: value.loopMode,
      shuffle: value.shuffle,
      layout: value.layout,
      hwdec: value.hwdec,
      mute: value.mute,
      startTime: value.startTime,
      playbackRate: value.playbackRate,
      hdrMode: value.hdrMode,
      toneMappingAlgorithm: value.toneMappingAlgorithm,
      toneMappingParam: value.toneMappingParam,
      toneMappingMode: value.toneMappingMode,
      toneMappingComputePeak: value.toneMappingComputePeak,
    ),
    autostartEnabled: value.autostartEnabled,
    restoreLastWallpaper: value.restoreLastWallpaper,
    power: PowerSettingsDto(
      pauseOnBattery: value.pauseOnBattery,
      pauseOnFullscreen: value.pauseOnFullscreen,
      batteryFpsLimit: value.batteryFpsLimit,
    ),
    libraryFolders: value.libraryFolders.toList(),
  );
}
