import 'dart:convert';
import 'dart:ffi' as ffi;
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:isar_community/isar.dart';
import 'package:varpaper/main.dart';
import 'package:varpaper/src/storage/settings_store.dart';
import 'package:varpaper/src/bridge_generated.dart/bridge.dart';

import 'support/fakes.dart';

Future<void> _initIsarCore() async {
  final configFile = File('.dart_tool/package_config.json').absolute;
  final packageConfig = jsonDecode(await configFile.readAsString());
  final packages = packageConfig['packages'] as List<dynamic>;
  final isarPackage = packages.firstWhere(
    (entry) => entry['name'] == 'isar_community_flutter_libs',
  );
  final packageRoot = configFile.uri.resolve(isarPackage['rootUri'] as String);
  final libraryPath = '${File.fromUri(packageRoot).path}/linux/libisar.so';
  await Isar.initializeIsarCore(libraries: {ffi.Abi.current(): libraryPath});
}

WallpaperSourceDto _source(String path, String id, {String type = 'video'}) =>
    WallpaperSourceDto(
      version: 1,
      sourceId: id,
      sourcePath: path,
      wallpaperType: type,
      propertyOverridesJson: '{}',
    );

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  late Directory directory;
  late Isar isar;
  late SettingsStore store;

  setUp(() async {
    await _initIsarCore();
    directory = await Directory.systemTemp.createTemp('varpaper-store-');
    isar = await Isar.open(
      [AppSettingsRecordSchema, WallpaperAssignmentRecordSchema],
      directory: directory.path,
      name: 'store_test',
    );
    store = SettingsStore(isar);
  });

  tearDown(() async {
    await isar.close(deleteFromDisk: true);
    await directory.delete(recursive: true);
  });

  test(
    'assignment descriptions reject incomplete records and unknown versions',
    () async {
      final source = _source('/valid.mp4', 'valid');
      for (final damage in <void Function(WallpaperAssignmentRecord)>[
        (record) => record.sourceVersion = null,
        (record) => record.sourceVersion = 2,
        (record) => record.sourceId = null,
        (record) => record.wallpaperType = null,
        (record) => record.propertyOverridesJson = null,
      ]) {
        final record = WallpaperAssignmentRecord()
          ..setSourceDescription(source);
        damage(record);
        expect(() => record.sourceDescription, throwsStateError);
      }
      final record = WallpaperAssignmentRecord()..setSourceDescription(source);
      const unsupported = WallpaperSourceDto(
        version: 2,
        sourceId: 'future',
        sourcePath: '/future.mp4',
        wallpaperType: 'video',
        propertyOverridesJson: '{}',
      );
      expect(
        () => record.setSourceDescription(unsupported),
        throwsArgumentError,
      );
      expect(record.sourceDescription, source);
      await store.saveAssignment(output: 'A', source: source);
      await expectLater(
        store.saveAssignment(output: 'A', source: unsupported),
        throwsArgumentError,
      );
      expect((await store.loadAssignments())['A']!.sourceDescription, source);
      await store.saveAssignment(output: 'A', source: null);
      final cleared = (await store.loadAssignments())['A']!;
      expect(cleared.sourceDescription, isNull);
      expect(cleared.sourcePath, isEmpty);
      expect([
        cleared.sourceVersion,
        cleared.sourceId,
        cleared.wallpaperType,
        cleared.projectSourceJson,
        cleared.propertyOverridesJson,
      ], everyElement(isNull));
    },
  );

  test(
    'runtime fps patches retain, clear and accept the u32 maximum',
    () async {
      final defaults = SettingsDto.defaults();
      final limited = defaults.apply(
        SettingsPatch(fpsLimit: FpsLimitPatch(value: 0xffffffff)),
      );
      final retained = limited.apply(SettingsPatch(fpsLimit: FpsLimitPatch()));
      expect(retained.playback.fpsLimit, 0xffffffff);
      final unlimited = retained.apply(
        SettingsPatch(fpsLimit: FpsLimitPatch.unlimited()),
      );
      expect(unlimited.playback.fpsLimit, isNull);
      await store.save(unlimited);
      expect((await store.load()).playback.fpsLimit, isNull);
    },
  );

  test(
    'controller persists defaults before initializing the service',
    () async {
      expect(await isar.appSettingsRecords.get(0), isNull);
      final service = _PersistedConfigService(store);
      final controller = await WayvidController.create(
        store,
        serviceOverride: service,
      );
      controller.eventTimer?.cancel();
      try {
        expect(service.initializedWithRecord, isTrue);
        expect(service.configs, [SettingsDto.defaults().toEngineConfig()]);
        expect(await isar.appSettingsRecords.count(), 1);
      } finally {
        await controller.shutdown();
        controller.dispose();
      }
    },
  );

  test('reopened settings supply the complete engine configuration', () async {
    final defaults = SettingsDto.defaults();
    await store.save(
      SettingsDto(
        gui: defaults.gui.copyWith(language: 'zh', windowWidth: 1440),
        playback: defaults.playback.copyWith(
          volume: 0.35,
          fpsLimit: 48,
          loopMode: false,
          layout: 'contain',
          hwdec: 'no',
          mute: false,
          startTime: 2,
          playbackRate: 1.25,
          hdrMode: 'off',
          toneMappingAlgorithm: 'reinhard',
          toneMappingParam: 0.5,
          toneMappingMode: 'rgb',
          toneMappingComputePeak: false,
        ),
        autostartEnabled: false,
        restoreLastWallpaper: false,
        power: defaults.power.copyWith(pauseOnBattery: true),
        libraryFolders: const ['/saved-wallpapers'],
      ),
    );
    await isar.close();
    isar = await Isar.open(
      [AppSettingsRecordSchema, WallpaperAssignmentRecordSchema],
      directory: directory.path,
      name: 'store_test',
    );
    store = SettingsStore(isar);
    final service = _PersistedConfigService(store);
    final controller = await WayvidController.create(
      store,
      serviceOverride: service,
    );
    controller.eventTimer?.cancel();
    try {
      expect(controller.settings.gui.language, 'zh');
      expect(controller.settings.gui.windowWidth, 1440);
      expect(service.calls, contains('scan:/saved-wallpapers'));
      expect(service.configs, [
        const EngineConfigDto(
          volume: 0.35,
          fpsLimit: 48,
          loopPlayback: false,
          layout: 'contain',
          hwdec: 'no',
          mute: false,
          startTime: 2,
          playbackRate: 1.25,
          hdrMode: 'off',
          toneMappingAlgorithm: 'reinhard',
          toneMappingParam: 0.5,
          toneMappingMode: 'rgb',
          toneMappingComputePeak: false,
          autoPlay: true,
          pauseOnBattery: true,
        ),
      ]);
    } finally {
      await controller.shutdown();
      controller.dispose();
    }
  });

  test(
    'runtime config is committed to Isar before each service update',
    () async {
      final service = _PersistedConfigService(store);
      final controller = await WayvidController.create(
        store,
        serviceOverride: service,
      );
      controller.eventTimer?.cancel();
      try {
        service.configs.clear();
        await controller.update(const SettingsPatch(theme: 'dark'));
        expect((await store.load()).gui.theme, 'dark');
        expect(service.configs, isEmpty);

        for (final patch in [
          const SettingsPatch(volume: 0.6),
          const SettingsPatch(loopMode: false),
          const SettingsPatch(mute: false),
          const SettingsPatch(fpsLimit: FpsLimitPatch(value: 24)),
          const SettingsPatch(pauseOnBattery: true),
          const SettingsPatch(fpsLimit: FpsLimitPatch.unlimited()),
        ]) {
          final before = service.configs.length;
          await controller.update(patch);
          expect(controller.error, isNull);
          expect(service.configs, hasLength(before + 1));
          expect(service.configs.last, (await store.load()).toEngineConfig());
        }
        expect(service.configs.last.volume, 0.6);
        expect(service.configs.last.loopPlayback, isFalse);
        expect(service.configs.last.mute, isFalse);
        expect(service.configs.last.pauseOnBattery, isTrue);
        expect(service.configs.last.fpsLimit, isNull);

        service.events.add(const ServiceEvent.engineStopped());
        await controller.pollEvents();
        final before = service.configs.length;
        await controller.update(const SettingsPatch(volume: 0.2));
        expect((await store.load()).playback.volume, 0.2);
        expect(service.configs, hasLength(before));
      } finally {
        await controller.shutdown();
        controller.dispose();
      }
    },
  );

  test('invalid engine settings never reach Isar or the service', () async {
    final service = _PersistedConfigService(store);
    final controller = await WayvidController.create(
      store,
      serviceOverride: service,
    );
    controller.eventTimer?.cancel();
    try {
      final original = (await store.load()).toEngineConfig();
      service.configs.clear();
      for (final patch in const [
        SettingsPatch(volume: -0.1),
        SettingsPatch(volume: 1.1),
        SettingsPatch(volume: double.nan, theme: 'dark'),
        SettingsPatch(volume: double.infinity),
        SettingsPatch(volume: -double.infinity),
        SettingsPatch(fpsLimit: FpsLimitPatch(value: 0)),
        SettingsPatch(fpsLimit: FpsLimitPatch(value: -1)),
        SettingsPatch(fpsLimit: FpsLimitPatch(value: 0x100000000)),
      ]) {
        await controller.update(patch);
        expect(controller.error, isNotNull);
        expect(controller.settings.toEngineConfig(), original);
        expect((await store.load()).toEngineConfig(), original);
        expect((await store.load()).gui.theme, 'system');
        expect(service.configs, isEmpty);
      }
      // Valid boundary values still persist and reach the engine.
      for (final volume in [0.0, 1.0]) {
        await controller.update(SettingsPatch(volume: volume));
        expect(controller.error, isNull);
        expect(service.configs.last.volume, volume);
      }
    } finally {
      await controller.shutdown();
      controller.dispose();
    }
  });

  test('start minimized defaults to false and survives reopening', () async {
    expect((await store.load()).gui.startMinimized, isFalse);
    for (final enabled in [true, false]) {
      await store.save(
        (await store.load()).apply(SettingsPatch(startMinimized: enabled)),
      );
      await isar.close();
      isar = await Isar.open(
        [AppSettingsRecordSchema, WallpaperAssignmentRecordSchema],
        directory: directory.path,
        name: 'store_test',
      );
      store = SettingsStore(isar);
      expect((await store.load()).gui.startMinimized, enabled);
    }
  });

  test('per-output assignments are upserted, listed and removed', () async {
    expect(await store.loadAssignments(), isEmpty);

    await store.saveAssignment(
      output: 'A',
      source: _source('/a.mp4', '/a.mp4', type: 'video'),
    );
    await store.saveAssignment(
      output: 'B',
      source: _source('/b.mp4', 'b', type: 'video'),
    );
    // Saving again for the same output updates the existing record.
    await store.saveAssignment(
      output: 'A',
      source: _source('/a2.mp4', 'a2', type: 'video'),
    );

    var assignments = await store.loadAssignments();
    expect(assignments.keys, unorderedEquals(['A', 'B']));
    expect(assignments['A']!.sourcePath, '/a2.mp4');
    expect(assignments['A']!.sourceId, 'a2');
    expect(assignments['B']!.sourceId, 'b');

    await store.removeAssignment('A');
    assignments = await store.loadAssignments();
    expect(assignments.keys, ['B']);

    await store.removeAllAssignments();
    expect(await store.loadAssignments(), isEmpty);
  });

  test(
    'project descriptions and overrides survive reopening and replacement',
    () async {
      for (final kind in ['scene', 'web']) {
        final source = WallpaperSourceDto(
          version: 1,
          sourceId: '$kind-id',
          sourcePath: '/$kind/project.json',
          wallpaperType: kind,
          projectSourceJson: '{"properties":{"caption":{"value":"initial"}}}',
          propertyOverridesJson: kind == 'web'
              ? '{"caption":{"value":"saved"}}'
              : '{}',
        );
        await store.saveAssignment(output: kind, source: source);
      }
      await isar.close();
      isar = await Isar.open(
        [AppSettingsRecordSchema, WallpaperAssignmentRecordSchema],
        directory: directory.path,
        name: 'store_test',
      );
      store = SettingsStore(isar);
      final restored = await store.loadAssignments();
      expect(restored['scene']!.sourceDescription!.wallpaperType, 'scene');
      expect(
        restored['scene']!.sourceDescription!.projectSourceJson,
        contains('initial'),
      );
      expect(
        restored['web']!.sourceDescription!.propertyOverridesJson,
        contains('saved'),
      );
      expect(restored['web']!.sourceId, 'web-id');
      await store.saveAssignment(
        output: 'web',
        source: _source('/replacement.mp4', '/replacement.mp4', type: 'video'),
      );
      final replaced = (await store.loadAssignments())['web']!;
      expect(replaced.sourceDescription!.wallpaperType, 'video');
      expect(replaced.propertyOverridesJson, '{}');
    },
  );

  test('default assignment replaces all per-output assignments', () async {
    await store.saveAssignment(
      output: 'A',
      source: _source('/a.mp4', '/a.mp4', type: 'video'),
    );
    await store.saveAssignment(
      output: 'B',
      source: _source('/b.mp4', '/b.mp4', type: 'video'),
    );

    await store.saveDefaultAssignment(
      source: _source('/all.mp4', 'x', type: 'video'),
    );

    final assignments = await store.loadAssignments();
    expect(assignments.keys, ['all']);
    expect(assignments['all']!.sourcePath, '/all.mp4');
    expect(assignments['all']!.sourceId, 'x');
  });

  test('settings patches survive a save/load round trip', () async {
    final settings = SettingsDto.defaults().apply(
      const SettingsPatch(
        theme: 'dark',
        language: 'zh',
        volume: 0.4,
        loopMode: false,
        mute: false,
        fpsLimit: FpsLimitPatch(value: 30),
        pauseOnBattery: true,
        pauseOnFullscreen: false,
        autostartEnabled: true,
        restoreLastWallpaper: false,
        sidebarCollapsed: true,
        libraryFolders: ['/w1', '/w2'],
      ),
    );
    await store.save(settings);
    final loaded = await store.load();

    expect(loaded.gui.theme, 'dark');
    expect(loaded.gui.language, 'zh');
    expect(loaded.gui.sidebarCollapsed, isTrue);
    expect(loaded.playback.volume, closeTo(0.4, 1e-9));
    expect(loaded.playback.loopMode, isFalse);
    expect(loaded.playback.mute, isFalse);
    expect(loaded.playback.fpsLimit, 30);
    expect(loaded.power.pauseOnBattery, isTrue);
    expect(loaded.power.pauseOnFullscreen, isFalse);
    expect(loaded.autostartEnabled, isTrue);
    expect(loaded.restoreLastWallpaper, isFalse);
    expect(loaded.libraryFolders, ['/w1', '/w2']);

    final engine = loaded.toEngineConfig();
    expect(engine.fpsLimit, 30);
    expect(engine.loopPlayback, isFalse);
    expect(engine.pauseOnBattery, isTrue);

    final unlimited = loaded.apply(
      const SettingsPatch(fpsLimit: FpsLimitPatch.unlimited()),
    );
    expect(unlimited.playback.fpsLimit, isNull);
    // Fields not in the patch are left untouched.
    expect(unlimited.gui.theme, 'dark');
  });
}

/// Read the real database at the service boundary, before recording each call.
class _PersistedConfigService extends FakeService {
  _PersistedConfigService(this.store) : super(engineRunning: false);

  final SettingsStore store;
  bool initializedWithRecord = false;

  @override
  Future<ServiceInfo> initialize() async {
    initializedWithRecord = await store.isar.appSettingsRecords.get(0) != null;
    return super.initialize();
  }

  Future<void> _expectPersisted(EngineConfigDto config) async {
    expect(initializedWithRecord, isTrue);
    expect((await store.load()).toEngineConfig(), config);
  }

  @override
  Future<void> createEngine({required EngineConfigDto config}) async {
    await _expectPersisted(config);
    await super.createEngine(config: config);
  }

  @override
  Future<void> updateEngineConfig({required EngineConfigDto config}) async {
    await _expectPersisted(config);
    await super.updateEngineConfig(config: config);
  }
}
