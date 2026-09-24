import 'package:flutter_test/flutter_test.dart';
import 'package:varpaper/main.dart';
import 'package:varpaper/src/bridge_generated.dart/bridge.dart';
import 'package:varpaper/src/storage/settings_store.dart';

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
    required String sourcePath,
    String? sourceId,
  }) async {}

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Service implements WayvidService {
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
}
