import 'dart:async';
import 'dart:collection';

import 'bridge_generated.dart/bridge.dart';

/// Bounds encoded preview results retained by the controller, independently of
/// Flutter's decoded ImageCache and images still held by visible widgets.
class PreviewCache {
  PreviewCache({
    this.maxEntries = 100,
    this.maxBytes = 32 * 1024 * 1024,
    this.maxPending = 100,
  }) : assert(maxEntries > 0),
       assert(maxBytes >= 0),
       assert(maxPending > 0);

  final int maxEntries;
  final int maxBytes;
  final int maxPending;
  final _completed = LinkedHashMap<String, _CachedPreview>();
  final _pending = <(int, String), Future<PreviewDto?>>{};
  int _generation = 0;
  int _bytes = 0;
  bool _closed = false;

  int get entryCount => _completed.length;
  int get retainedBytes => _bytes;
  int get pendingCount => _pending.length;

  Future<PreviewDto?> load(String id, Future<PreviewDto> Function() loader) {
    if (_closed) return Future.value(null);
    final cached = _completed.remove(id);
    if (cached != null) {
      _completed[id] = cached;
      return cached.future;
    }
    final key = (_generation, id);
    final pending = _pending[key];
    if (pending != null) return pending;
    // Old-generation work still consumes slots. Refresh cannot bypass the
    // limit; saturation is transient and does not become a cached failure.
    if (_pending.length >= maxPending) return Future.value(null);

    final completer = Completer<PreviewDto?>();
    final future = completer.future;
    _pending[key] = future;
    () async {
      PreviewDto? preview;
      try {
        preview = await loader();
      } catch (_) {
        // Retain a bounded failure entry, preserving existing fallback behavior.
      }
      _pending.remove(key);
      if (!_closed && key.$1 == _generation) {
        final bytes = preview?.bytes.length ?? 0;
        if (bytes <= maxBytes) {
          _completed[id] = _CachedPreview(future, bytes);
          _bytes += bytes;
          while (_completed.length > maxEntries || _bytes > maxBytes) {
            final oldest = _completed.remove(_completed.keys.first)!;
            _bytes -= oldest.bytes;
          }
        }
      }
      completer.complete(preview);
    }();
    return future;
  }

  void clear() {
    _generation++;
    _completed.clear();
    _bytes = 0;
  }

  void close() {
    _closed = true;
    clear();
  }
}

class _CachedPreview {
  const _CachedPreview(this.future, this.bytes);
  final Future<PreviewDto?> future;
  final int bytes;
}
