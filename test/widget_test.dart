import 'package:flutter_test/flutter_test.dart';
import 'package:wayvid_gui/main.dart';

void main() {
  test('Flutter bridge app exports its root widget', () {
    expect(WayvidApp, isNotNull);
  });
}
