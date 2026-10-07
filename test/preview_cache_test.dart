import 'dart:async';
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';

import 'package:varpaper/src/bridge_generated.dart/bridge.dart';
import 'package:varpaper/src/preview_cache.dart';

PreviewDto preview(int size) => PreviewDto(bytes: Uint8List(size));

void main() {
  test(
    'default budgets remain bounded across 1256 disk-sized previews',
    () async {
      final cache = PreviewCache();
      for (var i = 0; i < 1256; i++) {
        await cache.load('$i', () async => preview(64 * 1024));
        expect(cache.entryCount, lessThanOrEqualTo(100));
        expect(cache.retainedBytes, lessThanOrEqualTo(32 * 1024 * 1024));
        expect(cache.pendingCount, 0);
      }
      expect(cache.entryCount, 100);
      expect(cache.retainedBytes, 100 * 64 * 1024);
      cache.clear();
      expect(cache.entryCount, 0);
      expect(cache.retainedBytes, 0);
    },
  );

  test('evicted previews reload current disk data', () async {
    final folder = await Directory.systemTemp.createTemp('preview-cache-');
    try {
      final file = File('${folder.path}/preview');
      await file.writeAsBytes([1]);
      final cache = PreviewCache(maxEntries: 1);
      Future<PreviewDto> load() async =>
          PreviewDto(bytes: await file.readAsBytes());
      expect((await cache.load('a', load))!.bytes, [1]);
      await file.writeAsBytes([2]);
      expect((await cache.load('a', load))!.bytes, [1]);
      await cache.load('b', () async => preview(0));
      expect((await cache.load('a', load))!.bytes, [2]);
    } finally {
      await folder.delete(recursive: true);
    }
  });

  test('access order and byte budget evict results for reload', () async {
    final cache = PreviewCache(maxEntries: 2, maxBytes: 6);
    var loads = 0;
    Future<PreviewDto> load() async {
      loads++;
      return preview(3);
    }

    final a = cache.load('a', load);
    await a;
    await cache.load('b', load);
    expect(identical(cache.load('a', load), a), isTrue);
    await cache.load('c', load);
    expect(cache.entryCount, 2);
    expect(cache.retainedBytes, 6);
    await cache.load('b', load);
    expect(loads, 4);
    await cache.load('large', () async => preview(5));
    expect(cache.entryCount, 1);
    expect(cache.retainedBytes, 5);
  });

  test('oversized previews are delivered without retention', () async {
    final cache = PreviewCache(maxBytes: 2);
    expect(
      (await cache.load('large', () async => preview(3)))!.bytes.length,
      3,
    );
    expect(cache.entryCount, 0);
    expect(cache.retainedBytes, 0);
    await cache.load('large', () async => preview(1));
    expect(cache.retainedBytes, 1);
  });

  test('path-only and failed results still obey entry budget', () async {
    final cache = PreviewCache(maxEntries: 1);
    final path = PreviewDto(imagePath: '/cached.png', bytes: Uint8List(0));
    expect(await cache.load('path', () async => path), path);
    var failures = 0;
    Future<PreviewDto> fail() async {
      failures++;
      throw StateError('load failed');
    }

    expect(await cache.load('bad', fail), isNull);
    expect(await cache.load('bad', fail), isNull);
    expect(failures, 1);
    expect(cache.entryCount, 1);
    expect(cache.retainedBytes, 0);
    await cache.load('path', () async => path);
    expect(cache.entryCount, 1);
  });

  test('deduplication and saturation span repeated refreshes', () async {
    final cache = PreviewCache(maxPending: 2);
    final first = Completer<PreviewDto>();
    final second = Completer<PreviewDto>();
    final a = cache.load('a', () => first.future);
    expect(
      identical(cache.load('a', () => throw StateError('duplicate')), a),
      isTrue,
    );
    final b = cache.load('b', () => second.future);
    for (var i = 0; i < 10; i++) {
      cache.clear();
      expect(
        await cache.load('c', () => throw StateError('saturated')),
        isNull,
      );
      expect(cache.pendingCount, 2);
    }
    first.complete(preview(1));
    await a;
    expect(cache.pendingCount, 1);
    expect(cache.entryCount, 0);
    await cache.load('c', () async => preview(2));
    second.complete(preview(3));
    await b;
    expect(cache.pendingCount, 0);
    expect(cache.entryCount, 1);
    expect(cache.retainedBytes, 2);
  });

  test('old completion cannot replace or remove new same-id work', () async {
    final cache = PreviewCache();
    final old = Completer<PreviewDto>();
    final current = Completer<PreviewDto>();
    final oldFuture = cache.load('a', () => old.future);
    cache.clear();
    final currentFuture = cache.load('a', () => current.future);
    old.complete(preview(5));
    await oldFuture;
    expect(cache.pendingCount, 1);
    expect(
      identical(
        cache.load('a', () => throw StateError('duplicate')),
        currentFuture,
      ),
      isTrue,
    );
    current.complete(preview(2));
    await currentFuture;
    expect(cache.retainedBytes, 2);
    expect(
      identical(
        cache.load('a', () => throw StateError('reload')),
        currentFuture,
      ),
      isTrue,
    );
  });

  test('close clears retained data and ignores pending completions', () async {
    final cache = PreviewCache();
    await cache.load('cached', () async => preview(3));
    final pending = Completer<PreviewDto>();
    final future = cache.load('pending', () => pending.future);
    cache.close();
    pending.complete(preview(4));
    await future;
    expect(cache.entryCount, 0);
    expect(cache.retainedBytes, 0);
    expect(cache.pendingCount, 0);
    expect(await cache.load('new', () => throw StateError('closed')), isNull);
  });
}
