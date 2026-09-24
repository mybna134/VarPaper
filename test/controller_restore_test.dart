import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:varpaper/main.dart';
import 'package:varpaper/src/bridge_generated.dart/bridge.dart';
import 'package:varpaper/src/storage/settings_store.dart';

class _Store implements SettingsStore {
  _Store({SettingsDto? settings})
    : settings = settings ?? SettingsDto.defaults();

  final SettingsDto settings;
  final assignments = <String, WallpaperAssignmentRecord>{};

  @override
  Future<SettingsDto> load() async => settings;

  @override
  Future<void> save(SettingsDto settings) async {}

  @override
  Future<Map<String, WallpaperAssignmentRecord>> loadAssignments() async =>
      Map.of(assignments);

  @override
  Future<void> saveAssignment({
    required String output,
    required String sourcePath,
    String? sourceId,
  }) async {
    assignments[output] = WallpaperAssignmentRecord()
      ..output = output
      ..sourcePath = sourcePath
      ..sourceId = sourceId;
  }

  @override
  Future<void> saveDefaultAssignment({
    required String sourcePath,
    String? sourceId,
  }) async {
    assignments.clear();
    await saveAssignment(
      output: 'all',
      sourcePath: sourcePath,
      sourceId: sourceId,
    );
  }

  @override
  Future<void> removeAssignment(String output) async {
    assignments.remove(output);
  }

  @override
  Future<void> removeAllAssignments() async {
    assignments.clear();
  }

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

class _Service implements WayvidService {
  _Service(this.outputs);

  List<MonitorDto> outputs;
  final events = <ServiceEvent>[];
  final applied = <String>[];

  @override
  Future<ServiceInfo> initialize() async =>
      const ServiceInfo(workshopAvailable: false, engineRunning: true);

  @override
  Future<List<MonitorDto>> refreshMonitors() async => outputs;

  @override
  Future<List<ServiceEvent>> pollEvents() async {
    final result = List<ServiceEvent>.of(events);
    events.clear();
    return result;
  }

  @override
  Future<void> applyWallpaper({required String path, String? output}) async {
    applied.add(output ?? 'all');
  }

  @override
  Future<void> clearWallpaper({String? output}) async {}

  @override
  Future<void> shutdown() async {}

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

const _monitor = MonitorDto(
  name: 'HDMI-A-1',
  width: 1920,
  height: 1080,
  x: 0,
  y: 0,
  scale: 1,
  primary: true,
);

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  test(
    'saved wallpaper resumes at startup and after monitor reconnect',
    () async {
      final directory = await Directory.systemTemp.createTemp(
        'varpaper-restore-',
      );
      final source = File('${directory.path}/wallpaper.mp4');
      await source.writeAsString('fixture');
      final store = _Store();
      await store.saveAssignment(
        output: _monitor.name,
        sourcePath: source.path,
        sourceId: 'wallpaper-1',
      );
      final service = _Service([_monitor]);
      final controller = await WayvidController.create(
        store,
        serviceOverride: service,
      );
      try {
        expect(service.applied, [_monitor.name]);
        expect(controller.isAppliedTo('wallpaper-1', _monitor.name), isTrue);

        service.events.add(const ServiceEvent.outputsChanged(outputs: []));
        await controller.pollEvents();
        expect(controller.appliedByOutput, isEmpty);
        expect(store.assignments.containsKey(_monitor.name), isTrue);

        service.events.add(
          const ServiceEvent.outputsChanged(outputs: [_monitor]),
        );
        await controller.pollEvents();
        expect(service.applied, [_monitor.name, _monitor.name]);

        service.events.add(
          const ServiceEvent.outputsChanged(outputs: [_monitor]),
        );
        await controller.pollEvents();
        expect(service.applied.length, 2);

        await controller.clear(output: _monitor.name);
        service.events.add(const ServiceEvent.outputsChanged(outputs: []));
        service.events.add(
          const ServiceEvent.outputsChanged(outputs: [_monitor]),
        );
        await controller.pollEvents();
        expect(service.applied.length, 2);
        expect(store.assignments, isEmpty);
      } finally {
        await controller.shutdown();
        await directory.delete(recursive: true);
      }
    },
  );

  test('clearing one output overrides an all monitor assignment', () async {
    final directory = await Directory.systemTemp.createTemp(
      'varpaper-restore-',
    );
    final source = File('${directory.path}/wallpaper.mp4');
    await source.writeAsString('fixture');
    final store = _Store();
    await store.saveDefaultAssignment(
      sourcePath: source.path,
      sourceId: 'wallpaper-1',
    );
    final service = _Service([_monitor]);
    final controller = await WayvidController.create(
      store,
      serviceOverride: service,
    );
    try {
      expect(service.applied, [_monitor.name]);
      await controller.clear(output: _monitor.name);
      expect(store.assignments[_monitor.name]?.sourcePath, isEmpty);

      service.events.add(const ServiceEvent.outputsChanged(outputs: []));
      service.events.add(
        const ServiceEvent.outputsChanged(outputs: [_monitor]),
      );
      await controller.pollEvents();
      expect(service.applied, [_monitor.name]);
    } finally {
      await controller.shutdown();
      await directory.delete(recursive: true);
    }
  });

  test('restore setting disables startup replay', () async {
    final directory = await Directory.systemTemp.createTemp(
      'varpaper-restore-',
    );
    final source = File('${directory.path}/wallpaper.mp4');
    await source.writeAsString('fixture');
    final store = _Store(
      settings: SettingsDto.defaults().apply(
        const SettingsPatch(restoreLastWallpaper: false),
      ),
    );
    await store.saveAssignment(output: _monitor.name, sourcePath: source.path);
    final service = _Service([_monitor]);
    final controller = await WayvidController.create(
      store,
      serviceOverride: service,
    );
    try {
      expect(service.applied, isEmpty);
      service.events.add(const ServiceEvent.outputsChanged(outputs: []));
      service.events.add(
        const ServiceEvent.outputsChanged(outputs: [_monitor]),
      );
      await controller.pollEvents();
      expect(service.applied, isEmpty);
    } finally {
      await controller.shutdown();
      await directory.delete(recursive: true);
    }
  });
}
