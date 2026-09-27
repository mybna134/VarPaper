import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:varpaper/src/l10n.dart';

void main() {
  test('English passes text through and Chinese translates known keys', () {
    const en = WayvidLocalizations(Locale('en'));
    const zh = WayvidLocalizations(Locale('zh'));
    expect(en.isChinese, isFalse);
    expect(zh.isChinese, isTrue);
    expect(en.text('Library'), 'Library');
    expect(zh.text('Library'), '壁纸库');
    expect(zh.text('Quit'), '退出');
    // Unknown keys fall back to the English source string.
    expect(zh.text('Untranslated string'), 'Untranslated string');
  });

  test('delegate supports en and zh only and loads matching locale', () async {
    const delegate = WayvidLocalizations.delegate;
    expect(delegate.isSupported(const Locale('en')), isTrue);
    expect(delegate.isSupported(const Locale('zh', 'CN')), isTrue);
    expect(delegate.isSupported(const Locale('fr')), isFalse);
    final loaded = await delegate.load(const Locale('zh'));
    expect(loaded.locale, const Locale('zh'));
    expect(loaded.text('Settings'), '设置');
    expect(delegate.shouldReload(delegate), isFalse);
  });

  testWidgets('of() uses the delegate and falls back to English', (
    tester,
  ) async {
    late WayvidLocalizations withoutDelegate;
    await tester.pumpWidget(
      Builder(
        builder: (context) {
          withoutDelegate = WayvidLocalizations.of(context);
          return const SizedBox();
        },
      ),
    );
    expect(withoutDelegate.isChinese, isFalse);

    late WayvidLocalizations withDelegate;
    await tester.pumpWidget(
      Localizations(
        locale: const Locale('zh'),
        delegates: const [
          WayvidLocalizations.delegate,
          DefaultWidgetsLocalizations.delegate,
        ],
        child: Builder(
          builder: (context) {
            withDelegate = WayvidLocalizations.of(context);
            return const SizedBox();
          },
        ),
      ),
    );
    await tester.pump();
    expect(withDelegate.text('About'), '关于');
  });
}
