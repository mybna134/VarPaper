import 'dart:convert';
import 'dart:ffi' as ffi;
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:isar_community/isar.dart';
import 'package:varpaper/src/storage/settings_store.dart';
import 'package:varpaper/src/bridge_generated.dart/bridge.dart';

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
