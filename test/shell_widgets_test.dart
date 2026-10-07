import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:varpaper/main.dart';
import 'package:varpaper/src/bridge_generated.dart/bridge.dart';
import 'package:varpaper/src/l10n.dart';
import 'package:varpaper/src/storage/settings_store.dart';

import 'support/fakes.dart';

Future<WayvidController> _controller(
  FakeStore store,
  FakeService service,
) async {
  final controller = await WayvidController.create(
    store,
    serviceOverride: service,
  );
  controller.eventTimer?.cancel();
  return controller;
}

Future<void> _pumpShell(
  WidgetTester tester,
  WayvidController controller, {
  Locale locale = const Locale('en'),
}) async {
  tester.view.physicalSize = const Size(1200, 900);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.resetPhysicalSize);
  addTearDown(tester.view.resetDevicePixelRatio);
  await tester.pumpWidget(
    AnimatedBuilder(
      animation: controller,
      builder: (context, _) => MaterialApp(
        locale: locale,
        supportedLocales: const [Locale('en'), Locale('zh')],
        localizationsDelegates: const [
          WayvidLocalizations.delegate,
          GlobalMaterialLocalizations.delegate,
          GlobalWidgetsLocalizations.delegate,
          GlobalCupertinoLocalizations.delegate,
        ],
        home: WayvidShell(controller: controller),
      ),
    ),
  );
  await tester.pumpAndSettle();
}

FakeService _libraryService() => FakeService(
  monitors: [
    monitor('A', primary: true, current: '/wallpapers/a.mp4'),
    monitor('B'),
  ],
  folders: {
    '/w': [
      wallpaper(
        'a',
        name: 'Aurora',
        author: 'Alice',
        description: 'Northern lights',
      ),
      wallpaper('b', name: 'Beach', sourceType: 'workshop', category: 'scene'),
    ],
  },
)..previews['a'] = PreviewDto(bytes: tinyPng);

FakeStore _libraryStore() => FakeStore(
  settings: SettingsDto.defaults().apply(
    const SettingsPatch(restoreLastWallpaper: false, libraryFolders: ['/w']),
  ),
);

void main() {
  testWidgets('library search, filters and wallpaper details', (tester) async {
    final store = _libraryStore();
    final service = _libraryService();
    final controller = await _controller(store, service);
    try {
      await _pumpShell(tester, controller);
      expect(find.text('Aurora'), findsOneWidget);
      expect(find.text('Beach'), findsOneWidget);

      // Search expands into a text field and filters the grid.
      await tester.tap(find.byTooltip('Search wallpapers'));
      await tester.pumpAndSettle();
      await tester.enterText(find.byType(TextField), 'aur');
      await tester.pumpAndSettle();
      expect(controller.search, 'aur');
      expect(find.text('Beach'), findsNothing);
      await tester.tap(find.byTooltip('Close search'));
      await tester.pumpAndSettle();
      expect(find.byType(TextField), findsNothing);
      controller.setSearch('');
      await tester.pumpAndSettle();

      // The category filter hides scenes.
      await tester.tap(find.text('Scene'));
      await tester.pumpAndSettle();
      expect(controller.wallpaperCategories, {'video'});
      expect(find.text('Beach'), findsNothing);
      await tester.tap(find.text('Scene'));
      await tester.pumpAndSettle();

      // The "more" popup toggles source visibility.
      await tester.tap(find.byTooltip('More'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Workshop wallpapers'));
      await tester.pumpAndSettle();
      expect(controller.showWorkshopWallpapers, isFalse);
      expect(find.text('Beach'), findsNothing);
      await tester.tap(find.text('Local wallpapers'));
      await tester.pumpAndSettle();
      expect(controller.showLocalWallpapers, isFalse);
      expect(
        find.text('No wallpapers found. Add a folder to begin.'),
        findsOneWidget,
      );
      await tester.tapAt(const Offset(600, 850));
      await tester.pumpAndSettle();
      controller.setSourceVisibility(local: true, workshop: true);
      await tester.pumpAndSettle();

      // Tapping a card opens the detail panel.
      await tester.tap(find.text('Aurora'));
      await tester.pump(kDoubleTapTimeout);
      await tester.pumpAndSettle();
      expect(controller.selectedWallpaperId, 'a');
      expect(find.text('By: Alice'), findsOneWidget);
      expect(find.text('Northern lights'), findsOneWidget);

      await tester.tap(find.text('Apply to A'));
      await tester.pumpAndSettle();
      expect(controller.isAppliedTo('a', 'A'), isTrue);
      expect(find.text('Unapply from A'), findsOneWidget);
      await tester.tap(find.text('Unapply from A'));
      await tester.pumpAndSettle();
      expect(controller.appliedByOutput, isEmpty);

      await tester.tap(find.text('Apply to all monitors'));
      await tester.pumpAndSettle();
      expect(controller.isAppliedEverywhere('a'), isTrue);
      await tester.tap(find.text('Unapply from all monitors'));
      await tester.pumpAndSettle();
      expect(controller.appliedByOutput, isEmpty);

      await tester.tap(find.byTooltip('Close details'));
      await tester.pumpAndSettle();
      expect(find.text('By: Alice'), findsNothing);

      service.calls.clear();
      await tester.tap(find.byTooltip('Rescan wallpapers'));
      await tester.pumpAndSettle();
      expect(service.calls, contains('scan:/w'));
    } finally {
      await tester.pumpWidget(const SizedBox.shrink());
      await controller.shutdown();
    }
  });

  testWidgets('sidebar navigates every page and toggles labels', (
    tester,
  ) async {
    final store = _libraryStore();
    final service = _libraryService();
    final controller = await _controller(store, service);
    try {
      await _pumpShell(tester, controller);

      await tester.tap(find.byIcon(Icons.folder_outlined));
      await tester.pumpAndSettle();
      expect(controller.page, 'folders');
      expect(find.text('/w'), findsOneWidget);
      service.calls.clear();
      await tester.tap(find.byTooltip('Scan folder'));
      await tester.pumpAndSettle();
      expect(service.calls, ['scan:/w']);
      expect(find.text('Add folder'), findsOneWidget);
      service.calls.clear();

      await tester.tap(find.byIcon(Icons.desktop_windows_outlined));
      await tester.pumpAndSettle();
      expect(controller.page, 'monitors');
      expect(find.text('Active'), findsOneWidget);
      expect(find.byIcon(Icons.star), findsOneWidget);
      await tester.tap(find.byTooltip('Clear wallpaper').last);
      await tester.pumpAndSettle();
      expect(service.calls.last, 'clear:B');

      await tester.tap(find.byIcon(Icons.info_outline));
      await tester.pumpAndSettle();
      expect(controller.page, 'about');
      await tester.tap(find.text('Project website'));
      await tester.tap(find.text('Report an issue'));
      await tester.pumpAndSettle();
      expect(service.calls, [
        'clear:B',
        'open:https://github.com/mybna134/var_paper',
        'open:https://github.com/mybna134/var_paper/issues',
      ]);

      expect(find.text('About'), findsWidgets);
      await tester.tap(find.byTooltip('Hide labels'));
      await tester.pumpAndSettle();
      expect(store.settings.gui.sidebarCollapsed, isTrue);
      expect(find.byTooltip('Show labels'), findsOneWidget);

      controller.error = 'Something failed';
      controller.navigate('library');
      await tester.pumpAndSettle();
      expect(find.text('Something failed'), findsOneWidget);
    } finally {
      await tester.pumpWidget(const SizedBox.shrink());
      await controller.shutdown();
    }
  });

  testWidgets('empty folders and monitors pages show placeholders', (
    tester,
  ) async {
    final controller = await _controller(
      FakeStore(),
      FakeService(monitors: []),
    );
    try {
      await _pumpShell(tester, controller);
      controller.navigate('folders');
      await tester.pumpAndSettle();
      expect(find.text('No library folders configured.'), findsOneWidget);
      controller.navigate('monitors');
      await tester.pumpAndSettle();
      expect(find.text('No displays detected.'), findsOneWidget);
    } finally {
      await tester.pumpWidget(const SizedBox.shrink());
      await controller.shutdown();
    }
  });

  testWidgets('settings page switches, slider and dropdowns persist', (
    tester,
  ) async {
    final store = FakeStore();
    final service = FakeService();
    final controller = await _controller(store, service);
    controller.navigate('settings');
    try {
      await _pumpShell(tester, controller);

      await tester.ensureVisible(find.text('System'));
      await tester.tap(find.text('System'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Dark').last);
      await tester.pumpAndSettle();
      expect(store.settings.gui.theme, 'dark');

      await tester.tap(find.text('English'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('中文').last);
      await tester.pumpAndSettle();
      expect(store.settings.gui.language, 'zh');

      Future<void> toggle(String title) async {
        final row = find.ancestor(
          of: find.text(title),
          matching: find.byType(Row),
        );
        final toggle = find.descendant(
          of: row.first,
          matching: find.byType(Switch),
        );
        await tester.ensureVisible(toggle);
        await tester.tap(toggle);
        await tester.pumpAndSettle();
      }

      await toggle('Minimize to tray');
      expect(store.settings.gui.minimizeToTray, isFalse);
      await toggle('Start minimized');
      expect(store.settings.gui.startMinimized, isTrue);
      await toggle('Loop mode');
      expect(store.settings.playback.loopMode, isFalse);
      await toggle('Pause on battery');
      expect(store.settings.power.pauseOnBattery, isTrue);
      await toggle('Pause on fullscreen applications');
      expect(store.settings.power.pauseOnFullscreen, isFalse);
      await toggle('Launch at login');
      expect(store.autostart, [true]);
      // The fake store starts with restore disabled.
      await toggle('Restore last wallpaper');
      expect(store.settings.restoreLastWallpaper, isTrue);
      // Loop mode and pause-on-battery are engine settings.
      expect(service.configs.length, 2);

      final slider = find.byType(Slider);
      await tester.ensureVisible(slider);
      await tester.tap(slider);
      await tester.pumpAndSettle();
      expect(store.settings.playback.volume, closeTo(0.5, 0.01));
    } finally {
      await tester.pumpWidget(const SizedBox.shrink());
      await controller.shutdown();
    }
  });

  testWidgets('Chinese locale translates navigation', (tester) async {
    final controller = await _controller(FakeStore(), FakeService());
    try {
      await _pumpShell(tester, controller, locale: const Locale('zh'));
      expect(find.text('壁纸库'), findsWidgets);
      expect(find.text('设置'), findsOneWidget);
    } finally {
      await tester.pumpWidget(const SizedBox.shrink());
      await controller.shutdown();
    }
  });

  group('WayvidApp', () {
    const trayChannel = MethodChannel('tray_manager');
    const windowChannel = MethodChannel('window_manager');
    late List<String> windowCalls;
    late List<Map<Object?, Object?>> menus;
    late bool windowVisible;

    setUp(() {
      windowCalls = [];
      menus = [];
      windowVisible = true;
      final messenger =
          TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
      messenger.setMockMethodCallHandler(trayChannel, (call) async {
        if (call.method == 'setContextMenu') {
          final args = call.arguments as Map<Object?, Object?>;
          menus.add(args['menu'] as Map<Object?, Object?>);
        }
        return null;
      });
      messenger.setMockMethodCallHandler(windowChannel, (call) async {
        windowCalls.add(call.method);
        if (call.method == 'setSkipTaskbar') {
          expect(call.arguments, {'isSkipTaskbar': false});
        }
        if (call.method == 'isVisible') return windowVisible;
        if (call.method.startsWith('is')) return false;
        return null;
      });
    });

    tearDown(() {
      final messenger =
          TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
      messenger.setMockMethodCallHandler(trayChannel, null);
      messenger.setMockMethodCallHandler(windowChannel, null);
    });

    Future<void> send(String channel, String method, Object? args) async {
      final messenger =
          TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
      await messenger.handlePlatformMessage(
        channel,
        const StandardMethodCodec().encodeMethodCall(MethodCall(method, args)),
        (_) {},
      );
    }

    List<Map<Object?, Object?>> items() =>
        (menus.last['items'] as List).cast<Map<Object?, Object?>>();

    Future<void> click(WidgetTester tester, String key) async {
      final id = items().firstWhere((item) => item['key'] == key)['id'];
      await send('tray_manager', 'onTrayMenuItemClick', {'id': id});
      await tester.pumpAndSettle();
    }

    testWidgets('builds themed app and drives the tray menu', (tester) async {
      final store = FakeStore(
        settings: SettingsDto.defaults().apply(
          const SettingsPatch(
            restoreLastWallpaper: false,
            libraryFolders: ['/w'],
            theme: 'dark',
          ),
        ),
      );
      final service = FakeService(
        folders: {
          '/w': [wallpaper('a', name: 'A'), wallpaper('b', name: 'B')],
        },
      );
      final controller = await _controller(store, service);
      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            wayvidControllerProvider.overrideWith((ref) => controller),
          ],
          child: const WayvidApp(),
        ),
      );
      await tester.pumpAndSettle();

      final app = tester.widget<MaterialApp>(find.byType(MaterialApp));
      expect(app.themeMode, ThemeMode.dark);
      expect(menus, isNotEmpty);
      final labels = items().map((item) => item['label']).toList();
      expect(labels, containsAll(['Show VarPaper', 'Pause', 'Unmute']));

      windowCalls.clear();
      await click(tester, 'settings');
      expect(controller.page, 'settings');
      expect(windowCalls, ['setSkipTaskbar', 'isMinimized', 'show', 'focus']);
      windowCalls.clear();
      await click(tester, 'library');
      expect(controller.page, 'library');
      expect(windowCalls, ['setSkipTaskbar', 'isMinimized', 'show', 'focus']);
      windowCalls.clear();
      await click(tester, 'show');
      expect(windowCalls, ['setSkipTaskbar', 'isMinimized', 'show', 'focus']);

      await click(tester, 'next');
      expect(controller.appliedByOutput, {'A': 'a'});
      await click(tester, 'previous');
      expect(controller.appliedByOutput, {'A': 'b'});

      await click(tester, 'pause');
      expect(controller.manuallyPaused, isTrue);
      expect(
        items().firstWhere((item) => item['key'] == 'pause')['label'],
        'Resume',
      );
      await click(tester, 'mute');
      expect(store.settings.playback.mute, isFalse);
      expect(
        items().firstWhere((item) => item['key'] == 'mute')['label'],
        'Mute',
      );

      // Language changes rebuild the tray menu in Chinese.
      await controller.update(const SettingsPatch(language: 'zh'));
      await tester.pumpAndSettle();
      expect(items().first['label'], '显示 VarPaper');

      // Tray icon click toggles window visibility.
      windowCalls.clear();
      await send('tray_manager', 'onTrayIconMouseDown', null);
      await tester.pumpAndSettle();
      expect(windowCalls, ['isVisible', 'hide']);
      windowVisible = false;
      windowCalls.clear();
      await send('tray_manager', 'onTrayIconMouseDown', null);
      await tester.pumpAndSettle();
      expect(windowCalls, [
        'isVisible',
        'setSkipTaskbar',
        'isMinimized',
        'show',
        'focus',
      ]);

      // Closing hides to tray while minimizeToTray is enabled.
      windowCalls.clear();
      await send('window_manager', 'onEvent', {'eventName': 'close'});
      await tester.pumpAndSettle();
      expect(windowCalls, ['hide']);

      await controller.update(const SettingsPatch(minimizeToTray: false));
      await tester.pumpAndSettle();
      windowCalls.clear();
      await send('window_manager', 'onEvent', {'eventName': 'close'});
      await tester.pumpAndSettle();
      expect(windowCalls, ['destroy']);
      expect(service.calls, contains('shutdown'));

      service.calls.clear();
      await click(tester, 'quit');
      expect(service.calls, ['shutdown']);

      await tester.pumpWidget(const SizedBox.shrink());
      expect(service.calls, ['shutdown', 'shutdown']);
    });
  });
}
