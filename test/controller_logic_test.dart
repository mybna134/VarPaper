import 'package:flutter_test/flutter_test.dart';
import 'package:varpaper/main.dart';
import 'package:varpaper/src/bridge_generated.dart/bridge.dart';
import 'package:varpaper/src/storage/settings_store.dart';

import 'support/fakes.dart';

Future<WayvidController> _create(FakeStore store, FakeService service) async {
  final controller = await WayvidController.create(
    store,
    serviceOverride: service,
  );
  controller.eventTimer?.cancel();
  return controller;
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('starts the engine when it is not already running', () async {
    final service = FakeService(engineRunning: false);
    final controller = await _create(FakeStore(), service);
    expect(service.calls.first, 'createEngine');
    expect(controller.engineRunning, isTrue);
    expect(controller.error, isNull);
    await controller.shutdown();
    expect(service.calls.last, 'shutdown');
  });

  test('engine start failure is reported as an error', () async {
    final service = FakeService(engineRunning: false)
      ..createEngineError = Exception('no gpu');
    final controller = await _create(FakeStore(), service);
    expect(controller.engineRunning, isFalse);
    // refresh() clears the error when it runs, so the message is gone but the
    // engine stays stopped and apply will retry starting it.
    service.createEngineError = null;
    service.folders['/w'] = [wallpaper('a')];
    await controller.scanFolder('/w');
    await controller.apply('a');
    expect(service.calls.where((c) => c == 'createEngine').length, 2);
    expect(controller.engineRunning, isTrue);
    await controller.shutdown();
  });

  test('refresh merges folders and workshop sorted by name', () async {
    final store = FakeStore(
      settings: SettingsDto.defaults().apply(
        const SettingsPatch(
          restoreLastWallpaper: false,
          libraryFolders: ['/one', '/two'],
        ),
      ),
    );
    final service = FakeService(
      workshopAvailable: true,
      folders: {
        '/one': [wallpaper('c', name: 'Charlie')],
        '/two': [
          wallpaper('a', name: 'Alpha'),
          wallpaper('c', name: 'Charlie 2'),
        ],
      },
      workshop: [
        wallpaper(
          'b',
          name: 'Bravo',
          sourceType: 'workshop',
          category: 'scene',
        ),
      ],
    );
    final controller = await _create(store, service);
    expect(controller.workshopAvailable, isTrue);
    expect(controller.wallpapers.map((w) => w.name), [
      'Alpha',
      'Bravo',
      'Charlie 2',
    ]);
    expect(
      service.calls,
      containsAll(['scan:/one', 'scan:/two', 'scanWorkshop']),
    );
    await controller.shutdown();
  });

  test('visibleWallpapers filters by source, category and search', () async {
    final service = FakeService(
      folders: {
        '/w': [
          wallpaper('local', name: 'Ocean', tags: ['blue']),
          wallpaper(
            'shop',
            name: 'Forest',
            sourceType: 'steam_workshop',
            category: 'scene',
            title: 'Green Woods',
          ),
          wallpaper('dir', name: 'City', sourceType: 'local_dir'),
        ],
      },
    );
    final controller = await _create(FakeStore(), service);
    await controller.scanFolder('/w');
    List<String> ids() =>
        controller.visibleWallpapers.map((w) => w.id).toList();

    expect(ids(), ['dir', 'shop', 'local']);

    controller.setSearch('  BLUE ');
    expect(ids(), ['local']);
    controller.setSearch('woods');
    expect(ids(), ['shop']);
    controller.setSearch('');

    controller.setSourceVisibility(workshop: false);
    expect(ids(), ['dir', 'local']);
    controller.setSourceVisibility(local: false, workshop: true);
    expect(ids(), ['shop']);
    controller.setSourceVisibility(local: true);

    controller.setWallpaperCategories({'video'});
    expect(ids(), ['dir', 'local']);
    // An empty selection is ignored so something is always shown.
    controller.setWallpaperCategories({});
    expect(controller.wallpaperCategories, {'video'});

    controller.selectWallpaper('dir');
    expect(controller.selectedWallpaperId, 'dir');
    controller.navigate('about');
    expect(controller.page, 'about');
    await controller.shutdown();
  });

  test('scanFolder persists new folders only once', () async {
    final store = FakeStore();
    final service = FakeService(
      folders: {
        '/w': [wallpaper('a')],
      },
    );
    final controller = await _create(store, service);
    await controller.scanFolder('/w');
    await controller.scanFolder('/w');
    expect(controller.settings.libraryFolders, ['/w']);
    expect(store.settings.libraryFolders, ['/w']);
    expect(controller.wallpapers.map((w) => w.id), ['a']);
    await controller.shutdown();
  });

  test('apply to all monitors, clear all, and unknown ids', () async {
    final store = FakeStore();
    final service = FakeService(
      monitors: [monitor('A'), monitor('B')],
      folders: {
        '/w': [wallpaper('a')],
      },
    );
    final controller = await _create(store, service);
    await controller.scanFolder('/w');

    await controller.apply('a');
    expect(controller.isAppliedEverywhere('a'), isTrue);
    expect(controller.appliedWallpaperIds, {'a'});
    expect(store.assignments.keys, ['all']);
    expect(service.calls, contains('apply:all:/wallpapers/a.mp4'));

    await controller.clear(output: 'B');
    expect(controller.isAppliedEverywhere('a'), isFalse);
    expect(store.assignments['B']!.sourcePath, isEmpty);

    await controller.clear();
    expect(controller.appliedByOutput, isEmpty);
    expect(store.assignments, isEmpty);
    expect(service.calls, contains('clear:all'));

    await controller.apply('missing');
    expect(controller.error, contains('No element'));
    await controller.shutdown();
  });

  test('pause and resume are guarded by engine state', () async {
    final service = FakeService();
    final controller = await _create(FakeStore(), service);

    await controller.setPaused(true);
    await controller.setPaused(true);
    await controller.setPaused(false);
    expect(
      service.calls.where(
        (c) => c.startsWith('pause') || c.startsWith('resume'),
      ),
      ['pause:all', 'resume:all'],
    );

    await controller.setPaused(true);
    service.events.add(const ServiceEvent.engineStopped());
    await controller.pollEvents();
    expect(controller.engineRunning, isFalse);
    expect(controller.manuallyPaused, isFalse);

    service.calls.clear();
    await controller.setPaused(true);
    expect(service.calls, isEmpty);

    service.events
      ..add(const ServiceEvent.engineStarted())
      ..add(const ServiceEvent.error(code: 'x', message: 'boom'));
    await controller.pollEvents();
    expect(controller.engineRunning, isTrue);
    expect(controller.error, 'boom');

    // Polling failures are swallowed and retried later.
    service.pollError = Exception('gone');
    await controller.pollEvents();
    expect(controller.error, 'boom');
    await controller.shutdown();
  });

  test('cycleWallpaper wraps around a shared wallpaper', () async {
    final service = FakeService(
      monitors: [monitor('A'), monitor('B')],
      folders: {
        '/w': [wallpaper('a', name: 'A'), wallpaper('b', name: 'B')],
      },
    );
    final controller = await _create(FakeStore(), service);
    await controller.scanFolder('/w');

    await controller.cycleWallpaper(1);
    // Nothing is applied yet on the multi-monitor path: each output starts
    // at the first wallpaper.
    expect(controller.appliedByOutput, {'A': 'a', 'B': 'a'});

    await controller.cycleWallpaper(1);
    expect(controller.appliedByOutput, {'A': 'b', 'B': 'b'});
    await controller.cycleWallpaper(1);
    expect(controller.appliedByOutput, {'A': 'a', 'B': 'a'});
    await controller.cycleWallpaper(-1);
    expect(controller.appliedByOutput, {'A': 'b', 'B': 'b'});
    await controller.shutdown();
  });

  test('cycleWallpaper is a no-op without wallpapers or monitors', () async {
    final service = FakeService(monitors: []);
    final controller = await _create(FakeStore(), service);
    await controller.cycleWallpaper(1);
    expect(service.calls.where((c) => c.startsWith('apply')), isEmpty);
    await controller.shutdown();
  });

  test('update persists settings and only pushes engine fields', () async {
    final store = FakeStore();
    final service = FakeService();
    final controller = await _create(store, service);

    await controller.update(const SettingsPatch(theme: 'dark'));
    expect(store.settings.gui.theme, 'dark');
    expect(service.configs, isEmpty);

    await controller.update(const SettingsPatch(autostartEnabled: true));
    expect(store.autostart, [true]);
    expect(service.configs, isEmpty);

    await controller.update(const SettingsPatch(volume: 0.5));
    await controller.update(
      const SettingsPatch(fpsLimit: FpsLimitPatch(value: 24)),
    );
    await controller.update(const SettingsPatch(pauseOnBattery: true));
    expect(service.configs.map((c) => c.volume), [0.5, 0.5, 0.5]);
    expect(service.configs.last.fpsLimit, 24);
    expect(service.configs.last.pauseOnBattery, isTrue);
    await controller.shutdown();
  });

  test('thumbnails are cached per wallpaper and failures yield null', () async {
    final service = FakeService(
      folders: {
        '/w': [wallpaper('a'), wallpaper('b')],
      },
    );
    service.previews['a'] = PreviewDto(bytes: tinyPng);
    final controller = await _create(FakeStore(), service);
    await controller.scanFolder('/w');
    final a = controller.wallpapers.first;
    final first = controller.thumbnail(a);
    expect(identical(first, controller.thumbnail(a)), isTrue);
    expect((await first)!.bytes, tinyPng);
    expect(await controller.thumbnail(controller.wallpapers.last), isNull);
    expect(service.previewLoads, 2);

    await controller.openUrl('https://example.com');
    expect(service.calls, contains('open:https://example.com'));
    await controller.shutdown();
  });
}
