import 'dart:convert';
import 'dart:ffi' as ffi;
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:isar_community/isar.dart';
import 'package:varpaper/main.dart';
import 'package:varpaper/src/bridge_generated.dart/bridge.dart';
import 'package:varpaper/src/storage/settings_store.dart';

class _Store implements SettingsStore {
  SettingsDto settings = SettingsDto.defaults().apply(
    const SettingsPatch(restoreLastWallpaper: false),
  );

  @override
  Future<SettingsDto> load() async => settings;

  @override
  Future<void> save(SettingsDto value) async => settings = value;

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Service implements WayvidService {
  int engineUpdates = 0;

  @override
  Future<ServiceInfo> initialize() async =>
      const ServiceInfo(workshopAvailable: false, engineRunning: true);

  @override
  Future<List<MonitorDto>> refreshMonitors() async => [];

  @override
  Future<void> updateEngineConfig({required EngineConfigDto config}) async {
    engineUpdates++;
  }

  @override
  Future<void> shutdown() async {}

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test('palette defaults and Isar round trip', () async {
    final configFile = File('.dart_tool/package_config.json').absolute;
    final packageConfig = jsonDecode(await configFile.readAsString());
    final packages = packageConfig['packages'] as List<dynamic>;
    final isarPackage = packages.firstWhere(
      (entry) => entry['name'] == 'isar_community_flutter_libs',
    );
    final packageRoot = configFile.uri.resolve(
      isarPackage['rootUri'] as String,
    );
    final libraryPath = '${File.fromUri(packageRoot).path}/linux/libisar.so';
    await Isar.initializeIsarCore(libraries: {ffi.Abi.current(): libraryPath});
    final directory = await Directory.systemTemp.createTemp(
      'varpaper-palette-',
    );
    final isar = await Isar.open(
      [AppSettingsRecordSchema, WallpaperAssignmentRecordSchema],
      directory: directory.path,
      name: 'palette_test',
    );
    final store = SettingsStore(isar);
    try {
      expect((await store.load()).gui.themeColor, defaultThemeColor);

      // A pre-palette record has zero/empty values for the new Isar fields.
      final oldRecord = AppSettingsRecord()
        ..themeColor = 0
        ..colorSchemeVariant = '';
      await isar.writeTxn(() => isar.appSettingsRecords.put(oldRecord));
      final migrated = await store.load();
      expect(migrated.gui.themeColor, defaultThemeColor);
      expect(migrated.gui.colorSchemeVariant, defaultColorSchemeVariant);

      final selected = migrated.apply(
        const SettingsPatch(
          themeColor: 0xff123456,
          customThemeColors: [0xff123456],
          colorSchemeVariant: 'vibrant',
        ),
      );
      await store.save(selected);
      final restored = await store.load();
      expect(restored.gui.themeColor, 0xff123456);
      expect(restored.gui.customThemeColors, [0xff123456]);
      expect(restored.gui.colorSchemeVariant, 'vibrant');
      expect(
        themeFromSettings(restored.gui, Brightness.dark).colorScheme,
        ColorScheme.fromSeed(
          seedColor: const Color(0xff123456),
          brightness: Brightness.dark,
          dynamicSchemeVariant: DynamicSchemeVariant.vibrant,
        ),
      );
    } finally {
      await isar.close(deleteFromDisk: true);
      await directory.delete(recursive: true);
    }
  });

  testWidgets('select, add, remove and reset palette in settings', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(1200, 900);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final store = _Store();
    final service = _Service();
    final controller = await WayvidController.create(
      store,
      serviceOverride: service,
    );
    controller.navigate('settings');
    try {
      await tester.pumpWidget(
        AnimatedBuilder(
          animation: controller,
          builder: (context, child) => MaterialApp(
            theme: themeFromSettings(controller.settings.gui, Brightness.light),
            home: WayvidShell(controller: controller),
          ),
        ),
      );
      final before = Theme.of(tester.element(find.text('Appearance')))
          .colorScheme
          .primary;

      await tester.tap(find.byTooltip('Theme color #03A9F4'));
      await tester.pumpAndSettle();
      expect(store.settings.gui.themeColor, 0xff03a9f4);
      expect(
        Theme.of(tester.element(find.text('Appearance'))).colorScheme.primary,
        isNot(before),
      );
      expect(service.engineUpdates, 0);

      await tester.tap(find.byTooltip('Add custom color'));
      await tester.pumpAndSettle();
      await tester.enterText(find.byType(TextField).last, '#123456');
      await tester.tap(find.text('Add color'));
      await tester.pumpAndSettle();
      expect(store.settings.gui.themeColor, 0xff123456);
      expect(store.settings.gui.customThemeColors, [0xff123456]);

      await tester.tap(find.byTooltip('Remove custom color'));
      await tester.pump();
      expect(store.settings.gui.themeColor, defaultThemeColor);
      expect(store.settings.gui.customThemeColors, isEmpty);

      await tester.tap(find.byTooltip('Theme color #03A9F4'));
      await tester.pump();
      await tester.tap(find.text('Tonal spot'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Vibrant').last);
      await tester.pumpAndSettle();
      expect(store.settings.gui.colorSchemeVariant, 'vibrant');

      await tester.tap(find.text('Reset palette'));
      await tester.pump();
      expect(store.settings.gui.themeColor, defaultThemeColor);
      expect(store.settings.gui.colorSchemeVariant, defaultColorSchemeVariant);
      expect(service.engineUpdates, 0);
    } finally {
      await tester.pumpWidget(const SizedBox.shrink());
      await controller.shutdown();
    }
  });
}
