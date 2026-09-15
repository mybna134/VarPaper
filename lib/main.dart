import 'dart:async';

import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:window_manager/window_manager.dart';

import 'src/bridge_generated.dart/bridge.dart';
import 'src/bridge_generated.dart/frb_generated.dart';

Future<void> main(List<String> args) async {
  WidgetsFlutterBinding.ensureInitialized();
  await windowManager.ensureInitialized();
  await RustLib.init();

  final controller = await WayvidController.create(args);
  final settings = controller.settings;
  final options = WindowOptions(
    size: Size(
      settings.gui.windowWidth.toDouble(),
      settings.gui.windowHeight.toDouble(),
    ),
    minimumSize: const Size(800, 600),
    center: true,
    title: 'Wayvid',
    skipTaskbar: settings.gui.startMinimized,
  );
  await windowManager.waitUntilReadyToShow(options, () async {
    if (!settings.gui.startMinimized && !controller.anotherInstance) {
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

class WayvidController extends ChangeNotifier {
  WayvidController._(this.service, this.snapshot)
    : settings = snapshot.settings,
      engineRunning = snapshot.engineRunning,
      anotherInstance = snapshot.anotherInstance;

  final WayvidService service;
  final InitializationSnapshot snapshot;
  SettingsDto settings;
  List<WallpaperDto> wallpapers = [];
  List<MonitorDto> monitors = [];
  bool engineRunning;
  final bool anotherInstance;
  String? error;
  String search = '';
  String filter = 'all';
  String? selectedWallpaperId;
  String page = 'library';
  Timer? eventTimer;
  final Map<String, Future<Uint8List?>> _thumbnailFutures = {};

  void navigate(String value) {
    page = value;
    notifyListeners();
  }

  void setSearch(String value) {
    search = value;
    notifyListeners();
  }

  void setFilter(String value) {
    filter = value;
    notifyListeners();
  }

  void selectWallpaper(String value) {
    selectedWallpaperId = value;
    notifyListeners();
  }

  static Future<WayvidController> create(List<String> args) async {
    final service = await WayvidService.newInstance(args: args);
    final snapshot = await service.initialize();
    final controller = WayvidController._(service, snapshot);
    await controller.refresh();
    controller.eventTimer = Timer.periodic(const Duration(milliseconds: 250), (
      _,
    ) {
      controller.pollEvents();
    });
    return controller;
  }

  Future<void> refresh() async {
    await _run(() async {
      wallpapers = await service.loadLibrary();
      monitors = await service.refreshMonitors();
      if (snapshot.workshopAvailable) {
        _mergeWallpapers(await service.scanWorkshop());
      }
    });
  }

  Future<void> scanFolder(String path) async {
    await _run(() async {
      _mergeWallpapers(await service.scanFolder(path: path));
      settings = await service.getSettings();
    });
  }

  Future<void> pollEvents() async {
    try {
      for (final event in await service.pollEvents()) {
        if (event is ServiceEvent_EngineStarted) engineRunning = true;
        if (event is ServiceEvent_EngineStopped) engineRunning = false;
        if (event is ServiceEvent_OutputsChanged) monitors = event.outputs;
        if (event is ServiceEvent_Error) error = event.message;
        if (event is ServiceEvent_ShowWindow) {
          await windowManager.show();
          await windowManager.focus();
        }
        if (event is ServiceEvent_TrayAction) {
          if (event.action == 'show') {
            await windowManager.show();
            await windowManager.focus();
          } else if (event.action == 'hide') {
            await windowManager.hide();
          } else if (event.action == 'toggle_pause') {
            await (engineRunning ? service.pause() : service.resume());
          } else if (event.action == 'quit') {
            await shutdown();
            await windowManager.destroy();
          }
        }
      }
      notifyListeners();
    } catch (_) {
      // Transient engine shutdowns are retried by the next poll.
    }
  }

  Future<void> toggleEngine() async {
    await _run(() async {
      if (engineRunning) {
        await service.stopEngine();
      } else {
        await service.startEngine();
      }
      engineRunning = !engineRunning;
    });
  }

  Future<void> apply(String wallpaperId, {String? output}) async {
    await _run(() async {
      await service.applyWallpaper(wallpaperId: wallpaperId, output: output);
      // Applying a wallpaper starts the embedded Rust engine on demand.
      engineRunning = true;
    });
  }

  Future<void> clear({String? output}) async {
    await _run(() => service.clearWallpaper(output: output));
  }

  Future<void> update(SettingsPatch patch) async {
    await _run(() async {
      settings = await service.updateSettings(patch: patch);
    });
  }

  Future<Uint8List?> thumbnail(WallpaperDto wallpaper) {
    return _thumbnailFutures.putIfAbsent(
      wallpaper.id,
      () => _loadThumbnail(wallpaper),
    );
  }

  Future<Uint8List?> _loadThumbnail(WallpaperDto wallpaper) async {
    try {
      return await service.loadThumbnail(
        wallpaperId: wallpaper.id,
        path: wallpaper.sourcePath,
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
      final sourceMatches = filter == 'all' || wallpaper.sourceType == filter;
      final text =
          '${wallpaper.name} ${wallpaper.metadata.title ?? ''} ${wallpaper.metadata.tags.join(' ')}'
              .toLowerCase();
      return sourceMatches && (query.isEmpty || text.contains(query));
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

class _WayvidAppState extends ConsumerState<WayvidApp> with WindowListener {
  @override
  void initState() {
    super.initState();
    windowManager.addListener(this);
  }

  @override
  void dispose() {
    windowManager.removeListener(this);
    ref.read(wayvidControllerProvider).shutdown();
    super.dispose();
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
    final dark = controller.settings.gui.theme == 'dark';
    return MaterialApp(
      debugShowCheckedModeBanner: false,
      title: 'Wayvid',
      themeMode: dark ? ThemeMode.dark : ThemeMode.light,
      theme: ThemeData(
        colorSchemeSeed: Colors.teal,
        brightness: Brightness.light,
        useMaterial3: true,
      ),
      darkTheme: ThemeData(
        colorSchemeSeed: Colors.teal,
        brightness: Brightness.dark,
        useMaterial3: true,
      ),
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
    final collapsed = controller.settings.gui.sidebarCollapsed;
    const pages = ['library', 'folders', 'monitors', 'settings', 'about'];
    const destinations = [
      NavigationRailDestination(
        icon: Icon(Icons.video_library_outlined),
        selectedIcon: Icon(Icons.video_library),
        label: Text('Library'),
      ),
      NavigationRailDestination(
        icon: Icon(Icons.folder_outlined),
        selectedIcon: Icon(Icons.folder),
        label: Text('Folders'),
      ),
      NavigationRailDestination(
        icon: Icon(Icons.desktop_windows_outlined),
        selectedIcon: Icon(Icons.desktop_windows),
        label: Text('Monitors'),
      ),
      NavigationRailDestination(
        icon: Icon(Icons.settings_outlined),
        selectedIcon: Icon(Icons.settings),
        label: Text('Settings'),
      ),
      NavigationRailDestination(
        icon: Icon(Icons.info_outline),
        selectedIcon: Icon(Icons.info),
        label: Text('About'),
      ),
    ];
    final selectedIndex = pages
        .indexOf(controller.page)
        .clamp(0, pages.length - 1);
    // NavigationRail's internal item row needs a little more than the
    // nominal 72px Material minimum on some Flutter/Linux font metrics.
    // Keeping the collapsed rail at 80px prevents a fractional horizontal
    // overflow while preserving the compact layout.
    final width = collapsed ? 80.0 : 220.0;

    return AnimatedContainer(
      duration: const Duration(milliseconds: 180),
      curve: Curves.easeOut,
      width: width,
      clipBehavior: Clip.hardEdge,
      decoration: const BoxDecoration(),
      child: Column(
        children: [
          SizedBox(
            height: 72,
            child: Center(
              child: collapsed
                  ? const Icon(Icons.waves, size: 28)
                  : const Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Icon(Icons.waves, size: 28),
                        SizedBox(width: 10),
                        Text(
                          'Wayvid',
                          style: TextStyle(
                            fontSize: 20,
                            fontWeight: FontWeight.bold,
                          ),
                        ),
                      ],
                    ),
            ),
          ),
          Expanded(
            child: NavigationRail(
              extended: !collapsed,
              minWidth: 80,
              minExtendedWidth: 220,
              labelType: NavigationRailLabelType.none,
              selectedIndex: selectedIndex,
              onDestinationSelected: (index) =>
                  controller.navigate(pages[index]),
              destinations: destinations,
            ),
          ),
          Padding(
            padding: const EdgeInsets.only(bottom: 8),
            child: collapsed
                ? IconButton(
                    tooltip: controller.engineRunning
                        ? 'Stop engine'
                        : 'Start engine',
                    onPressed: controller.toggleEngine,
                    icon: Icon(
                      controller.engineRunning ? Icons.stop : Icons.play_arrow,
                    ),
                  )
                : FilledButton.icon(
                    onPressed: controller.toggleEngine,
                    icon: Icon(
                      controller.engineRunning ? Icons.stop : Icons.play_arrow,
                    ),
                    label: Text(
                      controller.engineRunning ? 'Stop engine' : 'Start engine',
                    ),
                  ),
          ),
          IconButton(
            tooltip: collapsed ? 'Expand sidebar' : 'Collapse sidebar',
            onPressed: () =>
                controller.update(SettingsPatch(sidebarCollapsed: !collapsed)),
            icon: Icon(collapsed ? Icons.chevron_right : Icons.chevron_left),
          ),
          const SizedBox(height: 8),
        ],
      ),
    );
  }
}

class _Content extends StatelessWidget {
  const _Content({required this.controller});
  final WayvidController controller;

  @override
  Widget build(BuildContext context) {
    const titles = {
      'library': 'Library',
      'folders': 'Folders',
      'monitors': 'Monitors',
      'settings': 'Settings',
      'about': 'About',
    };
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(28, 22, 28, 14),
          child: Row(
            children: [
              Text(
                titles[controller.page] ?? 'Wayvid',
                style: Theme.of(context).textTheme.headlineMedium,
              ),
              const Spacer(),
              if (controller.engineRunning)
                const Chip(
                  avatar: Icon(Icons.circle, size: 10, color: Colors.green),
                  label: Text('Running'),
                ),
              IconButton(
                onPressed: controller.refresh,
                tooltip: 'Refresh',
                icon: const Icon(Icons.refresh),
              ),
            ],
          ),
        ),
        if (controller.anotherInstance)
          const _Notice(
            icon: Icons.info_outline,
            text: 'Wayvid is already running; this window is a secondary instance.',
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
    WallpaperDto? selected;
    for (final wallpaper in controller.wallpapers) {
      if (wallpaper.id == controller.selectedWallpaperId) selected = wallpaper;
    }
    final chosen = selected;
    return Column(
      children: [
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 28),
          child: Row(
            children: [
              Expanded(
                child: TextField(
                  decoration: const InputDecoration(
                    prefixIcon: Icon(Icons.search),
                    hintText: 'Search wallpapers',
                    border: OutlineInputBorder(),
                  ),
                  onChanged: (value) {
                    controller.setSearch(value);
                  },
                ),
              ),
              const SizedBox(width: 12),
              DropdownButton<String>(
                value: controller.filter,
                items: const [
                  DropdownMenuItem(value: 'all', child: Text('All sources')),
                  DropdownMenuItem(value: 'local_file', child: Text('Local')),
                  DropdownMenuItem(
                    value: 'steam_workshop',
                    child: Text('Workshop'),
                  ),
                ],
                onChanged: (value) {
                  if (value != null) {
                    controller.setFilter(value);
                  }
                },
              ),
            ],
          ),
        ),
        const SizedBox(height: 16),
        Expanded(
          child: Row(
            children: [
              Expanded(
                child: controller.visibleWallpapers.isEmpty
                    ? const Center(
                        child: Text(
                          'No wallpapers found. Add a folder to begin.',
                        ),
                      )
                    : GridView.builder(
                        padding: const EdgeInsets.fromLTRB(28, 0, 16, 28),
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
              if (chosen != null && controller.settings.gui.detailPanelVisible)
                SizedBox(
                  width: 300,
                  child: _Details(controller: controller, wallpaper: chosen),
                ),
            ],
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
    color: controller.selectedWallpaperId == wallpaper.id
        ? Theme.of(context).colorScheme.secondaryContainer
        : null,
    child: InkWell(
      onTap: () {
        controller.selectWallpaper(wallpaper.id);
      },
      onDoubleTap: () => controller.apply(wallpaper.id),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Expanded(
            child: FutureBuilder<Uint8List?>(
              future: controller.thumbnail(wallpaper),
              builder: (_, snapshot) => snapshot.data == null
                  ? Container(
                      color: Theme.of(context)
                          .colorScheme
                          .surfaceContainerHighest,
                      child: Center(
                        child: Icon(
                          wallpaper.wallpaperType == 'video'
                              ? Icons.movie_outlined
                              : Icons.image_outlined,
                          size: 44,
                        ),
                      ),
                    )
                  : Image.memory(
                      snapshot.data!,
                      fit: BoxFit.cover,
                      width: double.infinity,
                    ),
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
              '${wallpaper.sourceType} · ${wallpaper.wallpaperType}',
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ),
        ],
      ),
    ),
  );
}

class _Details extends StatelessWidget {
  const _Details({required this.controller, required this.wallpaper});
  final WayvidController controller;
  final WallpaperDto wallpaper;

  @override
  Widget build(BuildContext context) {
    final metadata = wallpaper.metadata;
    return Card(
      margin: const EdgeInsets.only(right: 28, bottom: 28),
      child: Padding(
        padding: const EdgeInsets.all(18),
        child: ListView(
          children: [
            Text(wallpaper.name, style: Theme.of(context).textTheme.titleLarge),
            const SizedBox(height: 12),
            if (metadata.author != null) Text('By ${metadata.author}'),
            if (metadata.description != null)
              Padding(
                padding: const EdgeInsets.only(top: 10),
                child: Text(metadata.description!),
              ),
            const SizedBox(height: 18),
            FilledButton.icon(
              onPressed: () => controller.apply(wallpaper.id),
              icon: const Icon(Icons.wallpaper),
              label: const Text('Apply to all monitors'),
            ),
            const SizedBox(height: 8),
            for (final monitor in controller.monitors)
              OutlinedButton(
                onPressed: () =>
                    controller.apply(wallpaper.id, output: monitor.name),
                child: Text('Apply to ${monitor.name}'),
              ),
            const Divider(height: 28),
            Text('Path', style: Theme.of(context).textTheme.labelMedium),
            SelectableText(
              wallpaper.sourcePath,
              style: Theme.of(context).textTheme.bodySmall,
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
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.all(28),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        FilledButton.icon(
          onPressed: () async {
            final path = await getDirectoryPath(
              confirmButtonText: 'Add folder',
            );
            if (path != null) await controller.scanFolder(path);
          },
          icon: const Icon(Icons.add),
          label: const Text('Add folder'),
        ),
        const SizedBox(height: 18),
        Expanded(
          child: controller.settings.libraryFolders.isEmpty
              ? const Center(child: Text('No library folders configured.'))
              : ListView(
                  children: [
                    for (final folder in controller.settings.libraryFolders)
                      Card(
                        child: ListTile(
                          leading: const Icon(Icons.folder),
                          title: Text(folder),
                          trailing: IconButton(
                            onPressed: () => controller.scanFolder(folder),
                            tooltip: 'Scan folder',
                            icon: const Icon(Icons.refresh),
                          ),
                        ),
                      ),
                  ],
                ),
        ),
      ],
    ),
  );
}

class _MonitorsPage extends StatelessWidget {
  const _MonitorsPage({required this.controller});
  final WayvidController controller;

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.all(28),
    child: controller.monitors.isEmpty
        ? const Center(child: Text('No Wayland outputs detected.'))
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
                          const Chip(label: Text('Active')),
                        IconButton(
                          onPressed: () =>
                              controller.clear(output: monitor.name),
                          tooltip: 'Clear wallpaper',
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

class _SettingsPage extends StatelessWidget {
  const _SettingsPage({required this.controller});
  final WayvidController controller;

  @override
  Widget build(BuildContext context) {
    final gui = controller.settings.gui;
    final playback = controller.settings.playback;
    final power = controller.settings.power;
    return ListView(
      padding: const EdgeInsets.fromLTRB(28, 0, 28, 28),
      children: [
        Text('Appearance', style: Theme.of(context).textTheme.titleLarge),
        DropdownButtonFormField<String>(
          initialValue: gui.theme,
          decoration: const InputDecoration(labelText: 'Theme'),
          items: const [
            DropdownMenuItem(value: 'system', child: Text('System')),
            DropdownMenuItem(value: 'light', child: Text('Light')),
            DropdownMenuItem(value: 'dark', child: Text('Dark')),
          ],
          onChanged: (value) {
            if (value != null) controller.update(SettingsPatch(theme: value));
          },
        ),
        SwitchListTile(
          title: const Text('Show detail panel'),
          value: gui.detailPanelVisible,
          onChanged: (value) =>
              controller.update(SettingsPatch(detailPanelVisible: value)),
        ),
        SwitchListTile(
          title: const Text('Minimize to tray'),
          value: gui.minimizeToTray,
          onChanged: (value) =>
              controller.update(SettingsPatch(minimizeToTray: value)),
        ),
        SwitchListTile(
          title: const Text('Start minimized'),
          value: gui.startMinimized,
          onChanged: (value) =>
              controller.update(SettingsPatch(startMinimized: value)),
        ),
        const Divider(height: 28),
        Text('Playback', style: Theme.of(context).textTheme.titleLarge),
        ListTile(
          title: const Text('Volume'),
          subtitle: Slider(
            value: playback.volume,
            onChanged: (value) =>
                controller.update(SettingsPatch(volume: value)),
          ),
        ),
        SwitchListTile(
          title: const Text('Loop mode'),
          value: playback.loopMode,
          onChanged: (value) => controller.update(
            SettingsPatch(renderer: value ? 'loop' : 'single'),
          ),
        ),
        const Divider(height: 28),
        Text('Power', style: Theme.of(context).textTheme.titleLarge),
        SwitchListTile(
          title: const Text('Pause on battery'),
          value: power.pauseOnBattery,
          onChanged: (value) =>
              controller.update(SettingsPatch(pauseOnBattery: value)),
        ),
        SwitchListTile(
          title: const Text('Pause on fullscreen applications'),
          value: power.pauseOnFullscreen,
          onChanged: (value) =>
              controller.update(SettingsPatch(pauseOnFullscreen: value)),
        ),
        SwitchListTile(
          title: const Text('Launch at login'),
          value: controller.settings.autostartEnabled,
          onChanged: (value) =>
              controller.update(SettingsPatch(autostartEnabled: value)),
        ),
        SwitchListTile(
          title: const Text('Restore last wallpaper'),
          value: controller.settings.restoreLastWallpaper,
          onChanged: (value) =>
              controller.update(SettingsPatch(restoreLastWallpaper: value)),
        ),
      ],
    );
  }
}

class _AboutPage extends StatelessWidget {
  const _AboutPage({required this.controller});
  final WayvidController controller;

  @override
  Widget build(BuildContext context) => Center(
    child: ConstrainedBox(
      constraints: const BoxConstraints(maxWidth: 560),
      child: Card(
        child: Padding(
          padding: const EdgeInsets.all(32),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Icon(Icons.waves, size: 64),
              const SizedBox(height: 14),
              Text('Wayvid', style: Theme.of(context).textTheme.headlineMedium),
              const Text('A Wayland wallpaper engine for Linux'),
              const SizedBox(height: 18),
              const Text(
                'Flutter UI · Rust engine · flutter_rust_bridge service facade',
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 22),
              Wrap(
                spacing: 8,
                children: [
                  OutlinedButton(
                    onPressed: () =>
                        controller.openUrl('https://github.com/wayvid/wayvid'),
                    child: const Text('Project website'),
                  ),
                  OutlinedButton(
                    onPressed: () => controller.openUrl(
                      'https://github.com/wayvid/wayvid/issues',
                    ),
                    child: const Text('Report an issue'),
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
