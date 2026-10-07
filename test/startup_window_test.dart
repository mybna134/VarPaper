import 'dart:async';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:varpaper/main.dart';
import 'package:varpaper/src/storage/settings_store.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  const channel = MethodChannel('window_manager');
  const screenChannel = MethodChannel(
    'dev.leanflutter.plugins/screen_retriever',
  );
  final messenger =
      TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger;
  late List<MethodCall> calls;
  Completer<void>? showGate;

  setUp(() {
    calls = [];
    showGate = null;
    messenger.setMockMethodCallHandler(channel, (call) async {
      calls.add(call);
      if (call.method.startsWith('is')) return false;
      if (call.method == 'getBounds') {
        return {'x': 0.0, 'y': 0.0, 'width': 1100.0, 'height': 700.0};
      }
      if (call.method == 'show') await showGate?.future;
      return null;
    });
    messenger.setMockMethodCallHandler(screenChannel, (call) async {
      final display = {
        'id': 'test',
        'size': {'width': 1920.0, 'height': 1080.0},
        'visiblePosition': {'dx': 0.0, 'dy': 0.0},
      };
      if (call.method == 'getPrimaryDisplay') return display;
      if (call.method == 'getAllDisplays') {
        return {
          'displays': [display],
        };
      }
      return {'dx': 0.0, 'dy': 0.0};
    });
  });

  tearDown(() {
    messenger.setMockMethodCallHandler(channel, null);
    messenger.setMockMethodCallHandler(screenChannel, null);
  });

  for (final minimized in [true, false]) {
    test('startup configures window with startMinimized=$minimized', () async {
      final gui = SettingsDto.defaults().gui.copyWith(
        startMinimized: minimized,
        windowWidth: 1100,
        windowHeight: 700,
      );
      await configureStartupWindow(gui);

      final methods = calls.map((call) => call.method).toList();
      expect(methods.first, 'waitUntilReadyToShow');
      for (final entry in {
        'setBounds': [1100.0, 700.0],
        'setMinimumSize': [800.0, 600.0],
      }.entries) {
        final args =
            calls.firstWhere((call) => call.method == entry.key).arguments
                as Map;
        expect(args['width'], entry.value[0]);
        expect(args['height'], entry.value[1]);
      }
      final position =
          calls.lastWhere((call) => call.method == 'setBounds').arguments
              as Map;
      expect(position['x'], 410.0);
      expect(position['y'], 190.0);
      expect(calls.singleWhere((call) => call.method == 'setTitle').arguments, {
        'title': 'VarPaper',
      });
      expect(
        calls.singleWhere((call) => call.method == 'setSkipTaskbar').arguments,
        {'isSkipTaskbar': minimized},
      );
      if (minimized) {
        expect(methods, isNot(contains('show')));
        expect(methods, isNot(contains('focus')));
      } else {
        expect(methods.sublist(methods.length - 2), ['show', 'focus']);
      }
    });
  }

  test('startup waits for show before focusing and completing', () async {
    showGate = Completer<void>();
    var completed = false;
    final startup = configureStartupWindow(SettingsDto.defaults().gui)
        .then((_) {
          completed = true;
        });
    // Drain configuration calls while the native show response is pending.
    await Future<void>.delayed(Duration.zero);
    expect(calls.last.method, 'show');
    expect(completed, isFalse);
    expect(calls.map((call) => call.method), isNot(contains('focus')));
    showGate!.complete();
    await startup;
    expect(calls.last.method, 'focus');
    expect(completed, isTrue);
  });
}
