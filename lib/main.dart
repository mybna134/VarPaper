import 'dart:async';
import 'dart:io';
import 'dart:math' as math;

import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';
import 'package:flutter_colorpicker/flutter_colorpicker.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:menu_base/menu_base.dart' show Menu, MenuItem;
import 'package:shimmer/shimmer.dart';
import 'package:tray_manager/tray_manager.dart' hide MenuItem;
import 'package:window_manager/window_manager.dart';

import 'src/bridge_generated.dart/bridge.dart';
import 'src/bridge_generated.dart/frb_generated.dart';
import 'src/l10n.dart';
import 'src/storage/settings_store.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await windowManager.ensureInitialized();
  final settingsStore = await SettingsStore.open();
  await RustLib.init();

  final controller = await WayvidController.create(settingsStore);
  final settings = controller.settings;
  final options = WindowOptions(
    size: Size(
      settings.gui.windowWidth.toDouble(),
      settings.gui.windowHeight.toDouble(),
    ),
    minimumSize: const Size(800, 600),
    center: true,
    title: 'VarPaper',
    skipTaskbar: settings.gui.startMinimized,
  );
  await windowManager.waitUntilReadyToShow(options, () async {
    if (!settings.gui.startMinimized) {
      await windowManager.show();
      await windowManager.focus();
    }
  });

  runApp(
    ProviderScope(
      overrides: [wayvidControllerProvider.overrideWith((ref) => controller)],
      child: const WayvidApp(),
    ),
  );
}

final wayvidControllerProvider = ChangeNotifierProvider<WayvidController>(
  (ref) => throw StateError('WayvidController must be overridden'),
);

const _presetThemeColors = <int>[
  defaultThemeColor,
  0xff795548,
  0xff03a9f4,
  0xffffc107,
  0xff8bc34a,
  0xffe91e63,
  0xff673ab7,
];

const _colorSchemeVariants = <String, DynamicSchemeVariant>{
  'tonalSpot': DynamicSchemeVariant.tonalSpot,
  'fidelity': DynamicSchemeVariant.fidelity,
  'content': DynamicSchemeVariant.content,
  'neutral': DynamicSchemeVariant.neutral,
  'vibrant': DynamicSchemeVariant.vibrant,
  'expressive': DynamicSchemeVariant.expressive,
};

/// Builds the app's Material theme from the saved palette and brightness.
ThemeData themeFromSettings(GuiSettingsDto gui, Brightness brightness) {
  final variant =
      _colorSchemeVariants[gui.colorSchemeVariant] ??
      DynamicSchemeVariant.tonalSpot;
  return ThemeData(
    colorScheme: ColorScheme.fromSeed(
      seedColor: Color(gui.themeColor),
      brightness: brightness,
      dynamicSchemeVariant: variant,
    ),
    useMaterial3: true,
  );
}

class WayvidController extends ChangeNotifier {
  WayvidController._(
    this.service,
    this.settingsStore,
    this.settings,
    this.engineRunning,
    this.workshopAvailable,
  );

  final WayvidService service;
  final SettingsStore settingsStore;
  SettingsDto settings;
  List<WallpaperDto> wallpapers = [];
  List<MonitorDto> monitors = [];
  bool engineRunning;
  bool manuallyPaused = false;
  final bool workshopAvailable;
  bool get anotherInstance => false;
  String? error;
  String search = '';
  bool showLocalWallpapers = true;
  bool showWorkshopWallpapers = true;
  Set<String> wallpaperCategories = {'scene', 'video'};
  String? selectedWallpaperId;
  String page = 'library';
  Timer? eventTimer;
  bool _pollingEvents = false;
  final Map<String, Future<PreviewDto?>> _thumbnailFutures = {};
  final Map<String, WallpaperAssignmentRecord> _assignments = {};

  /// Wallpaper id currently applied to each output, keyed by output name.
  final Map<String, String> appliedByOutput = {};

  Set<String> get appliedWallpaperIds => appliedByOutput.values.toSet();

  bool isAppliedTo(String wallpaperId, String output) =>
      appliedByOutput[output] == wallpaperId;

  bool isAppliedEverywhere(String wallpaperId) =>
      monitors.isNotEmpty &&
      monitors.every((monitor) => isAppliedTo(wallpaperId, monitor.name));

  void navigate(String value) {
    page = value;
    notifyListeners();
  }

  void setSearch(String value) {
    search = value;
    notifyListeners();
  }

  void setSourceVisibility({bool? local, bool? workshop}) {
    showLocalWallpapers = local ?? showLocalWallpapers;
    showWorkshopWallpapers = workshop ?? showWorkshopWallpapers;
    notifyListeners();
  }

  void setWallpaperCategories(Set<String> values) {
    if (values.isEmpty) return;
    wallpaperCategories = {...values};
    notifyListeners();
  }

  void selectWallpaper(String value) {
    selectedWallpaperId = value;
    notifyListeners();
  }

  static Future<WayvidController> create(
    SettingsStore settingsStore, {
    WayvidService? serviceOverride,
  }) async {
    final settings = await settingsStore.load();
    await settingsStore.save(settings);
    final service = serviceOverride ?? await WayvidService.newInstance();
    final info = await service.initialize();
    final controller = WayvidController._(
      service,
      settingsStore,
      settings,
      info.engineRunning,
      info.workshopAvailable,
    );
    if (!controller.engineRunning) {
      try {
        await service.createEngine(config: settings.toEngineConfig());
        controller.engineRunning = true;
      } catch (exception) {
        controller.error = exception.toString();
      }
    }
    await controller.refresh();
    if (settings.restoreLastWallpaper) {
      controller._assignments.addAll(await settingsStore.loadAssignments());
      await controller._restoreOutputs(
        controller.monitors.map((monitor) => monitor.name),
      );
    }
    controller.eventTimer = Timer.periodic(const Duration(milliseconds: 250), (
      _,
    ) {
      controller.pollEvents();
    });
    return controller;
  }

  Future<void> refresh() async {
    await _run(() async {
      wallpapers = [];
      _thumbnailFutures.clear();
      monitors = await service.refreshMonitors();
      for (final folder in settings.libraryFolders) {
        _mergeWallpapers(await service.scanFolder(path: folder));
      }
      if (workshopAvailable) {
        _mergeWallpapers(await service.scanWorkshop());
      }
    });
  }

  Future<void> scanFolder(String path) async {
    await _run(() async {
      _mergeWallpapers(await service.scanFolder(path: path));
      if (!settings.libraryFolders.contains(path)) {
        settings = settings.apply(
          SettingsPatch(libraryFolders: [...settings.libraryFolders, path]),
        );
        await settingsStore.save(settings);
      }
    });
  }

  Future<void> pollEvents() async {
    if (_pollingEvents) return;
    _pollingEvents = true;
    try {
      for (final event in await service.pollEvents()) {
        if (event is ServiceEvent_EngineStarted) engineRunning = true;
        if (event is ServiceEvent_EngineStopped) {
          engineRunning = false;
          manuallyPaused = false;
        }
        if (event is ServiceEvent_OutputsChanged) {
          final previous = monitors.map((monitor) => monitor.name).toSet();
          monitors = event.outputs;
          final connected = monitors.map((monitor) => monitor.name).toSet();
          appliedByOutput.removeWhere((name, _) => !connected.contains(name));
          await _restoreOutputs(connected.difference(previous));
        }
        if (event is ServiceEvent_Error) error = event.message;
      }
      notifyListeners();
    } catch (_) {
      // Transient engine shutdowns are retried by the next poll.
    } finally {
      _pollingEvents = false;
    }
  }

  Future<void> _restoreOutputs(Iterable<String> outputs) async {
    for (final output in outputs) {
      if (appliedByOutput.containsKey(output)) continue;
      final assignment = _assignments[output] ?? _assignments['all'];
      if (assignment == null || assignment.sourcePath.isEmpty) continue;
      try {
        if (await FileSystemEntity.type(assignment.sourcePath) ==
            FileSystemEntityType.notFound) {
          continue;
        }
        await service.applyWallpaper(
          path: assignment.sourcePath,
          output: output,
        );
        final matching = wallpapers.where(
          (item) => item.sourcePath == assignment.sourcePath,
        );
        final id =
            assignment.sourceId ??
            (matching.isEmpty ? null : matching.first.id);
        if (id != null) appliedByOutput[output] = id;
      } catch (exception) {
        error = exception.toString();
      }
    }
    notifyListeners();
  }

  Future<void> apply(String wallpaperId, {String? output}) async {
    await _run(() async {
      final wallpaper = wallpapers.firstWhere((item) => item.id == wallpaperId);
      if (!engineRunning) {
        await service.createEngine(config: settings.toEngineConfig());
        engineRunning = true;
      }
      await service.applyWallpaper(path: wallpaper.sourcePath, output: output);
      if (manuallyPaused) await service.pause(output: output);
      if (output == null) {
        await settingsStore.saveDefaultAssignment(
          sourcePath: wallpaper.sourcePath,
          sourceId: wallpaper.id,
        );
        _assignments.clear();
        _assignments['all'] = WallpaperAssignmentRecord()
          ..output = 'all'
          ..sourcePath = wallpaper.sourcePath
          ..sourceId = wallpaper.id;
        appliedByOutput.clear();
        for (final monitor in monitors) {
          appliedByOutput[monitor.name] = wallpaper.id;
        }
      } else {
        await settingsStore.saveAssignment(
          output: output,
          sourcePath: wallpaper.sourcePath,
          sourceId: wallpaper.id,
        );
        _assignments[output] = WallpaperAssignmentRecord()
          ..output = output
          ..sourcePath = wallpaper.sourcePath
          ..sourceId = wallpaper.id;
        appliedByOutput[output] = wallpaper.id;
      }
    });
  }

  Future<void> clear({String? output}) async {
    await _run(() async {
      await service.clearWallpaper(output: output);
      if (output == null) {
        await settingsStore.removeAllAssignments();
        _assignments.clear();
        appliedByOutput.clear();
      } else {
        if (_assignments.containsKey('all')) {
          await settingsStore.saveAssignment(output: output, sourcePath: '');
          _assignments[output] = WallpaperAssignmentRecord()
            ..output = output
            ..sourcePath = '';
        } else {
          await settingsStore.removeAssignment(output);
          _assignments.remove(output);
        }
        appliedByOutput.remove(output);
      }
    });
  }

  Future<void> setPaused(bool paused) async {
    if (manuallyPaused == paused || !engineRunning) return;
    await _run(() async {
      if (paused) {
        await service.pause();
      } else {
        await service.resume();
      }
      manuallyPaused = paused;
    });
  }

  Future<void> cycleWallpaper(int direction) async {
    if (wallpapers.isEmpty || monitors.isEmpty) return;
    final ordered = [...wallpapers]..sort((a, b) => a.name.compareTo(b.name));
    final current = monitors
        .map((monitor) => appliedByOutput[monitor.name])
        .toSet();
    if (current.length == 1) {
      final id = current.single;
      final index = ordered.indexWhere((item) => item.id == id);
      final next = index < 0
          ? (direction > 0 ? 0 : ordered.length - 1)
          : (index + direction + ordered.length) % ordered.length;
      await apply(ordered[next].id);
      return;
    }
    for (final monitor in monitors) {
      final index = ordered.indexWhere(
        (item) => item.id == appliedByOutput[monitor.name],
      );
      final next = index < 0
          ? (direction > 0 ? 0 : ordered.length - 1)
          : (index + direction + ordered.length) % ordered.length;
      await apply(ordered[next].id, output: monitor.name);
      if (error != null) return;
    }
  }

  Future<void> update(SettingsPatch patch) async {
    await _run(() async {
      settings = settings.apply(patch);
      await settingsStore.save(settings);
      if (patch.autostartEnabled != null) {
        await settingsStore.syncAutostart(patch.autostartEnabled!);
      }
      if (engineRunning &&
          (patch.volume != null ||
              patch.loopMode != null ||
              patch.mute != null ||
              patch.fpsLimit != null ||
              patch.pauseOnBattery != null)) {
        await service.updateEngineConfig(config: settings.toEngineConfig());
      }
    });
  }

  Future<PreviewDto?> thumbnail(WallpaperDto wallpaper) {
    return _thumbnailFutures.putIfAbsent(
      wallpaper.id,
      () => _loadThumbnail(wallpaper),
    );
  }

  Future<PreviewDto?> _loadThumbnail(WallpaperDto wallpaper) async {
    try {
      return await service.loadPreview(
        wallpaperId: wallpaper.id,
        path: wallpaper.sourcePath,
        wallpaperType: wallpaper.wallpaperType,
        width: 640,
        height: 360,
      );
    } catch (_) {
      return null;
    }
  }

  Future<void> openUrl(String url) async =>
      _run(() => service.openUrl(url: url));

  Future<void> shutdown() async {
    eventTimer?.cancel();
    await service.shutdown();
  }

  List<WallpaperDto> get visibleWallpapers {
    final query = search.trim().toLowerCase();
    return wallpapers.where((wallpaper) {
      final isLocal =
          wallpaper.sourceType == 'local_file' ||
          wallpaper.sourceType == 'local_dir';
      final isWorkshop =
          wallpaper.sourceType == 'workshop' ||
          wallpaper.sourceType == 'steam_workshop';
      final sourceMatches =
          (isLocal && showLocalWallpapers) ||
          (isWorkshop && showWorkshopWallpapers);
      final categoryMatches = wallpaperCategories.contains(
        wallpaper.wallpaperCategory,
      );
      final text =
          '${wallpaper.name} ${wallpaper.metadata.title ?? ''} ${wallpaper.metadata.tags.join(' ')}'
              .toLowerCase();
      return sourceMatches &&
          categoryMatches &&
          (query.isEmpty || text.contains(query));
    }).toList();
  }

  void _mergeWallpapers(List<WallpaperDto> values) {
    final byId = {for (final item in wallpapers) item.id: item};
    for (final item in values) {
      byId[item.id] = item;
    }
    wallpapers = byId.values.toList()..sort((a, b) => a.name.compareTo(b.name));
  }

  Future<void> _run(Future<void> Function() action) async {
    error = null;
    notifyListeners();
    try {
      await action();
    } on FrbException catch (exception) {
      error = exception.toString();
    } catch (exception) {
      error = exception.toString();
    }
    notifyListeners();
  }
}

class WayvidApp extends ConsumerStatefulWidget {
  const WayvidApp({super.key});

  @override
  ConsumerState<WayvidApp> createState() => _WayvidAppState();
}

class _WayvidAppState extends ConsumerState<WayvidApp>
    with WindowListener, TrayListener {
  Object? _traySignature;
  Future<void> _trayUpdate = Future.value();

  @override
  void initState() {
    super.initState();
    windowManager.addListener(this);
    trayManager.addListener(this);
    ref.read(wayvidControllerProvider).addListener(_refreshTray);
    _initTray();
  }

  @override
  void dispose() {
    windowManager.removeListener(this);
    trayManager.removeListener(this);
    ref.read(wayvidControllerProvider).removeListener(_refreshTray);
    trayManager.destroy();
    ref.read(wayvidControllerProvider).shutdown();
    super.dispose();
  }

  Future<void> _initTray() async {
    try {
      await trayManager.setIcon('packaging/varpaper.png');
      await trayManager.setToolTip('VarPaper');
      _refreshTray();
    } catch (_) {
      // Tray support is optional on desktops without an AppIndicator host.
    }
  }

  void _refreshTray() {
    if (!mounted) return;
    final controller = ref.read(wayvidControllerProvider);
    final signature = (
      controller.settings.gui.language,
      controller.settings.playback.mute,
      controller.manuallyPaused,
      controller.engineRunning,
      controller.wallpapers.isNotEmpty,
      controller.monitors.isNotEmpty,
    );
    if (_traySignature == signature) return;
    _traySignature = signature;
    _trayUpdate = _trayUpdate.then((_) async {
      if (!mounted) return;
      final l10n = WayvidLocalizations(
        Locale(controller.settings.gui.language),
      );
      final available =
          controller.engineRunning && controller.monitors.isNotEmpty;
      try {
        await trayManager.setContextMenu(
          Menu(
            items: [
              MenuItem(
                key: 'show',
                label: l10n.text('Show VarPaper'),
                onClick: (_) async {
                  await windowManager.show();
                  await windowManager.focus();
                },
              ),
              MenuItem(
                key: 'library',
                label: l10n.text('Library'),
                onClick: (_) async {
                  controller.navigate('library');
                  await windowManager.show();
                  await windowManager.focus();
                },
              ),
              MenuItem(
                key: 'settings',
                label: l10n.text('Settings'),
                onClick: (_) async {
                  controller.navigate('settings');
                  await windowManager.show();
                  await windowManager.focus();
                },
              ),
              MenuItem.separator(),
              MenuItem(
                key: 'previous',
                label: l10n.text('Previous wallpaper'),
                disabled: !available || controller.wallpapers.isEmpty,
                onClick: (_) => controller.cycleWallpaper(-1),
              ),
              MenuItem(
                key: 'next',
                label: l10n.text('Next wallpaper'),
                disabled: !available || controller.wallpapers.isEmpty,
                onClick: (_) => controller.cycleWallpaper(1),
              ),
              MenuItem(
                key: 'pause',
                label: l10n.text(
                  controller.manuallyPaused ? 'Resume' : 'Pause',
                ),
                disabled: !available,
                onClick: (_) =>
                    controller.setPaused(!controller.manuallyPaused),
              ),
              MenuItem(
                key: 'mute',
                label: l10n.text(
                  controller.settings.playback.mute ? 'Unmute' : 'Mute',
                ),
                disabled: !controller.engineRunning,
                onClick: (_) => controller.update(
                  SettingsPatch(mute: !controller.settings.playback.mute),
                ),
              ),
              MenuItem.separator(),
              MenuItem(
                key: 'quit',
                label: l10n.text('Quit'),
                onClick: (_) async {
                  await controller.shutdown();
                  await windowManager.destroy();
                },
              ),
            ],
          ),
        );
      } catch (_) {
        // Some desktops do not expose an AppIndicator host.
      }
    });
  }

  @override
  void onTrayIconMouseDown() async {
    if (await windowManager.isVisible()) {
      await windowManager.hide();
    } else {
      await windowManager.show();
      await windowManager.focus();
    }
  }

  @override
  Future<void> onWindowClose() async {
    final controller = ref.read(wayvidControllerProvider);
    if (controller.settings.gui.minimizeToTray) {
      await windowManager.hide();
    } else {
      await controller.shutdown();
      await windowManager.destroy();
    }
  }

  @override
  Widget build(BuildContext context) {
    final controller = ref.watch(wayvidControllerProvider);
    final gui = controller.settings.gui;
    final themeMode = switch (gui.theme) {
      'dark' => ThemeMode.dark,
      'light' => ThemeMode.light,
      _ => ThemeMode.system,
    };
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      title: 'VarPaper',
      locale: Locale(controller.settings.gui.language),
      supportedLocales: const [Locale('en'), Locale('zh')],
      localizationsDelegates: const [
        WayvidLocalizations.delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
      ],
      themeMode: themeMode,
      theme: themeFromSettings(gui, Brightness.light),
      darkTheme: themeFromSettings(gui, Brightness.dark),
      home: WayvidShell(controller: controller),
    );
  }
}

class WayvidShell extends StatelessWidget {
  const WayvidShell({required this.controller, super.key});
  final WayvidController controller;

  @override
  Widget build(BuildContext context) => Scaffold(
    body: Row(
      children: [
        _Sidebar(controller: controller),
        const VerticalDivider(width: 1),
        Expanded(child: _Content(controller: controller)),
      ],
    ),
  );
}

class _Sidebar extends StatelessWidget {
  const _Sidebar({required this.controller});
  final WayvidController controller;

  @override
  Widget build(BuildContext context) {
    final l10n = WayvidLocalizations.of(context);
    final labelsHidden = controller.settings.gui.sidebarCollapsed;
    const pages = ['library', 'folders', 'monitors', 'settings', 'about'];
    final destinations = [
      NavigationRailDestination(
        icon: Icon(Icons.video_library_outlined),
        selectedIcon: Icon(Icons.video_library),
        label: Text(l10n.text('Library')),
      ),
      NavigationRailDestination(
        icon: Icon(Icons.folder_outlined),
        selectedIcon: Icon(Icons.folder),
        label: Text(l10n.text('Folders')),
      ),
      NavigationRailDestination(
        icon: Icon(Icons.desktop_windows_outlined),
        selectedIcon: Icon(Icons.desktop_windows),
        label: Text(l10n.text('Monitors')),
      ),
      NavigationRailDestination(
        icon: Icon(Icons.settings_outlined),
        selectedIcon: Icon(Icons.settings),
        label: Text(l10n.text('Settings')),
      ),
      NavigationRailDestination(
        icon: Icon(Icons.info_outline),
        selectedIcon: Icon(Icons.info),
        label: Text(l10n.text('About')),
      ),
    ];
    final selectedIndex = pages
        .indexOf(controller.page)
        .clamp(0, pages.length - 1);
    // The rail always stays compact: icons are stacked vertically and the
    // text label sits directly below each icon. `sidebarCollapsed` is reused
    // as "labels hidden" so the persisted setting keeps its schema.
    const width = 88.0;

    return SizedBox(
      width: width,
      child: Column(
        children: [
          Expanded(
            child: Padding(
              padding: const EdgeInsets.only(top: 16),
              child: NavigationRail(
                minWidth: width,
                labelType: labelsHidden
                    ? NavigationRailLabelType.none
                    : NavigationRailLabelType.all,
                selectedIndex: selectedIndex,
                onDestinationSelected: (index) =>
                    controller.navigate(pages[index]),
                destinations: destinations,
              ),
            ),
          ),
          IconButton(
            tooltip: l10n.text(labelsHidden ? 'Show labels' : 'Hide labels'),
            onPressed: () => controller.update(
              SettingsPatch(sidebarCollapsed: !labelsHidden),
            ),
            icon: const Icon(Icons.menu),
          ),
          const SizedBox(height: 8),
        ],
      ),
    );
  }
}

class CommonScaffold extends StatelessWidget {
  const CommonScaffold({
    required this.title,
    required this.body,
    this.actions = const [],
    this.headerBottom = const SizedBox.shrink(),
    super.key,
  });

  final String title;
  final Widget body;
  final List<Widget> actions;
  final Widget headerBottom;

  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      Padding(
        padding: const EdgeInsets.fromLTRB(28, 22, 28, 14),
        child: Row(
          children: [
            Text(title, style: Theme.of(context).textTheme.headlineMedium),
            if (actions.isNotEmpty) ...[const Spacer(), ...actions],
          ],
        ),
      ),
      headerBottom,
      Expanded(child: body),
    ],
  );
}

/// Square slot shared by every header action so icons line up regardless of
/// the platform visual density applied to [IconButton].
const double _kActionSlot = 48;

class _AnimatedSearchAction extends StatelessWidget {
  const _AnimatedSearchAction({
    required this.visible,
    required this.controller,
    required this.hintText,
    required this.closeTooltip,
    required this.searchTooltip,
    required this.onToggle,
    required this.onChanged,
  });

  static const double _expandedWidth = 320;

  final bool visible;
  final TextEditingController controller;
  final String hintText;
  final String closeTooltip;
  final String searchTooltip;
  final VoidCallback onToggle;
  final ValueChanged<String> onChanged;

  @override
  Widget build(BuildContext context) => AnimatedContainer(
    duration: const Duration(milliseconds: 250),
    curve: Curves.easeOutCubic,
    width: visible ? _expandedWidth : _kActionSlot,
    height: _kActionSlot,
    clipBehavior: Clip.hardEdge,
    decoration: const BoxDecoration(),
    // The field is always laid out at its full width and anchored to the
    // left edge of the container. As the container grows the left edge moves
    // outward, so the search icon appears to slide into the prefix position.
    child: Stack(
      alignment: Alignment.centerLeft,
      children: [
        if (visible)
          Positioned(
            left: 0,
            top: 0,
            bottom: 0,
            width: _expandedWidth,
            child: Center(
              child: TextField(
                controller: controller,
                autofocus: true,
                onChanged: onChanged,
                decoration: InputDecoration(
                  isDense: true,
                  contentPadding: const EdgeInsets.symmetric(vertical: 10),
                  prefixIcon: const Icon(Icons.search),
                  prefixIconConstraints: const BoxConstraints(
                    minWidth: _kActionSlot,
                    minHeight: 40,
                  ),
                  suffixIcon: IconButton(
                    tooltip: closeTooltip,
                    onPressed: onToggle,
                    icon: const Icon(Icons.close),
                  ),
                  suffixIconConstraints: const BoxConstraints(
                    minWidth: _kActionSlot,
                    minHeight: 40,
                  ),
                  hintText: hintText,
                  border: const OutlineInputBorder(),
                ),
              ),
            ),
          )
        else
          SizedBox.square(
            dimension: _kActionSlot,
            child: Center(
              child: IconButton(
                tooltip: searchTooltip,
                onPressed: onToggle,
                icon: const Icon(Icons.search),
              ),
            ),
          ),
      ],
    ),
  );
}

class _MoreAction extends StatefulWidget {
  const _MoreAction({required this.controller, required this.l10n});

  final WayvidController controller;
  final WayvidLocalizations l10n;

  @override
  State<_MoreAction> createState() => _MoreActionState();
}

class _MoreActionState extends State<_MoreAction> {
  final _buttonKey = GlobalKey();

  Future<void> _openPopup() async {
    final button = _buttonKey.currentContext?.findRenderObject() as RenderBox?;
    final overlay = Navigator.of(context).overlay;
    final overlayBox = overlay?.context.findRenderObject() as RenderBox?;
    if (button == null || overlayBox == null) return;

    final topLeft = button.localToGlobal(Offset.zero, ancestor: overlayBox);
    final viewPadding = MediaQuery.viewPaddingOf(context);
    // Anchor the popup's top-right corner just below the button.
    final offset = topLeft.translate(
      button.size.width + viewPadding.right,
      button.size.height + 4 + viewPadding.top,
    );
    await Navigator.of(context).push<void>(
      _LibraryPopupRoute(
        offset: offset,
        child: _MorePopup(controller: widget.controller, l10n: widget.l10n),
      ),
    );
  }

  @override
  Widget build(BuildContext context) => SizedBox.square(
    key: _buttonKey,
    dimension: _kActionSlot,
    child: Center(
      child: IconButton(
        tooltip: widget.l10n.text('More'),
        onPressed: _openPopup,
        icon: const Icon(Icons.more_vert),
      ),
    ),
  );
}

class _MorePopup extends StatelessWidget {
  const _MorePopup({required this.controller, required this.l10n});

  final WayvidController controller;
  final WayvidLocalizations l10n;

  @override
  Widget build(BuildContext context) => SizedBox(
    width: 260,
    child: Card(
      elevation: 12,
      color: Theme.of(context).colorScheme.surfaceContainer,
      clipBehavior: Clip.antiAlias,
      shape: RoundedSuperellipseBorder(borderRadius: BorderRadius.circular(14)),
      child: AnimatedBuilder(
        animation: controller,
        builder: (context, _) => SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              CheckboxListTile(
                value: controller.showWorkshopWallpapers,
                onChanged: (value) =>
                    controller.setSourceVisibility(workshop: value),
                controlAffinity: ListTileControlAffinity.leading,
                dense: true,
                title: Text(l10n.text('Workshop wallpapers')),
              ),
              CheckboxListTile(
                value: controller.showLocalWallpapers,
                onChanged: (value) =>
                    controller.setSourceVisibility(local: value),
                controlAffinity: ListTileControlAffinity.leading,
                dense: true,
                title: Text(l10n.text('Local wallpapers')),
              ),
            ],
          ),
        ),
      ),
    ),
  );
}

class _LibraryPopupRoute extends PopupRoute<void> {
  _LibraryPopupRoute({required this.offset, required this.child});

  final Offset offset;
  final Widget child;

  @override
  Color? get barrierColor => null;

  @override
  bool get barrierDismissible => true;

  @override
  String? get barrierLabel => 'Dismiss';

  @override
  Duration get transitionDuration => const Duration(milliseconds: 250);

  @override
  Widget buildPage(
    BuildContext context,
    Animation<double> animation,
    Animation<double> secondaryAnimation,
  ) => child;

  @override
  Widget buildTransitions(
    BuildContext context,
    Animation<double> animation,
    Animation<double> secondaryAnimation,
    Widget child,
  ) {
    final curveAnimation = animation.drive(
      CurveTween(curve: Curves.easeOutBack),
    );
    return SafeArea(
      child: CustomSingleChildLayout(
        delegate: _PopupLayoutDelegate(offset: offset),
        child: FadeTransition(
          opacity: curveAnimation,
          child: ScaleTransition(
            alignment: Alignment.topRight,
            scale: curveAnimation,
            child: SlideTransition(
              position: curveAnimation.drive(
                Tween(begin: const Offset(0, -0.02), end: Offset.zero),
              ),
              child: child,
            ),
          ),
        ),
      ),
    );
  }
}

class _PopupLayoutDelegate extends SingleChildLayoutDelegate {
  const _PopupLayoutDelegate({required this.offset});

  static const double _margin = 16;
  static const double _maxWidth = 320;

  final Offset offset;

  @override
  Size getSize(BoxConstraints constraints) => constraints.biggest;

  @override
  BoxConstraints getConstraintsForChild(BoxConstraints constraints) {
    // The overlay hands us tight full-screen constraints; without loosening
    // them the popup card is forced to fill the whole window.
    final size = constraints.biggest;
    final maxWidth = (size.width - _margin * 2).clamp(0.0, _maxWidth);
    final maxHeight = (size.height - offset.dy - _margin).clamp(
      0.0,
      math.max(0.0, size.height - _margin * 2),
    );
    return BoxConstraints(
      maxWidth: maxWidth.toDouble(),
      maxHeight: maxHeight.toDouble(),
    );
  }

  @override
  Offset getPositionForChild(Size size, Size childSize) {
    final x = (offset.dx - childSize.width).clamp(
      0.0,
      math.max(0.0, size.width - _margin - childSize.width),
    );
    final y = offset.dy.clamp(
      0.0,
      math.max(0.0, size.height - _margin - childSize.height),
    );
    return Offset(x.toDouble(), y.toDouble());
  }

  @override
  bool shouldRelayout(covariant _PopupLayoutDelegate oldDelegate) =>
      oldDelegate.offset != offset;
}

class _Content extends StatefulWidget {
  const _Content({required this.controller});
  final WayvidController controller;

  @override
  State<_Content> createState() => _ContentState();
}

class _ContentState extends State<_Content> {
  late final TextEditingController _searchController;
  bool _searchVisible = false;

  WayvidController get controller => widget.controller;

  @override
  void initState() {
    super.initState();
    _searchController = TextEditingController(text: controller.search);
  }

  @override
  void dispose() {
    _searchController.dispose();
    super.dispose();
  }

  void _toggleSearch() {
    setState(() => _searchVisible = !_searchVisible);
    if (_searchVisible) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) {
          _searchController.selection = TextSelection.collapsed(
            offset: _searchController.text.length,
          );
        }
      });
    }
  }

  Widget _searchAction(WayvidLocalizations l10n) => _AnimatedSearchAction(
    visible: _searchVisible,
    controller: _searchController,
    hintText: l10n.text('Search wallpapers'),
    closeTooltip: l10n.text('Close search'),
    searchTooltip: l10n.text('Search wallpapers'),
    onToggle: _toggleSearch,
    onChanged: controller.setSearch,
  );

  Widget _moreAction(WayvidLocalizations l10n) =>
      _MoreAction(controller: controller, l10n: l10n);

  @override
  Widget build(BuildContext context) {
    final l10n = WayvidLocalizations.of(context);
    const titles = {
      'library': 'Library',
      'folders': 'Folders',
      'monitors': 'Monitors',
      'settings': 'Settings',
      'about': 'About',
    };
    final isLibrary = controller.page == 'library';
    return CommonScaffold(
      title: l10n.text(titles[controller.page] ?? 'VarPaper'),
      actions: isLibrary ? [_searchAction(l10n), _moreAction(l10n)] : const [],
      headerBottom: isLibrary
          ? Padding(
              padding: const EdgeInsets.fromLTRB(28, 0, 28, 12),
              child: SegmentedButton<String>(
                segments: [
                  ButtonSegment<String>(
                    value: 'scene',
                    label: Text(l10n.text('Scene')),
                    icon: Icon(Icons.auto_awesome_motion_outlined),
                  ),
                  ButtonSegment<String>(
                    value: 'video',
                    label: Text(l10n.text('Video')),
                    icon: Icon(Icons.movie_outlined),
                  ),
                ],
                selected: controller.wallpaperCategories,
                multiSelectionEnabled: true,
                emptySelectionAllowed: false,
                onSelectionChanged: controller.setWallpaperCategories,
              ),
            )
          : const SizedBox.shrink(),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (controller.anotherInstance)
            const _Notice(
              icon: Icons.info_outline,
              text: 'VarPaper is already running; this window is a secondary instance.',
            ),
          if (controller.error != null)
            _Notice(
              icon: Icons.error_outline,
              text: controller.error!,
              error: true,
            ),
          Expanded(
            child: switch (controller.page) {
              'folders' => _FoldersPage(controller: controller),
              'monitors' => _MonitorsPage(controller: controller),
              'settings' => _SettingsPage(controller: controller),
              'about' => _AboutPage(controller: controller),
              _ => _LibraryPage(controller: controller),
            },
          ),
        ],
      ),
    );
  }
}

class _Notice extends StatelessWidget {
  const _Notice({required this.icon, required this.text, this.error = false});
  final IconData icon;
  final String text;
  final bool error;

  @override
  Widget build(BuildContext context) => Container(
    width: double.infinity,
    margin: const EdgeInsets.symmetric(horizontal: 28, vertical: 4),
    padding: const EdgeInsets.all(12),
    color: error
        ? Theme.of(context).colorScheme.errorContainer
        : Theme.of(context).colorScheme.secondaryContainer,
    child: Row(
      children: [
        Icon(icon),
        const SizedBox(width: 10),
        Expanded(child: Text(text)),
      ],
    ),
  );
}

class _LibraryPage extends StatelessWidget {
  const _LibraryPage({required this.controller});
  final WayvidController controller;

  @override
  Widget build(BuildContext context) {
    final l10n = WayvidLocalizations.of(context);
    return Stack(
      children: [
        Column(
          children: [
            Expanded(
              child: controller.visibleWallpapers.isEmpty
                  ? Center(
                      child: Text(
                        l10n.text(
                          'No wallpapers found. Add a folder to begin.',
                        ),
                      ),
                    )
                  : GridView.builder(
                      padding: const EdgeInsets.fromLTRB(28, 0, 28, 100),
                      gridDelegate:
                          const SliverGridDelegateWithMaxCrossAxisExtent(
                            maxCrossAxisExtent: 270,
                            mainAxisExtent: 220,
                            crossAxisSpacing: 14,
                            mainAxisSpacing: 14,
                          ),
                      itemCount: controller.visibleWallpapers.length,
                      itemBuilder: (_, index) => _WallpaperCard(
                        controller: controller,
                        wallpaper: controller.visibleWallpapers[index],
                      ),
                    ),
            ),
          ],
        ),
        Positioned(
          right: 28,
          bottom: 28,
          child: FloatingActionButton(
            onPressed: controller.refresh,
            tooltip: l10n.text('Rescan wallpapers'),
            child: const Icon(Icons.refresh),
          ),
        ),
      ],
    );
  }
}

class _WallpaperCard extends StatelessWidget {
  const _WallpaperCard({required this.controller, required this.wallpaper});
  final WayvidController controller;
  final WallpaperDto wallpaper;

  @override
  Widget build(BuildContext context) => Card(
    clipBehavior: Clip.antiAlias,
    color: controller.appliedWallpaperIds.contains(wallpaper.id)
        ? Theme.of(context).colorScheme.secondaryContainer
        : null,
    child: InkWell(
      splashFactory: NoSplash.splashFactory,
      onTapDown: (_) {
        controller.selectWallpaper(wallpaper.id);
        if (ModalRoute.of(context)?.isCurrent ?? true) {
          _showWallpaperDetails(context, controller, wallpaper);
        }
      },
      onDoubleTap: () => controller.apply(wallpaper.id),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Expanded(
            child: Stack(
              fit: StackFit.expand,
              children: [
                FutureBuilder<PreviewDto?>(
                  future: controller.thumbnail(wallpaper),
                  builder: (_, snapshot) =>
                      snapshot.connectionState == ConnectionState.waiting
                      ? Shimmer.fromColors(
                          baseColor: Theme.of(context)
                              .colorScheme
                              .surfaceContainerHighest,
                          highlightColor: Theme.of(context).colorScheme.surface,
                          child: Container(
                            color: Theme.of(context)
                                .colorScheme
                                .surfaceContainerHighest,
                          ),
                        )
                      : snapshot.data == null
                      ? Container(
                          color: Theme.of(context)
                              .colorScheme
                              .surfaceContainerHighest,
                        )
                      : snapshot.data!.imagePath != null
                      ? Image.file(
                          File(snapshot.data!.imagePath!),
                          fit: BoxFit.cover,
                          width: double.infinity,
                        )
                      : Image.memory(
                          snapshot.data!.bytes,
                          fit: BoxFit.cover,
                          width: double.infinity,
                        ),
                ),
                Positioned(
                  right: 8,
                  bottom: 8,
                  child: _WallpaperCategoryBadge(
                    category: wallpaper.wallpaperCategory,
                  ),
                ),
              ],
            ),
          ),
          Padding(
            padding: const EdgeInsets.fromLTRB(12, 9, 12, 2),
            child: Text(
              wallpaper.name,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: const TextStyle(fontWeight: FontWeight.w600),
            ),
          ),
          Padding(
            padding: const EdgeInsets.fromLTRB(12, 0, 12, 9),
            child: Text(
              wallpaper.sourceType,
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ),
        ],
      ),
    ),
  );
}

/// Small icon badge in the preview corner showing whether a wallpaper is a
/// scene or a video, matching the icons used by the category filter.
class _WallpaperCategoryBadge extends StatelessWidget {
  const _WallpaperCategoryBadge({required this.category});
  final String category;

  @override
  Widget build(BuildContext context) {
    final l10n = WayvidLocalizations.of(context);
    final isVideo = category == 'video';
    return Tooltip(
      message: l10n.text(isVideo ? 'Video' : 'Scene'),
      child: Container(
        padding: const EdgeInsets.all(4),
        decoration: BoxDecoration(
          color: Colors.black.withValues(alpha: 0.55),
          borderRadius: BorderRadius.circular(6),
        ),
        child: Icon(
          isVideo ? Icons.movie_outlined : Icons.auto_awesome_motion_outlined,
          size: 16,
          color: Colors.white,
        ),
      ),
    );
  }
}

void _showWallpaperDetails(
  BuildContext context,
  WayvidController controller,
  WallpaperDto wallpaper,
) {
  final screenWidth = MediaQuery.sizeOf(context).width;
  final panelWidth = screenWidth < 480 ? screenWidth * 0.9 : 420.0;
  showGeneralDialog<void>(
    context: context,
    barrierDismissible: true,
    barrierLabel: MaterialLocalizations.of(context).modalBarrierDismissLabel,
    barrierColor: Colors.black54,
    transitionDuration: const Duration(milliseconds: 260),
    pageBuilder: (context, animation, secondaryAnimation) => Align(
      alignment: Alignment.centerRight,
      child: SafeArea(
        child: SizedBox(
          width: panelWidth,
          height: double.infinity,
          child: _Details(controller: controller, wallpaper: wallpaper),
        ),
      ),
    ),
    transitionBuilder: (context, animation, secondaryAnimation, child) {
      final curved = CurvedAnimation(
        parent: animation,
        curve: Curves.easeOutCubic,
        reverseCurve: Curves.easeInCubic,
      );
      return SlideTransition(
        position: Tween<Offset>(
          begin: const Offset(1, 0),
          end: Offset.zero,
        ).animate(curved),
        child: child,
      );
    },
  );
}

class _Details extends StatelessWidget {
  const _Details({required this.controller, required this.wallpaper});
  final WayvidController controller;
  final WallpaperDto wallpaper;

  @override
  Widget build(BuildContext context) {
    final l10n = WayvidLocalizations.of(context);
    final metadata = wallpaper.metadata;
    return Material(
      color: Theme.of(context).colorScheme.surface,
      elevation: 16,
      child: Padding(
        padding: const EdgeInsets.fromLTRB(22, 18, 22, 22),
        child: ListView(
          children: [
            Row(
              children: [
                Expanded(
                  child: Text(
                    wallpaper.name,
                    style: Theme.of(context).textTheme.titleLarge,
                  ),
                ),
                IconButton(
                  onPressed: () => Navigator.of(context).pop(),
                  tooltip: l10n.text('Close details'),
                  icon: const Icon(Icons.close),
                ),
              ],
            ),
            const SizedBox(height: 12),
            if (metadata.author != null)
              Text('${l10n.text('By')}: ${metadata.author}'),
            if (metadata.description != null)
              Padding(
                padding: const EdgeInsets.only(top: 10),
                child: Text(metadata.description!),
              ),
            const SizedBox(height: 18),
            AnimatedBuilder(
              animation: controller,
              builder: (context, _) {
                final everywhere = controller.isAppliedEverywhere(wallpaper.id);
                return Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    everywhere
                        ? FilledButton.tonalIcon(
                            onPressed: controller.clear,
                            icon: const Icon(Icons.wallpaper_outlined),
                            label: Text(l10n.text('Unapply from all monitors')),
                          )
                        : FilledButton.icon(
                            onPressed: () => controller.apply(wallpaper.id),
                            icon: const Icon(Icons.wallpaper),
                            label: Text(l10n.text('Apply to all monitors')),
                          ),
                    const SizedBox(height: 16),
                    for (final monitor in controller.monitors)
                      Padding(
                        padding: const EdgeInsets.only(bottom: 12),
                        child:
                            controller.isAppliedTo(wallpaper.id, monitor.name)
                            ? OutlinedButton.icon(
                                onPressed: () =>
                                    controller.clear(output: monitor.name),
                                icon: const Icon(Icons.close),
                                label: Text(
                                  '${l10n.text('Unapply from')} ${monitor.name}',
                                ),
                              )
                            : OutlinedButton(
                                onPressed: () => controller.apply(
                                  wallpaper.id,
                                  output: monitor.name,
                                ),
                                child: Text(
                                  '${l10n.text('Apply to')} ${monitor.name}',
                                ),
                              ),
                      ),
                  ],
                );
              },
            ),
            const Divider(height: 28),
            Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Icon(
                  Icons.location_on_outlined,
                  size: 20,
                  semanticLabel: l10n.text('Path'),
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: SelectableText(
                    wallpaper.sourcePath,
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

class _FoldersPage extends StatelessWidget {
  const _FoldersPage({required this.controller});
  final WayvidController controller;

  @override
  Widget build(BuildContext context) {
    final l10n = WayvidLocalizations.of(context);
    return Stack(
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(28, 28, 28, 100),
          child: controller.settings.libraryFolders.isEmpty
              ? Center(child: Text(l10n.text('No library folders configured.')))
              : ListView(
                  children: [
                    for (final folder in controller.settings.libraryFolders)
                      Card(
                        child: ListTile(
                          leading: const Icon(Icons.folder),
                          title: Text(folder),
                          trailing: IconButton(
                            onPressed: () => controller.scanFolder(folder),
                            tooltip: l10n.text('Scan folder'),
                            icon: const Icon(Icons.refresh),
                          ),
                        ),
                      ),
                  ],
                ),
        ),
        Positioned(
          right: 28,
          bottom: 28,
          child: FloatingActionButton.extended(
            onPressed: () async {
              final path = await getDirectoryPath(
                confirmButtonText: l10n.text('Add folder'),
              );
              if (path != null) await controller.scanFolder(path);
            },
            icon: const Icon(Icons.add),
            label: Text(l10n.text('Add folder')),
          ),
        ),
      ],
    );
  }
}

class _MonitorsPage extends StatelessWidget {
  const _MonitorsPage({required this.controller});
  final WayvidController controller;

  @override
  Widget build(BuildContext context) {
    final l10n = WayvidLocalizations.of(context);
    return Padding(
      padding: const EdgeInsets.all(28),
      child: controller.monitors.isEmpty
          ? Center(child: Text(l10n.text('No displays detected.')))
          : ListView(
              children: [
                for (final monitor in controller.monitors)
                  Card(
                    child: ListTile(
                      leading: Icon(
                        monitor.primary ? Icons.star : Icons.desktop_windows,
                      ),
                      title: Text(monitor.name),
                      subtitle: Text(
                        '${monitor.width} × ${monitor.height}  ·  scale ${monitor.scale}',
                      ),
                      trailing: Row(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          if (monitor.currentWallpaper != null)
                            Chip(label: Text(l10n.text('Active'))),
                          IconButton(
                            onPressed: () =>
                                controller.clear(output: monitor.name),
                            tooltip: l10n.text('Clear wallpaper'),
                            icon: const Icon(Icons.clear),
                          ),
                        ],
                      ),
                    ),
                  ),
              ],
            ),
    );
  }
}

class _SettingsPage extends StatelessWidget {
  const _SettingsPage({required this.controller});
  final WayvidController controller;

  @override
  Widget build(BuildContext context) {
    final l10n = WayvidLocalizations.of(context);
    final gui = controller.settings.gui;
    final playback = controller.settings.playback;
    final power = controller.settings.power;
    return ListView(
      padding: const EdgeInsets.fromLTRB(28, 0, 28, 28),
      children: [
        Text(
          l10n.text('Appearance'),
          style: Theme.of(context).textTheme.titleLarge,
        ),
        _SettingsRow(
          title: l10n.text('Theme'),
          control: _DecoratedDropdown<String>(
            value: gui.theme,
            items: [
              DropdownMenuItem(
                value: 'system',
                child: Text(l10n.text('System')),
              ),
              DropdownMenuItem(value: 'light', child: Text(l10n.text('Light'))),
              DropdownMenuItem(value: 'dark', child: Text(l10n.text('Dark'))),
            ],
            onChanged: (value) {
              if (value != null) controller.update(SettingsPatch(theme: value));
            },
          ),
        ),
        _SettingsRow(
          title: l10n.text('Theme color'),
          below: _PaletteSettings(controller: controller),
        ),
        _SettingsRow(
          title: l10n.text('Color style'),
          control: _DecoratedDropdown<String>(
            value: _colorSchemeVariants.containsKey(gui.colorSchemeVariant)
                ? gui.colorSchemeVariant
                : defaultColorSchemeVariant,
            items: [
              for (final variant in _colorSchemeVariants.keys)
                DropdownMenuItem(
                  value: variant,
                  child: Text(l10n.text(_variantLabel(variant))),
                ),
            ],
            onChanged: (value) {
              if (value != null) {
                controller.update(SettingsPatch(colorSchemeVariant: value));
              }
            },
          ),
        ),
        _SettingsRow(
          title: l10n.text('Language'),
          control: _DecoratedDropdown<String>(
            value: gui.language == 'zh' ? 'zh' : 'en',
            items: [
              DropdownMenuItem(value: 'en', child: Text(l10n.text('English'))),
              DropdownMenuItem(value: 'zh', child: Text(l10n.text('中文'))),
            ],
            onChanged: (value) {
              if (value != null) {
                controller.update(SettingsPatch(language: value));
              }
            },
          ),
        ),
        _SettingsRow(
          title: l10n.text('Minimize to tray'),
          control: Switch(
            value: gui.minimizeToTray,
            onChanged: (value) =>
                controller.update(SettingsPatch(minimizeToTray: value)),
          ),
        ),
        _SettingsRow(
          title: l10n.text('Start minimized'),
          control: Switch(
            value: gui.startMinimized,
            onChanged: (value) =>
                controller.update(SettingsPatch(startMinimized: value)),
          ),
        ),
        Text(
          l10n.text('Playback'),
          style: Theme.of(context).textTheme.titleLarge,
        ),
        _SettingsRow(
          title: l10n.text('Volume'),
          below: Slider(
            // Opt in to the Material 3 (2024) slider: tall track with a gap
            // around the narrow handle and a rounded value indicator. The
            // flag is deprecated but is the only supported opt-in until the
            // framework flips the default.
            // ignore: deprecated_member_use
            year2023: false,
            value: playback.volume,
            divisions: 100,
            label: '${(playback.volume * 100).round()}%',
            onChanged: (value) =>
                controller.update(SettingsPatch(volume: value)),
          ),
        ),
        _SettingsRow(
          title: l10n.text('Loop mode'),
          control: Switch(
            value: playback.loopMode,
            onChanged: (value) =>
                controller.update(SettingsPatch(loopMode: value)),
          ),
        ),
        const Divider(height: 36),
        Text(l10n.text('Power'), style: Theme.of(context).textTheme.titleLarge),
        _SettingsRow(
          title: l10n.text('Pause on battery'),
          control: Switch(
            value: power.pauseOnBattery,
            onChanged: (value) =>
                controller.update(SettingsPatch(pauseOnBattery: value)),
          ),
        ),
        _SettingsRow(
          title: l10n.text('Pause on fullscreen applications'),
          control: Switch(
            value: power.pauseOnFullscreen,
            onChanged: (value) =>
                controller.update(SettingsPatch(pauseOnFullscreen: value)),
          ),
        ),
        _SettingsRow(
          title: l10n.text('Launch at login'),
          control: Switch(
            value: controller.settings.autostartEnabled,
            onChanged: (value) =>
                controller.update(SettingsPatch(autostartEnabled: value)),
          ),
        ),
        _SettingsRow(
          title: l10n.text('Restore last wallpaper'),
          control: Switch(
            value: controller.settings.restoreLastWallpaper,
            onChanged: (value) =>
                controller.update(SettingsPatch(restoreLastWallpaper: value)),
          ),
        ),
      ],
    );
  }
}

String _variantLabel(String variant) => switch (variant) {
  'fidelity' => 'Fidelity',
  'content' => 'Content',
  'neutral' => 'Neutral',
  'vibrant' => 'Vibrant',
  'expressive' => 'Expressive',
  _ => 'Tonal spot',
};

class _PaletteSettings extends StatelessWidget {
  const _PaletteSettings({required this.controller});

  final WayvidController controller;

  @override
  Widget build(BuildContext context) {
    final l10n = WayvidLocalizations.of(context);
    final gui = controller.settings.gui;
    final customColors = gui.customThemeColors
        .where((color) => !_presetThemeColors.contains(color))
        .toSet()
        .toList();
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: [
            for (final color in _presetThemeColors)
              _PaletteSwatch(
                color: color,
                selected: gui.themeColor == color,
                onPressed: () =>
                    controller.update(SettingsPatch(themeColor: color)),
              ),
            for (final color in customColors)
              _PaletteSwatch(
                color: color,
                selected: gui.themeColor == color,
                onPressed: () =>
                    controller.update(SettingsPatch(themeColor: color)),
                onDelete: () => controller.update(
                  SettingsPatch(
                    customThemeColors: customColors
                        .where((value) => value != color)
                        .toList(),
                    themeColor: gui.themeColor == color
                        ? defaultThemeColor
                        : null,
                  ),
                ),
              ),
            SizedBox(
              width: 56,
              height: 56,
              child: Center(
                child: IconButton.filledTonal(
                  tooltip: l10n.text('Add custom color'),
                  icon: const Icon(Icons.add),
                  onPressed: () async {
                    final color = await showDialog<int>(
                      context: context,
                      builder: (context) =>
                          _ColorPickerDialog(initialColor: gui.themeColor),
                    );
                    if (color == null) return;
                    controller.update(
                      SettingsPatch(
                        themeColor: color,
                        customThemeColors:
                            !_presetThemeColors.contains(color) &&
                                !customColors.contains(color)
                            ? [...customColors, color]
                            : null,
                      ),
                    );
                  },
                ),
              ),
            ),
          ],
        ),
        if (gui.themeColor != defaultThemeColor ||
            customColors.isNotEmpty ||
            gui.colorSchemeVariant != defaultColorSchemeVariant)
          TextButton.icon(
            onPressed: () => controller.update(
              const SettingsPatch(
                themeColor: defaultThemeColor,
                customThemeColors: [],
                colorSchemeVariant: defaultColorSchemeVariant,
              ),
            ),
            icon: const Icon(Icons.restart_alt),
            label: Text(l10n.text('Reset palette')),
          ),
      ],
    );
  }
}

class _PaletteSwatch extends StatelessWidget {
  const _PaletteSwatch({
    required this.color,
    required this.selected,
    required this.onPressed,
    this.onDelete,
  });

  final int color;
  final bool selected;
  final VoidCallback onPressed;
  final VoidCallback? onDelete;

  @override
  Widget build(BuildContext context) {
    final l10n = WayvidLocalizations.of(context);
    final swatchColor = Color(color);
    final onSwatch =
        ThemeData.estimateBrightnessForColor(swatchColor) == Brightness.dark
        ? Colors.white
        : Colors.black;
    return SizedBox(
      width: 56,
      height: 56,
      child: Stack(
        children: [
          Center(
            child: IconButton(
              tooltip:
                  '${l10n.text('Theme color')} '
                  '#${color.toRadixString(16).substring(2).toUpperCase()}',
              style: IconButton.styleFrom(
                backgroundColor: swatchColor,
                foregroundColor: onSwatch,
                side: BorderSide(
                  color: selected
                      ? Theme.of(context).colorScheme.onSurface
                      : Theme.of(context).colorScheme.outlineVariant,
                  width: selected ? 3 : 1,
                ),
              ),
              onPressed: onPressed,
              icon: Icon(selected ? Icons.check : null),
            ),
          ),
          if (onDelete != null)
            Positioned(
              right: 0,
              top: 0,
              child: SizedBox(
                width: 22,
                height: 22,
                child: IconButton.filledTonal(
                  tooltip: l10n.text('Remove custom color'),
                  padding: EdgeInsets.zero,
                  iconSize: 14,
                  onPressed: onDelete,
                  icon: const Icon(Icons.close),
                ),
              ),
            ),
        ],
      ),
    );
  }
}

class _ColorPickerDialog extends StatefulWidget {
  const _ColorPickerDialog({required this.initialColor});

  final int initialColor;

  @override
  State<_ColorPickerDialog> createState() => _ColorPickerDialogState();
}

class _ColorPickerDialogState extends State<_ColorPickerDialog> {
  late Color _color;

  @override
  void initState() {
    super.initState();
    _color = Color(widget.initialColor);
  }

  @override
  Widget build(BuildContext context) {
    final l10n = WayvidLocalizations.of(context);
    final previewText =
        ThemeData.estimateBrightnessForColor(_color) == Brightness.dark
        ? Colors.white
        : Colors.black;
    return AlertDialog(
      scrollable: true,
      title: Text(l10n.text('Custom color')),
      content: SizedBox(
        width: 320,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            ColorPicker(
              pickerColor: _color,
              onColorChanged: (color) => setState(() {
                _color = Color(color.toARGB32() | 0xff000000);
              }),
              enableAlpha: false,
              hexInputBar: true,
              labelTypes: const [ColorLabelType.rgb],
              portraitOnly: true,
              colorPickerWidth: 300,
              pickerAreaHeightPercent: 0.75,
              displayThumbColor: true,
              pickerAreaBorderRadius: BorderRadius.circular(12),
            ),
            Container(
              height: 48,
              alignment: Alignment.center,
              decoration: BoxDecoration(
                color: _color,
                borderRadius: BorderRadius.circular(12),
              ),
              child: Text(
                '#${_color.toARGB32().toRadixString(16).substring(2).toUpperCase()}',
                style: TextStyle(color: previewText),
              ),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: Text(l10n.text('Cancel')),
        ),
        FilledButton(
          onPressed: () => Navigator.pop(context, _color.toARGB32()),
          child: Text(l10n.text('Add color')),
        ),
      ],
    );
  }
}

/// A single settings row: a title on the left and an interactive [control]
/// on the right, or a full-width [below] widget under the title.
///
/// Unlike [ListTile] / [SwitchListTile], the row itself is not tappable —
/// only the control responds to input.
class _SettingsRow extends StatelessWidget {
  const _SettingsRow({required this.title, this.control, this.below})
    : assert(control == null || below == null);

  final String title;
  final Widget? control;
  final Widget? below;

  @override
  Widget build(BuildContext context) {
    final titleText = Text(title, style: Theme.of(context).textTheme.bodyLarge);
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 12),
      child: below != null
          ? Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [titleText, const SizedBox(height: 4), below!],
            )
          : Row(
              children: [
                Expanded(child: titleText),
                const SizedBox(width: 16),
                ?control,
              ],
            ),
    );
  }
}

/// A [DropdownButton] drawn inside an outlined, rounded [BoxDecoration]
/// instead of the default underline, for use as a settings row trailing.
class _DecoratedDropdown<T> extends StatelessWidget {
  const _DecoratedDropdown({
    required this.value,
    required this.items,
    required this.onChanged,
  });

  final T value;
  final List<DropdownMenuItem<T>> items;
  final ValueChanged<T?> onChanged;

  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12),
      decoration: BoxDecoration(
        color: colorScheme.surfaceContainerHighest,
        border: Border.all(color: colorScheme.outlineVariant),
        borderRadius: BorderRadius.circular(10),
      ),
      child: DropdownButtonHideUnderline(
        child: DropdownButton<T>(
          value: value,
          items: items,
          onChanged: onChanged,
          isDense: true,
          borderRadius: BorderRadius.circular(10),
          dropdownColor: colorScheme.surfaceContainer,
          padding: const EdgeInsets.symmetric(vertical: 8),
        ),
      ),
    );
  }
}

class _AboutPage extends StatelessWidget {
  const _AboutPage({required this.controller});
  final WayvidController controller;

  @override
  Widget build(BuildContext context) {
    final l10n = WayvidLocalizations.of(context);
    return Center(
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 560),
        child: Card(
          child: Padding(
            padding: const EdgeInsets.all(32),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Image.asset(
                  'packaging/varpaper.png',
                  width: 64,
                  height: 64,
                  semanticLabel: 'VarPaper',
                ),
                const SizedBox(height: 14),
                Text(
                  'VarPaper',
                  style: Theme.of(context).textTheme.headlineMedium,
                ),
                Text(l10n.text('An animated wallpaper engine for Linux')),
                const SizedBox(height: 22),
                Wrap(
                  spacing: 8,
                  children: [
                    OutlinedButton(
                      onPressed: () => controller.openUrl(
                        'https://github.com/mybna134/var_paper',
                      ),
                      child: Text(l10n.text('Project website')),
                    ),
                    OutlinedButton(
                      onPressed: () => controller.openUrl(
                        'https://github.com/mybna134/var_paper/issues',
                      ),
                      child: Text(l10n.text('Report an issue')),
                    ),
                  ],
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
