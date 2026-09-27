import 'dart:convert';

import 'package:varpaper/src/bridge_generated.dart/bridge.dart';
import 'package:varpaper/src/storage/settings_store.dart';

/// In-memory [SettingsStore] that records every persistence call.
class FakeStore implements SettingsStore {
  FakeStore({SettingsDto? settings})
    : settings =
          settings ??
          SettingsDto.defaults().apply(
            const SettingsPatch(restoreLastWallpaper: false),
          );

  SettingsDto settings;
  int saves = 0;
  final autostart = <bool>[];
  final assignments = <String, WallpaperAssignmentRecord>{};

  @override
  Future<SettingsDto> load() async => settings;

  @override
  Future<void> save(SettingsDto value) async {
    saves++;
    settings = value;
  }

  @override
  Future<void> syncAutostart(bool enabled) async => autostart.add(enabled);

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
    assignments
      ..clear()
      ..['all'] = (WallpaperAssignmentRecord()
        ..output = 'all'
        ..sourcePath = sourcePath
        ..sourceId = sourceId);
  }

  @override
  Future<void> removeAssignment(String output) async =>
      assignments.remove(output);

  @override
  Future<void> removeAllAssignments() async => assignments.clear();

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

WallpaperDto wallpaper(
  String id, {
  String? name,
  String sourceType = 'local_file',
  String category = 'video',
  String? title,
  String? author,
  String? description,
  List<String> tags = const [],
}) => WallpaperDto(
  id: id,
  name: name ?? id,
  sourcePath: '/wallpapers/$id.mp4',
  sourceType: sourceType,
  wallpaperCategory: category,
  wallpaperType: category,
  metadata: WallpaperMetadataDto(
    title: title,
    author: author,
    description: description,
    tags: tags,
  ),
  addedAt: '',
);

MonitorDto monitor(String name, {bool primary = false, String? current}) =>
    MonitorDto(
      name: name,
      width: 1920,
      height: 1080,
      x: 0,
      y: 0,
      scale: 1,
      primary: primary,
      currentWallpaper: current,
    );

/// 1x1 transparent PNG.
final tinyPng = base64Decode(
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA'
  '60e6kgAAAABJRU5ErkJggg==',
);

/// Scriptable [WayvidService] that records every call made by the controller.
class FakeService implements WayvidService {
  FakeService({
    this.engineRunning = true,
    this.workshopAvailable = false,
    List<MonitorDto>? monitors,
    Map<String, List<WallpaperDto>>? folders,
    this.workshop = const [],
  }) : monitors = monitors ?? [monitor('A', primary: true)],
       folders = folders ?? {};

  bool engineRunning;
  final bool workshopAvailable;
  List<MonitorDto> monitors;
  final Map<String, List<WallpaperDto>> folders;
  final List<WallpaperDto> workshop;

  Object? createEngineError;
  Object? pollError;
  final events = <ServiceEvent>[];
  final calls = <String>[];
  final configs = <EngineConfigDto>[];
  final previews = <String, PreviewDto>{};
  int previewLoads = 0;

  @override
  Future<ServiceInfo> initialize() async => ServiceInfo(
    workshopAvailable: workshopAvailable,
    engineRunning: engineRunning,
  );

  @override
  Future<void> createEngine({required EngineConfigDto config}) async {
    calls.add('createEngine');
    if (createEngineError != null) throw createEngineError!;
    configs.add(config);
  }

  @override
  Future<List<MonitorDto>> refreshMonitors() async => monitors;

  @override
  Future<List<WallpaperDto>> scanFolder({required String path}) async {
    calls.add('scan:$path');
    return folders[path] ?? const [];
  }

  @override
  Future<List<WallpaperDto>> scanWorkshop() async {
    calls.add('scanWorkshop');
    return workshop;
  }

  @override
  Future<List<ServiceEvent>> pollEvents() async {
    if (pollError != null) throw pollError!;
    final result = List<ServiceEvent>.of(events);
    events.clear();
    return result;
  }

  @override
  Future<void> applyWallpaper({required String path, String? output}) async =>
      calls.add('apply:${output ?? 'all'}:$path');

  @override
  Future<void> clearWallpaper({String? output}) async =>
      calls.add('clear:${output ?? 'all'}');

  @override
  Future<void> pause({String? output}) async =>
      calls.add('pause:${output ?? 'all'}');

  @override
  Future<void> resume({String? output}) async =>
      calls.add('resume:${output ?? 'all'}');

  @override
  Future<void> updateEngineConfig({required EngineConfigDto config}) async =>
      configs.add(config);

  @override
  Future<PreviewDto> loadPreview({
    required String wallpaperId,
    required String path,
    required String wallpaperType,
    required int width,
    required int height,
  }) async {
    previewLoads++;
    final preview = previews[wallpaperId];
    if (preview == null) throw StateError('no preview');
    return preview;
  }

  @override
  Future<void> openUrl({required String url}) async => calls.add('open:$url');

  @override
  Future<void> shutdown() async => calls.add('shutdown');

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}
