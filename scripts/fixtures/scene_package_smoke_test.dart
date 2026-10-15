import 'dart:io';

import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:varpaper/src/bridge_generated.dart/bridge.dart';
import 'package:varpaper/src/bridge_generated.dart/frb_generated.dart';

const _config = EngineConfigDto(
  volume: 0,
  fpsLimit: 30,
  loopPlayback: true,
  layout: 'stretch',
  hwdec: 'no',
  mute: true,
  startTime: 0,
  playbackRate: 1,
  hdrMode: 'off',
  toneMappingAlgorithm: 'auto',
  toneMappingParam: 0,
  toneMappingMode: 'auto',
  toneMappingComputePeak: false,
  autoPlay: true,
  pauseOnBattery: false,
);

void main() {
  final library = Platform.environment['VARPAPER_TEST_PACKAGE_LIBRARY'];
  test(
    'packaged Rust Scene applies, rejects bad assets, pauses and clears',
    () async {
      // Explicitly enable only under an owned Xvfb session, never the desktop.
      expect(Platform.environment['VARPAPER_TEST_X11'], '1');
      expect(Platform.environment['XDG_SESSION_TYPE'], 'x11');
      await RustLib.init(externalLibrary: ExternalLibrary.open(library!));
      final service = await WayvidService.newInstance();
      final root = await Directory.systemTemp.createTemp(
        'varpaper-package-scene-',
      );
      try {
        await File('${root.path}/project.json').writeAsString(
          '{"type":"scene","file":"scene.json","title":"Package smoke"}',
        );
        final scene = File('${root.path}/scene.json');
        const general =
            '"camera":{"center":"0 0 0","eye":"0 0 1","up":"0 1 0"},"general":{"orthogonalprojection":{"width":64,"height":64},'
            '"clearcolor":"0.25 0.5 0.75"}';
        await scene.writeAsString('{$general,"objects":[]}');
        await service.createEngine(config: _config);

        Future<ServiceEvent> waitFor(
          bool Function(ServiceEvent) matches,
        ) async {
          final deadline = DateTime.now().add(const Duration(seconds: 15));
          while (DateTime.now().isBefore(deadline)) {
            for (final event in await service.pollEvents()) {
              if (matches(event)) return event;
              if (event is ServiceEvent_Error) fail(event.message);
            }
            await Future<void>.delayed(const Duration(milliseconds: 20));
          }
          throw StateError('No expected packaged Scene event');
        }

        final description = await service.describeWallpaper(path: root.path);
        expect(description.version, 1);
        expect(description.wallpaperType, 'scene');
        expect(description.propertyOverridesJson, '{}');
        await service.applyWallpaperSource(source: description);
        final applied = await waitFor(
          (event) => event is ServiceEvent_WallpaperApplied,
        );
        expect(
          (applied as ServiceEvent_WallpaperApplied).path,
          '${root.path}/project.json',
        );
        await service.pause();
        await service.resume();
        await service.updateEngineConfig(config: _config);

        final broken = await Directory('${root.path}/broken').create();
        await File('${broken.path}/project.json')
            .writeAsString('{"type":"scene","file":"scene.json"}');
        await File('${broken.path}/scene.json').writeAsString(
          '{$general,"objects":[{"id":1,"name":"Missing model","image":"missing.json","origin":"32 32 0","size":"64 64"}]}',
        );
        await expectLater(
          service.applyWallpaper(path: broken.path),
          throwsA(
            isA<BridgeError>()
                .having((error) => error.code, 'code', 'apply_failed')
                .having(
                  (error) => error.message,
                  'message',
                  contains('missing.json'),
                ),
          ),
        );
        final failure = await waitFor(
          (event) => event is ServiceEvent_WallpaperFailed,
        );
        expect((failure as ServiceEvent_WallpaperFailed).code, 'wallpaper_failed');
        expect(failure.path, '${broken.path}/project.json');
        expect(failure.message, contains('missing.json'));
        // Assignment persistence is Flutter-owned; monitor discovery only
        // reports outputs. A clear event confirms the failed candidate left
        // an active assignment for this output.
        expect(await service.refreshMonitors(), isNotEmpty);
        await service.clearWallpaper();
        await waitFor((event) => event is ServiceEvent_WallpaperCleared);
      } finally {
        await service.shutdown();
        RustLib.dispose();
        await root.delete(recursive: true);
      }
    },
    skip: library == null,
  );
}
