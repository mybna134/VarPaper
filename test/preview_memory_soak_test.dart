import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:varpaper/main.dart';
import 'package:varpaper/src/bridge_generated.dart/bridge.dart';

import 'support/fakes.dart';

// Opt-in controlled encoded-preview retention probe. Normal CI does not wait.
void main() {
  test('preview memory soak', () async {
    if (Platform.environment['VARPAPER_MEMORY_SOAK'] != '1') return;
    TestWidgetsFlutterBinding.ensureInitialized();
    final folder = await Directory.systemTemp.createTemp('varpaper-preview-');
    final image = File('${folder.path}/encoded-preview');
    await image.writeAsBytes(Uint8List(64 * 1024)..fillRange(0, 64 * 1024, 42));
    final service = _DiskPreviewService(image);
    final controller = await WayvidController.create(
      FakeStore(),
      serviceOverride: service,
    );
    controller.eventTimer?.cancel();
    final log = File(Platform.environment['VARPAPER_MEMORY_LOG']!);
    final start = Stopwatch()..start();
    var loaded = 0;
    try {
      for (var tick = 0; tick <= 156; tick++) {
        final delay = Duration(seconds: tick * 5) - start.elapsed;
        if (delay > Duration.zero) await Future<void>.delayed(delay);
        for (var i = 0; i < 8; i++) {
          await controller.thumbnail(wallpaper('preview-${loaded++}'));
        }
        final stats = File('/proc/self/smaps_rollup').readAsLinesSync().where(
          (line) =>
              ['Rss:', 'Pss:', 'Anonymous:', 'Swap:'].any(line.startsWith),
        );
        await log.writeAsString(
          '${start.elapsedMilliseconds / 1000}\t$loaded\t${stats.join('\t')}\n',
          mode: FileMode.append,
        );
      }
      expect(service.loads, 1256);
    } finally {
      await controller.shutdown();
      await folder.delete(recursive: true);
    }
  }, timeout: const Timeout(Duration(minutes: 15)));
}

class _DiskPreviewService extends FakeService {
  _DiskPreviewService(this.image);
  final File image;
  int loads = 0;
  @override
  Future<PreviewDto> loadPreview({
    required String wallpaperId,
    required String path,
    required String wallpaperType,
    required int width,
    required int height,
  }) async {
    loads++;
    return PreviewDto(bytes: await image.readAsBytes());
  }
}
