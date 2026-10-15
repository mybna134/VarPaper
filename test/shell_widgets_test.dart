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
  for (final language in ['en', 'zh']) {
    final chinese = language == 'zh';
    final labels = chinese
        ? ['视频', '场景', '网页', '图片', 'GIF']
        : ['Video', 'Scene', 'Web', 'Image', 'GIF'];
    const types = ['video', 'scene', 'web', 'image', 'gif'];
    const icons = [
      Icons.movie_outlined,
      Icons.auto_awesome_motion_outlined,
      Icons.language,
      Icons.image_outlined,
      Icons.gif_box_outlined,
    ];
    testWidgets('$language Web details require missing component', (
      tester,
    ) async {
      final service = FakeService(
        folders: {
          '/w': [wallpaper('page', category: 'web')],
        },
      );
      final controller = await _controller(_libraryStore(), service);
      try {
        await _pumpShell(tester, controller, locale: Locale(language));
        await tester.tap(find.text('page'));
        await tester.pump(kDoubleTapTimeout);
        await tester.pumpAndSettle();
        expect(
          find.text(
            chinese
                ? '兼容性: 需要安装 Web 壁纸支持'
                : 'Compatibility: Web wallpaper support is required',
          ),
          findsOneWidget,
        );
        await tester.tap(find.text(chinese ? '打开设置' : 'Open settings'));
        await tester.pumpAndSettle();
        expect(controller.page, 'settings');
        expect(service.calls, isNot(contains('installWebSupport')));
        expect(
          service.calls.where((call) => call.startsWith('apply:')),
          isEmpty,
        );
      } finally {
        await tester.pumpWidget(const SizedBox.shrink());
        await controller.shutdown();
      }
    });
    for (var index = 0; index < types.length; index++) {
      final type = types[index];
      testWidgets('$language $type filter, fallback, details and output', (
        tester,
      ) async {
        final item = WallpaperDto(
          id: type,
          name: 'Fixture $type',
          sourcePath: '/fixtures/$type',
          sourceType: 'local_dir',
          // Actual type must win over a stale category.
          wallpaperCategory: 'video',
          wallpaperType: type,
          compatibility: 'ready',
          metadata: wallpaper(type).metadata,
          addedAt: '',
        );
        final service =
            FakeService(
                monitors: [monitor('A', primary: true), monitor('B')],
                folders: {
                  '/w': [item],
                },
              )
              ..webSupport = const WebSupportDto(
                state: 'installed',
                version: 'test',
                diskBytes: 0,
                downloadedBytes: 0,
                source: 'https://cef-builds.spotifycdn.com/',
                supported: true,
              );
        final controller = await _controller(_libraryStore(), service);
        try {
          await _pumpShell(tester, controller, locale: Locale(language));
          expect(find.text('Fixture $type'), findsOneWidget);
          expect(
            find.text(chinese ? '暂无预览' : 'Preview unavailable'),
            findsOneWidget,
          );
          expect(find.byIcon(icons[index]), findsWidgets);
          await tester.tap(find.text(labels[index]));
          await tester.pumpAndSettle();
          expect(find.text('Fixture $type'), findsNothing);
          await tester.tap(find.text(labels[index]));
          await tester.pumpAndSettle();
          await tester.tap(find.text('Fixture $type'));
          await tester.pump(kDoubleTapTimeout);
          await tester.pumpAndSettle();
          expect(
            find.text('${chinese ? '类型' : 'Type'}: ${labels[index]}'),
            findsOneWidget,
          );
          expect(
            find.text(chinese ? '来源: 本地项目' : 'Source: Local project'),
            findsOneWidget,
          );
          expect(
            find.text(chinese ? '兼容性: 可用' : 'Compatibility: Ready'),
            findsOneWidget,
          );
          await tester.tap(find.text(chinese ? '应用到 B' : 'Apply to B'));
          await tester.pumpAndSettle();
          expect(service.calls, contains('apply:B:/fixtures/$type'));
          expect(
            service.calls.where((call) => call.startsWith('apply:')).length,
            1,
          );
        } finally {
          await tester.pumpWidget(const SizedBox.shrink());
          await controller.shutdown();
        }
      });
    }

    for (final error in [
      (
        'Unsupported project type: Application',
        'Unsupported wallpaper type',
        '不支持的壁纸类型',
      ),
      (
        'Missing shared asset: clock.png',
        'Required wallpaper resource is missing',
        '缺少必需的壁纸资源',
      ),
      (
        'Invalid shader: effect.frag',
        'Wallpaper renderer unavailable',
        '壁纸渲染器不可用',
      ),
    ]) {
      testWidgets('$language compatibility error ${error.$1}', (tester) async {
        final service = FakeService(
          folders: {
            '/w': [
              wallpaper(
                'broken',
                name: 'Broken project',
                category: 'unsupported',
                compatibility: 'unsupported',
                compatibilityReason: error.$1,
              ),
            ],
          },
        );
        final controller = await _controller(_libraryStore(), service);
        try {
          await _pumpShell(tester, controller, locale: Locale(language));
          await tester.tap(find.text('Broken project'));
          await tester.pump(kDoubleTapTimeout);
          await tester.pumpAndSettle();
          expect(
            find.text(
              '${chinese ? '兼容性' : 'Compatibility'}: ${chinese ? error.$3 : error.$2}',
            ),
            findsOneWidget,
          );
          expect(find.text(error.$1), findsOneWidget);
          final apply = tester.widget<FilledButton>(
            find.widgetWithText(
              FilledButton,
              chinese ? '应用到所有显示器' : 'Apply to all monitors',
            ),
          );
          expect(apply.onPressed, isNull);
          expect(
            service.calls.where((call) => call.startsWith('apply:')),
            isEmpty,
          );
        } finally {
          await tester.pumpWidget(const SizedBox.shrink());
          await controller.shutdown();
        }
      });
    }
  }

  for (final filePreview in [false, true]) {
    testWidgets(
      'invalid ${filePreview ? 'file' : 'bytes'} preview uses type fallback',
      (tester) async {
        final service =
            FakeService(
                folders: {
                  '/w': [wallpaper('picture', category: 'image')],
                },
              )
              ..previews['picture'] = PreviewDto(
                bytes: tinyPng.sublist(0, 4),
                imagePath: filePreview ? '/missing/varpaper-preview.png' : null,
              );
        final controller = await _controller(_libraryStore(), service);
        try {
          await _pumpShell(tester, controller);
          await tester.runAsync(() async {
            await Future<void>.delayed(const Duration(milliseconds: 100));
          });
          await tester.pumpAndSettle();
          expect(find.text('Preview unavailable'), findsOneWidget);
          expect(find.byIcon(Icons.image_outlined), findsWidgets);
          expect(tester.takeException(), isNull);
        } finally {
          await tester.pumpWidget(const SizedBox.shrink());
          await controller.shutdown();
        }
      },
    );
  }

  testWidgets(
    'Web support setting shows truthful installed state and uninstall action',
    (tester) async {
      final service = FakeService();
      final controller = await _controller(FakeStore(), service);
      controller.navigate('settings');
      await _pumpShell(tester, controller);
      await tester.scrollUntilVisible(find.text('Web wallpaper support'), 400);
      await tester.pumpAndSettle();
      expect(find.text('Not installed'), findsOneWidget);
      await tester.tap(find.text('Install'));
      await tester.pumpAndSettle();
      expect(service.calls, contains('installWebSupport'));
      expect(find.text('Not installed'), findsOneWidget);
      service.webSupport = const WebSupportDto(
        state: 'installed',
        version: 'test',
        diskBytes: 1048576,
        downloadedBytes: 0,
        source: 'https://cef-builds.spotifycdn.com/',
        supported: true,
      );
      await controller.refreshWebSupport();
      await tester.pumpAndSettle();
      expect(find.text('Installed'), findsOneWidget);
      expect(find.text('Disk usage: 1.0 MB'), findsOneWidget);
      await tester.tap(find.text('Uninstall'));
      await tester.pumpAndSettle();
      expect(service.calls, contains('uninstallWebSupport'));
      expect(find.text('Not installed'), findsOneWidget);
      controller.dispose();
    },
  );

  testWidgets('Web support setting is localized in Chinese', (tester) async {
    final controller = await _controller(FakeStore(), FakeService());
    controller.navigate('settings');
    await _pumpShell(tester, controller, locale: const Locale('zh'));
    await tester.scrollUntilVisible(find.text('Web 壁纸支持'), 400);
    await tester.pumpAndSettle();
    expect(find.text('未安装'), findsOneWidget);
    expect(find.text('安装'), findsOneWidget);
    controller.dispose();
  });

  testWidgets(
    'Web component progress, cancellation and retry use service state',
    (tester) async {
      final service = FakeService();
      service.webSupport = const WebSupportDto(
        state: 'downloading',
        version: 'test',
        diskBytes: 0,
        downloadedBytes: 1048576,
        totalBytes: 2097152,
        source: 'https://cef-builds.spotifycdn.com/',
        supported: true,
      );
      final controller = await _controller(FakeStore(), service);
      controller.navigate('settings');
      await _pumpShell(tester, controller);
      await tester.scrollUntilVisible(find.text('Web wallpaper support'), 400);
      await tester.pump();
      expect(find.text('Downloading'), findsOneWidget);
      expect(
        tester
            .widget<LinearProgressIndicator>(
              find.byType(LinearProgressIndicator),
            )
            .value,
        0.5,
      );
      expect(find.text('1.0 MB / 2.0 MB'), findsOneWidget);
      await tester.tap(find.text('Cancel'));
      await tester.pump();
      expect(service.calls, contains('cancelWebSupportInstall'));
      service.webSupport = const WebSupportDto(
        state: 'error',
        version: 'test',
        diskBytes: 0,
        downloadedBytes: 0,
        source: 'https://cef-builds.spotifycdn.com/',
        supported: true,
        error: 'Download interrupted',
      );
      await controller.refreshWebSupport();
      await tester.pumpAndSettle();
      expect(find.text('Download interrupted'), findsOneWidget);
      await tester.tap(find.text('Retry installation'));
      await tester.pumpAndSettle();
      expect(service.calls, contains('installWebSupport'));
      controller.dispose();
    },
  );

  testWidgets(
    'uninstalled Web stays visible and details link to settings without applying',
    (tester) async {
      final service = FakeService(
        monitors: [monitor('A', primary: true)],
        folders: {
          '/w': [wallpaper('web', name: 'Clock page', category: 'web')],
        },
      );
      final controller = await _controller(_libraryStore(), service);
      await _pumpShell(tester, controller);
      expect(find.text('Clock page'), findsOneWidget);
      await tester.tap(find.text('Clock page'));
      await tester.pump(kDoubleTapTimeout);
      await tester.pumpAndSettle();
      final button = tester.widget<FilledButton>(
        find.widgetWithText(FilledButton, 'Apply to all monitors'),
      );
      expect(button.onPressed, isNull);
      expect(
        find.text(
          'Install Web wallpaper support in Settings before applying this wallpaper.',
        ),
        findsOneWidget,
      );
      await tester.tap(find.text('Open settings'));
      await tester.pumpAndSettle();
      expect(controller.page, 'settings');
      expect(service.calls, isNot(contains('installWebSupport')));
      expect(controller.appliedByOutput, isEmpty);
      controller.dispose();
    },
  );

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
      expect(controller.wallpaperCategories, {
        'video',
        'web',
        'image',
        'gif',
        'unsupported',
      });
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
      // The volume row may have been disposed after scrolling to restore.
      await tester.scrollUntilVisible(slider, -300);
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
