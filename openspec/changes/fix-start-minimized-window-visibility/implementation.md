# Implementation and validation

## Changes

- Linux runner no longer presents the top-level window on the first Flutter frame. Flutter view realization and plugin registration remain intact.
- `configureStartupWindow` applies saved window options and explicitly awaits normal-start show/focus. Hidden startup does not invoke either operation, and `runApp` still initializes the app and tray.
- All four tray restore paths use `showMainWindow`, clearing the startup taskbar hint before show/focus. Existing navigation, close-to-tray and quit behavior are retained.
- Added startup method-channel tests, a database close/reopen persistence test for both preference values, and ordered tray restore assertions including the plugin's internal minimized-state query.

## Automated checks

Validated on 2026-10-07:

- `cargo fmt --all -- --check`: passed.
- `RUSTFLAGS='-D warnings' cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `RUSTFLAGS='-D warnings' cargo llvm-cov --workspace --ignore-filename-regex frb_generated --lcov --output-path lcov-rust.info`: passed, 74 tests. The sandbox run stalled on local Wayland protocol sockets; the approved unsandboxed run passed.
- All seven script paths in the quality job passed `bash -n`; `packaging/flatpak/manifest.json` passed `python3 -m json.tool`.
- `flutter clean` and `flutter pub get`: completed.
- `flutter analyze lib/main.dart lib/src/l10n.dart lib/src/storage/settings_store.dart test`: passed after correcting a new test lint.
- `flutter test --coverage`: 45 tests passed, including saved wallpaper restoration and engine/controller tests. New test mocks were corrected to handle the window plugin's screen queries and minimized-state query.
- `flutter build linux --release`: passed.
- `openspec validate fix-start-minimized-window-visibility --strict` and `git diff --check`: passed.

The Flutter SDK is the project's configured Puro environment at `/home/mybna134/.puro/envs/varamusic/flutter`. Coverage outputs were copied to `/tmp/varpaper-startup-smoke/`; the tracked coverage file was restored to its pre-task contents. No push was requested or performed. Codecov upload actions were not run locally; a future push still requires all applicable quality steps to pass, including uploads.

## Desktop smoke checks

Used the release bundle with separate true/false preference databases under `/tmp/varpaper-startup-smoke/`, without touching the running user's application database. Each launch created a fresh process and was observed for approximately four seconds (40 samples). Wallpaper restoration was disabled in these isolated smoke settings.

### Wayland: Niri with registered StatusNotifier host

- Saved true: no main window mapped in any sample; the first run retained the same focused window throughout. Logs confirm integrated engine startup, and the tray was registered after Flutter initialization.
- Tray show/library/settings actions, dispatched through the test instance's D-Bus menu, each mapped and focused the hidden-start window. Closing it hid the window while keeping the process alive; subsequent actions restored it again.
- Saved false: normal startup mapped the main window (35–37 samples after initialization).
- The synthetic D-Bus action on the normal-start instance did not consistently obtain focus after hiding; this check failed its focus assertion. It does not establish behavior for an actual user click carrying desktop activation context. Retained as unverified rather than attributing it conclusively to either the compositor or application.
- A later baseline-focus comparison changed while no test window was mapped, so repeated tests limited the assertion to whether the test instance acquired focus. This shared desktop is not a controlled focus environment.

### X11 backend: Xwayland on the same desktop

- Launched with `GDK_BACKEND=x11`, `XDG_SESSION_TYPE=x11`, and no `WAYLAND_DISPLAY`.
- Saved true: zero visible X11 windows across all samples.
- Saved false: the normal-start window became visible (37 samples).
- All three tray menu actions showed the window; `xprop` confirmed `_NET_WM_STATE_SKIP_TASKBAR` was cleared each time.
- Logs confirm integrated engine startup for both values.

## Acceptance confirmation (task 3.1)

The agent verified native startup visibility on Wayland and the X11 backend as recorded above. On 2026-10-07, the user confirmed that the remaining acceptance verification was complete ("已经验证，完成"). Task 3.1 is therefore complete based on user verification. The user did not provide additional environment details or individual results; this confirmation is recorded separately from the agent-observed checks and does not imply additional agent-run desktop tests.
