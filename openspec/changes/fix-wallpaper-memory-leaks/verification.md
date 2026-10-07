# Verification — 2026-10-07

## Implemented behavior

Clear ends MPV playback and destroys its EGL/native wallpaper window. Replacement changes the source while retaining the native window and initialized EGL surface, including a Wayland surface still awaiting configure. Teardown rebinds the original context as a normal operation; context unbinding is not an exceptional condition. Session teardown precedes Wayland/X11 window destruction, and final context/display cleanup precedes native disconnection. Only displays initialized by this engine are terminated.

Wayland output removal and compositor closure end the associated session. Layer and wl_surface objects receive explicit destructor requests; supported wl_output bindings are released. Surface generations reject stale configure, closure and frame events. Engine state destruction also cleans up on error returns.

Completed encoded previews are retained by an LRU limited to 100 entries and 32 MiB. At most 100 requests remain outstanding across refresh generations. Refresh, shutdown and controller disposal invalidate late writes. Flutter's decoded image cache and preview references held by visible widgets are separate owners.

## Deterministic regression evidence

- Four native EGL handle tests under Xvfb validate real surface/context destruction, repeated destruction protection, owner lifetimes, three partial-initialization rollback points, and survival of an externally initialized display. Handle validity is queried while parent resources are still alive, so thread exit/display termination cannot hide a missing destructor.
- Native X11 tests verify MPV destruction with the original context current, sequential two-session cleanup, one injected recoverable binding failure, an unaffected second session still rendering, and a NUL-path initialization error whose diagnostic survives rollback.
- X11 native-window/surface identities remain stable on replacement; clear removes the window from the X server's tree. Twenty context lifetimes and 100 apply/render/clear cycles return their counts to baseline.
- The live engine regression ran 100 rendered apply/clear cycles across 20 starts/stops on **both X11 and a real nested Niri compositor**. Native resource snapshots are `(1 context, 1 surface)` during playback and `(1, 0)` after clear; shutdown checks `(0, 0)` on the owning engine thread. Events are drained.
- Three in-process Wayland protocol fixture tests observe actual wire destructor requests, ordering, duplicate cleanup, output removal/reconnection with named and fallback-name outputs, replacement before configure, and rejection of stale configure/closure/frame callbacks. Current callbacks still work.
- Flutter tests cover LRU access order, both budgets, oversized delivery without retention, path-only/failure entries, disk reload after eviction, pending deduplication/saturation across generations, delayed completions, shutdown/disposal and existing monitor assignment restoration. For the same 1256-preview workload used below, the helper retains exactly **100 entries / 6,553,600 encoded bytes**, zero outstanding loads, and zero entries/bytes after clear. These cache counts are verified by a separate deterministic regression, rather than inferred from RSS.

## Controlled memory measurements

Baseline: commit `f8434ae` (before this repair), built in an isolated temporary worktree. After: this working tree's production repair. Both use the same debug build/profile and workload code. No installed VarPaper process or desktop wallpaper was changed.

Every workload was sampled every 5 seconds for **780 seconds**, with the first 180 seconds treated as warm-up and the remaining 600 seconds as measurement: 157 samples per run. Raw RSS, PSS, anonymous and Swap values are preserved in [verification/](verification/); [summary.json](verification/summary.json) contains derived values. An unforced least-squares slope is calculated over the measurement window.

- Native playback: one disposable Xvfb output, 1280×720; the same 640×360 H.264 testsrc2 clip, 30 fps, 10 seconds looping; MPV software decoding, 30 fps limit, Mesa software OpenGL with two worker threads. `loop` keeps one session; `cycle` clears and reapplies every 15 seconds. Each workload uses its own display/process. GPU process memory is unavailable for this software-rendered test and is not reported as zero.
- Preview requests: the real controller with a fake service reading fresh 64 KiB encoded data from a temporary file. Eight unique previews every 5 seconds, **1256 unique loads** per run, Flutter 3.47.4/Dart 3.13.3. This isolates controller retention; it does not exercise decoded widget images, FRB preview generation or the disk cache generator.
- The host was under memory pressure and swapped pages during the runs. RSS/PSS alone would therefore understate retained memory. Anonymous + Swap is included below; absolute differences remain noisy because the processes share the host with other applications.

All sizes are MiB. Window change/slope refer to anonymous + Swap, rather than RSS alone.

| Workload | Final RSS | Final PSS | Final anonymous + Swap | Window change | MiB/min |
|---|---:|---:|---:|---:|---:|
| soak-before-loop | 138.93 | 58.95 | 83.44 | 2.5 | 0.196 |
| soak-after-loop | 134.17 | 59.17 | 83.0 | 2.76 | 0.226 |
| soak-before-cycle | 278.72 | 213.7 | 219.69 | 49.15 | 5.265 |
| soak-after-cycle | 280.56 | 243.9 | 218.36 | 33.39 | 3.33 |
| preview-before | 131.81 | 129.15 | 281.7 | 145.28 | 17.335 |
| preview-after | 97.9 | 91.14 | 195.93 | 60.17 | 8.85 |

Uninterrupted playback stayed below both investigation triggers (1 MiB/min and 32 MiB/window) before and after; this software workload does **not** reproduce the user's large long-running process footprint. Cycling and preview workloads exceed a trigger, so resource counts and allocation behavior were examined rather than declaring all memory growth fixed.

The additional 60-second allocator probe clears/reapplies every 5 seconds. After its initial warm-up, live malloc bytes remain around 67.5–67.7 MiB during playback while **free** allocator storage grows. After engine shutdown, live malloc drops to **996,208 bytes**, mmap allocations to zero, with **170,499,216 free allocator bytes** retained. A diagnostic-only malloc_trim did not return all free storage: RSS remained about 232 MiB and anonymous about 135 MiB. Thus substantial process RSS survives complete resource teardown without corresponding live malloc allocations. Trimming is not added to production. [Allocator probe](verification/allocator-probe.tsv) records the live/free measurements.

The repaired preview run ends about **85.8 MiB lower** in anonymous + Swap. Its observed memory falls after intermediate peaks, while deterministic cache counts remain bounded; VM/native allocator retention and temporary loads contribute to the process footprint. This is evidence for bounded encoded retention, not a complete Dart heap attribution or a claim that every source of the reported multi-GiB footprint is eliminated.

## Quality checks and remaining runtime limits

Passed Rust formatting, workspace Clippy with warnings denied, workspace LLVM coverage tests (32 engine + 32 library + 10 bridge tests), package shell syntax/Flatpak JSON validation, Flutter 3.47.4 analysis and coverage tests (**41 tests**), and strict OpenSpec validation. Native X11 cycles were enabled for the workspace coverage run; real Wayland cycles were checked separately under Niri. The initially available Flutter development SDK crashed in coverage finalization; the CI-pinned stable SDK passes.

No push, commit, coverage upload or archive was performed. Codecov uploads remain CI steps requiring its configured secret; the local coverage reports were generated. Hardware decoding, GPU process memory, an hours-long run on the user's real compositor/GPU and decoded gallery-widget memory were not measured. The repairs address demonstrated resource-lifetime/cache defects; the original uninterrupted real-desktop symptom is not declared resolved without that runtime evidence.

Reproduction entry points: `cargo test -p wayvid-engine` with `VARPAPER_TEST_X11=1` under Xvfb; set `VARPAPER_TEST_ENGINE_CYCLES=1` to run the real engine cycle regression, and `VARPAPER_TEST_WAYLAND=1` under a layer-shell compositor for Wayland enumeration. `cargo run -p wayvid-engine --example memory_soak -- <video> <loop|cycle> 780` records native memory (optional fourth argument is a cycle interval in 5-second ticks; optional fifth `trim` is a post-workload diagnostic). `VARPAPER_MEMORY_SOAK=1 VARPAPER_MEMORY_LOG=<path> flutter test test/preview_memory_soak_test.dart` runs the opt-in controller probe. Normal CI skips the 13-minute probe.
