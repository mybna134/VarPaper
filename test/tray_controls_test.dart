import 'package:flutter_test/flutter_test.dart';
import 'package:varpaper/main.dart';
import 'package:varpaper/src/bridge_generated.dart/bridge.dart';
import 'package:varpaper/src/storage/settings_store.dart';

import 'support/fakes.dart' as fixtures;

class _Store implements SettingsStore {
  _Store()
    : settings = SettingsDto.defaults().apply(
        const SettingsPatch(
          libraryFolders: ['/wallpapers'],
          restoreLastWallpaper: false,
        ),
      );

  SettingsDto settings;

  @override
  Future<SettingsDto> load() async => settings;

  @override
  Future<void> save(SettingsDto value) async => settings = value;

  @override
  Future<void> saveAssignment({
    required String output,
    required WallpaperSourceDto? source,
  }) async {}

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Service implements WayvidService {
  @override
  Future<WebSupportDto> webSupportStatus() async => const WebSupportDto(
    state: 'not_installed',
    version: 'test',
    diskBytes: 0,
    downloadedBytes: 0,
    source: 'https://cef-builds.spotifycdn.com/',
    supported: true,
  );

  final applied = <String>[];
  final paused = <String>[];
  final configs = <EngineConfigDto>[];

  @override
  Future<ServiceInfo> initialize() async =>
      const ServiceInfo(workshopAvailable: false, engineRunning: true);

  @override
  Future<List<MonitorDto>> refreshMonitors() async => const [
    MonitorDto(
      name: 'A',
      width: 1920,
      height: 1080,
      x: 0,
      y: 0,
      scale: 1,
      primary: true,
    ),
    MonitorDto(
      name: 'B',
      width: 1920,
      height: 1080,
      x: 1920,
      y: 0,
      scale: 1,
      primary: false,
    ),
  ];

  @override
  Future<List<WallpaperDto>> scanFolder({required String path}) async => const [
    WallpaperDto(
      id: 'alpha',
      compatibility: 'ready',
      name: 'Alpha',
      sourcePath: '/wallpapers/a.mp4',
      sourceType: 'local_file',
      wallpaperCategory: 'video',
      wallpaperType: 'video',
      metadata: WallpaperMetadataDto(tags: []),
      addedAt: '',
    ),
    WallpaperDto(
      id: 'beta',
      compatibility: 'ready',
      name: 'Beta',
      sourcePath: '/wallpapers/b.mp4',
      sourceType: 'local_file',
      wallpaperCategory: 'video',
      wallpaperType: 'video',
      metadata: WallpaperMetadataDto(tags: []),
      addedAt: '',
    ),
  ];

  @override
  Future<void> applyWallpaper({required String path, String? output}) async {
    applied.add('${output ?? 'all'}:$path');
  }

  @override
  Future<void> applyWallpaperSource({
    required WallpaperSourceDto source,
    String? output,
  }) => applyWallpaper(path: source.sourcePath, output: output);

  @override
  Future<void> pause({String? output}) async => paused.add(output ?? 'all');

  @override
  Future<void> resume({String? output}) async {}

  @override
  Future<void> updateEngineConfig({required EngineConfigDto config}) async {
    configs.add(config);
  }

  @override
  Future<void> shutdown() async {}

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test(
    'tray controls cycle each display and preserve pause and mute state',
    () async {
      final store = _Store();
      final service = _Service();
      final controller = await WayvidController.create(
        store,
        serviceOverride: service,
      );
      try {
        await controller.apply('alpha', output: 'A');
        await controller.apply('beta', output: 'B');
        await controller.cycleWallpaper(1);
        expect(controller.appliedByOutput, {'A': 'beta', 'B': 'alpha'});

        await controller.setPaused(true);
        await controller.cycleWallpaper(-1);
        expect(controller.appliedByOutput, {'A': 'alpha', 'B': 'beta'});
        expect(service.paused, ['all', 'A', 'B']);
        expect(controller.manuallyPaused, isTrue);

        await controller.update(const SettingsPatch(mute: false));
        expect(store.settings.playback.mute, isFalse);
        expect(service.configs.last.mute, isFalse);
      } finally {
        await controller.shutdown();
      }
    },
  );
  test(
    'tray cycles mixed types, skips unavailable projects and routes controls',
    () async {
      final store = fixtures.FakeStore();
      final types = ['video', 'scene', 'web', 'image', 'gif'];
      final service = fixtures.FakeService(
        monitors: [fixtures.monitor('A'), fixtures.monitor('B')],
      );
      final controller = await WayvidController.create(
        store,
        serviceOverride: service,
      );
      controller.eventTimer?.cancel();
      try {
        service.webSupport = const WebSupportDto(
          state: 'installed',
          version: 'test',
          diskBytes: 1,
          downloadedBytes: 0,
          source: 'fixture',
          supported: true,
        );
        await controller.refreshWebSupport();
        controller.wallpapers = [
          for (var i = 0; i < types.length; i++)
            fixtures.wallpaper('type-$i', name: '$i', category: types[i]),
          fixtures.wallpaper(
            'unsupported',
            name: '0a',
            category: 'unsupported',
            compatibility: 'unsupported',
          ),
          fixtures.wallpaper(
            'broken-scene',
            name: '1a',
            category: 'scene',
            compatibility: 'invalid',
          ),
        ];
        await controller.setPaused(true);
        for (var i = 0; i < types.length; i++) {
          await controller.apply('type-$i', output: 'A');
          expect(controller.error, isNull);
          expect(
            store.assignments['A']!.sourceDescription!.wallpaperType,
            types[i],
          );
          expect(service.calls.last, 'pause:A');
          await controller.update(
            SettingsPatch(volume: 0.4, mute: false, pauseOnBattery: true),
          );
          expect(service.configs.last.volume, 0.4);
          expect(service.configs.last.mute, isFalse);
          expect(service.configs.last.pauseOnBattery, isTrue);
          expect(controller.manuallyPaused, isTrue);
        }
        await controller.setPaused(false);
        expect(service.calls.last, 'resume:all');
        await controller.update(const SettingsPatch(pauseOnBattery: false));
        expect(service.configs.last.pauseOnBattery, isFalse);
        await controller.apply('type-0');
        await controller.cycleWallpaper(1);
        expect(controller.appliedByOutput, {'A': 'type-1', 'B': 'type-1'});
        await controller.cycleWallpaper(1);
        expect(controller.appliedByOutput, {'A': 'type-2', 'B': 'type-2'});
        service.webSupport = const WebSupportDto(
          state: 'not_installed',
          version: 'test',
          diskBytes: 0,
          downloadedBytes: 0,
          source: 'fixture',
          supported: true,
        );
        final library = controller.wallpapers;
        await controller.refreshWebSupport();
        controller.wallpapers = library;
        await controller.apply('type-1');
        await controller.cycleWallpaper(1);
        expect(controller.appliedByOutput, {'A': 'type-3', 'B': 'type-3'});
        controller.wallpapers = [
          fixtures.wallpaper('unavailable', compatibility: 'unsupported'),
        ];
        final calls = service.calls.length;
        await controller.cycleWallpaper(1);
        expect(service.calls.length, calls);
      } finally {
        await controller.shutdown();
      }
    },
  );
}
