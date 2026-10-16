// Opt-in acceptance: actual packaged Rust service and Settings install buttons.
import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:varpaper/main.dart';
import 'package:varpaper/src/bridge_generated.dart/bridge.dart';
import 'package:varpaper/src/bridge_generated.dart/frb_generated.dart';
import 'package:varpaper/src/l10n.dart';

import '../../test/support/fakes.dart' show FakeStore;

void main() {
  final library = Platform.environment['VARPAPER_TEST_PACKAGE_LIBRARY'];
  testWidgets(
    'package Settings explicitly installs and uninstalls Web support',
    (tester) async {
      expect(Platform.environment['VARPAPER_TEST_X11'], '1');
      final data = Platform.environment['XDG_DATA_HOME']!;
      expect(data, contains('varpaper-package-components.'));
      await tester.runAsync(() async {
        await RustLib.init(externalLibrary: ExternalLibrary.open(library!));
      });
      final service = await tester.runAsync(WayvidService.newInstance);
      final controller = (await tester.runAsync(
        () => WayvidController.create(FakeStore(), serviceOverride: service!),
      ))!;
      controller.eventTimer?.cancel();
      final project = Directory.systemTemp.createTempSync('varpaper-web-user-');
      final component = Directory('$data/varpaper/components/web');
      final media = Directory.systemTemp.createTempSync(
        'varpaper-package-media-',
      );
      Future<void> expectCallbackPixel(int red, int blue) async {
        final deadline = DateTime.now().add(const Duration(seconds: 20));
        List<int> pixel = [];
        do {
          final capture = await Process.run('python3', [
            'scripts/fixtures/x11_callback_pixel.py',
          ], stdoutEncoding: null);
          expect(capture.exitCode, 0, reason: '${capture.stderr}');
          pixel = capture.stdout as List<int>;
          if (pixel.length == 3 &&
              pixel[0] == red &&
              pixel[1] == 200 &&
              pixel[2] == blue) {
            debugPrint('Package Web callback pixel: $pixel');
            return;
          }
          await Future<void>.delayed(const Duration(milliseconds: 100));
        } while (DateTime.now().isBefore(deadline));
        fail('Expected Web callback pixel [$red, 200, $blue], got $pixel');
      }

      Future<void> apply(String path) async {
        debugPrint('Package fixture applying: $path');
        try {
          await service!.applyWallpaper(path: path);
        } on BridgeError catch (error) {
          fail('${error.code}: ${error.message}');
        }
        final deadline = DateTime.now().add(const Duration(seconds: 20));
        var applied = false;
        while (!applied && DateTime.now().isBefore(deadline)) {
          for (final event in await service.pollEvents()) {
            if (event is ServiceEvent_WallpaperApplied &&
                (event.path == path || event.path == '$path/project.json')) {
              applied = true;
            }
            if (event is ServiceEvent_WallpaperFailed) fail(event.message);
            if (event is ServiceEvent_Error) fail(event.message);
          }
          await Future<void>.delayed(const Duration(milliseconds: 20));
        }
        expect(applied, isTrue, reason: path);
        await service.pause();
        await service.resume();
      }

      try {
        expect(controller.engineRunning, isTrue, reason: controller.error);
        expect(controller.webSupport!.state, 'not_installed');
        expect(component.existsSync(), isFalse);
        File('${project.path}/project.json').writeAsStringSync(
          jsonEncode({
            'type': 'web',
            'file': 'index.html',
            'general': {
              'properties': {
                'enabled': {'type': 'bool', 'value': true},
                'speed': {'type': 'slider', 'value': 1, 'min': 0, 'max': 10},
                'caption': {'type': 'textinput', 'value': 'fixture'},
              },
            },
          }),
        );
        File('${project.path}/index.html').writeAsStringSync(
          '''<body style="margin:0;background:#ff0000"><script>
          let marker=0;
          window.wallpaperPropertyListener={applyUserProperties(p){
            if(typeof p.enabled.value==='boolean' && p.enabled.value &&
               typeof p.speed.value==='number' &&
               typeof p.caption.value==='string' && p.caption.value==='fixture')
              marker=p.speed.value===2?160:80;
          }};
          wallpaperRegisterAudioListener(a=>{
            if(!Array.isArray(a)||a.length!==128||!a.every(Number.isFinite))return;
            document.body.style.background='rgb('+marker+',200,'+
              (a.some(v=>v>0.00001)?200:20)+')';
          });</script>''',
        );
        await tester.runAsync(() async {
          final items = await service!.scanFolder(path: project.path);
          expect(items.single.wallpaperType, 'web');
          expect(items.single.compatibility, 'requires_renderer');
          await expectLater(
            service.applyWallpaper(path: project.path),
            throwsA(isA<BridgeError>()),
          );
          await service.pollEvents();
        });
        expect(component.existsSync(), isFalse);
        final video = '${media.path}/video.mp4';
        final gif = '${media.path}/animation.gif';
        final scene = Directory('${media.path}/scene')..createSync();
        File('${scene.path}/project.json')
            .writeAsStringSync('{"type":"scene","file":"scene.json"}');
        File('${scene.path}/scene.json').writeAsStringSync(
          '{"camera":{"center":"0 0 0","eye":"0 0 1","up":"0 1 0"},'
          '"general":{"orthogonalprojection":{"width":64,"height":64},'
          '"clearcolor":"0.25 0.5 0.75"},"objects":[]}',
        );
        final image = File('packaging/varpaper.png').absolute.path;
        await tester.runAsync(() async {
          expect(
            (await Process.run('ffmpeg', [
              '-v',
              'error',
              '-f',
              'lavfi',
              '-i',
              'color=c=red:s=64x32:r=20',
              '-t',
              '1',
              '-c:v',
              'mpeg4',
              video,
            ])).exitCode,
            0,
          );
          expect(
            (await Process.run('ffmpeg', [
              '-v',
              'error',
              '-i',
              video,
              gif,
            ])).exitCode,
            0,
          );
          for (final path in [video, image, gif, scene.path]) {
            await apply(path);
          }
          await service!.clearWallpaper();
        });
        // This is the current delivered subset, not a claim that live Scene
        // system audio or injectable local clock bindings are implemented.
        final support = File('docs/src/features/workshop.md')
            .readAsStringSync();
        expect(support, contains('live Scene system-audio visualization'));
        expect(support, contains('injected clock bindings'));
        debugPrint(
          'Scene live system audio / injected local clock: unavailable',
        );
        if (Platform.environment['VARPAPER_TEST_PACKAGE_DEFAULT_ONLY'] == '1') {
          debugPrint('Package default offline phase passed without CEF');
          return;
        }
        debugPrint('Package fixture opening Settings');
        controller.navigate('settings');
        tester.view.physicalSize = const Size(1200, 900);
        tester.view.devicePixelRatio = 1;
        await tester.pumpWidget(
          AnimatedBuilder(
            animation: controller,
            builder: (context, _) => MaterialApp(
              locale: const Locale('en'),
              localizationsDelegates: const [
                WayvidLocalizations.delegate,
                GlobalMaterialLocalizations.delegate,
                GlobalWidgetsLocalizations.delegate,
                GlobalCupertinoLocalizations.delegate,
              ],
              home: WayvidShell(controller: controller),
            ),
          ),
        );
        await tester.pumpAndSettle();
        await tester.scrollUntilVisible(
          find.text('Web wallpaper support'),
          400,
        );
        await tester.pumpAndSettle();
        debugPrint('Package fixture clicking Install');
        await tester.tap(find.text('Install'));
        await tester.runAsync(() async {
          final deadline = DateTime.now().add(const Duration(minutes: 10));
          do {
            await Future<void>.delayed(const Duration(milliseconds: 250));
            await controller.refreshWebSupport();
            if (controller.webSupport!.state == 'error') {
              fail(controller.webSupport!.error ?? 'Component install failed');
            }
          } while (controller.webSupport!.state != 'installed' &&
              DateTime.now().isBefore(deadline));
        });
        expect(controller.webSupport!.state, 'installed');
        expect(controller.webSupport!.diskBytes, greaterThan(0));
        await tester.runAsync(() async {
          for (final path in [video, image, gif, scene.path, project.path]) {
            await apply(path);
          }
          final available =
              Platform.environment['VARPAPER_TEST_AUDIO_AVAILABLE'] == '1';
          Process? tone;
          try {
            if (available) {
              final wav = '${media.path}/tone.wav';
              final generated = await Process.run('ffmpeg', [
                '-v',
                'error',
                '-f',
                'lavfi',
                '-i',
                'sine=frequency=375:sample_rate=48000:duration=60',
                wav,
              ]);
              expect(generated.exitCode, 0);
              tone = await Process.start('paplay', [wav]);
            }
            await expectCallbackPixel(80, available ? 200 : 20);
            final output = (await service!.refreshMonitors()).first.name;
            await service.updateWallpaperProperties(
              output: output,
              sourcePath: project.path,
              propertiesJson: '{"speed":{"value":2}}',
            );
            await expectCallbackPixel(160, available ? 200 : 20);
            debugPrint(
              'Package audio capture: ${available ? "available" : "unavailable (zero spectrum)"}',
            );
          } finally {
            tone?.kill();
            if (tone != null) await tone.exitCode;
          }
        });
        await tester.pumpAndSettle();
        debugPrint('Package fixture clicking Uninstall');
        await tester.tap(find.text('Uninstall'));
        await tester.runAsync(() async {
          final deadline = DateTime.now().add(const Duration(seconds: 30));
          do {
            await Future<void>.delayed(const Duration(milliseconds: 100));
            await controller.refreshWebSupport();
          } while (controller.webSupport!.state != 'not_installed' &&
              DateTime.now().isBefore(deadline));
        });
        expect(controller.webSupport!.state, 'not_installed');
        expect(controller.webSupport!.diskBytes, 0);
        expect(File('${project.path}/index.html').existsSync(), isTrue);
        expect(Directory('${component.path}/runtime').existsSync(), isFalse);
      } finally {
        await tester.pumpWidget(const SizedBox.shrink());
        await tester.runAsync(controller.shutdown);
        RustLib.dispose();
        tester.view.resetPhysicalSize();
        tester.view.resetDevicePixelRatio();
        project.deleteSync(recursive: true);
        media.deleteSync(recursive: true);
      }
    },
    timeout: const Timeout(Duration(minutes: 12)),
    skip:
        library == null ||
        Platform.environment['VARPAPER_TEST_COMPONENT_INSTALL'] != '1',
  );
}
