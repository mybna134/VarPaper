# Tasks

## 1. EGL Ownership and Failure Cleanup

- [x] 1.1 Implement stable thread-local EGL ownership in `egl.rs`, surface release state, and parent lifetime protection; verify native acquisition/release tests cover drop, explicit destruction followed by drop, foreign/released surface rejection, and surfaces outliving their public context wrapper.
- [x] 1.2 Guard partial EGL initialization and release owned contexts/display resources without terminating Flutter's display; verify fault injection at API binding, config selection, and context creation pairs successful acquisitions with release attempts and a live UI graphics owner survives engine cleanup.
- [x] 1.3 Add the native handle regression under Xvfb and document ownership/current-context invariants in the resource wrappers; verify a surface is invalid after release while its parent display remains live, and a context is invalid after final owner release without thread exit hiding missing destructors.

## 2. Ordered Playback Session Teardown

- [x] 2.1 Unify explicit and drop-based session teardown in `session.rs`/`mpv.rs`, bind the original GL context before freeing the render context, and release MPV before EGL/native resources; verify a cleanup-order test and two initialized sessions cleaned sequentially after the first unbinds EGL.
- [x] 2.2 Preserve the primary initialization error and retain ownership on a recoverable make-current failure so normal teardown can retry before native windows are destroyed; verify injected initialization/binding failure tests diagnose the error, clean resources on retry, and avoid double destruction; document the normal original-context cleanup contract.
- [x] 2.3 Integrate ordered session teardown with X11 clear, output removal, and shutdown, destroying the native window after playback cleanup; verify clear destroys the window, reapply creates a new window, hot-swap retains the current native window/EGL surface, repeated clear is safe, and an unaffected second output still renders under Xvfb.

## 3. Wayland Output and Surface Teardown

- [x] 3.1 Centralize Wayland cleanup for clear, output removal, compositor closure, shutdown, and event-loop errors, releasing session resources before native surfaces; preserve native window/EGL surface reuse on replacement, including replacement before configuration completes; verify clear removes session/layer entries, reapply creates new surfaces, and named/fallback-name outputs follow the correct destruction order.
- [x] 3.2 Explicitly destroy owned layer/surface objects on clear, removal, closure, and shutdown, and release output bindings where supported on output removal/shutdown, with idempotent drop protection; verify a Wayland protocol fixture or instrumented compositor run sees no window destruction on replacement, actual destruction on clear/removal/closure/shutdown, and no double destruction after clear followed by closure/removal.
- [x] 3.3 Add generation identity to configure, closure, and frame callback user data; verify old events cannot mutate or destroy a same-name replacement, and current events still configure and render it.
- [x] 3.4 Document teardown ownership in the backend code and verify output removal/reconnection retains existing notification and assignment-restoration behavior using controller restoration tests plus a compositor or fixture hotplug test.

## 4. Bounded Flutter Preview Cache

- [x] 4.1 Replace unbounded Future retention with a tested cache helper using LRU limits of 100 completed entries and 32 MiB encoded bytes; verify access-order eviction, byte accounting, path-only/failure entries, and delivery without retention of oversized previews.
- [x] 4.2 Bound outstanding request tracking to 100 loads across cache generations, preserve same-generation deduplication, and make saturation transient; verify fake-service load counts, Future sharing, saturation/retry, and repeated refresh never exceeding the outstanding-work limit.
- [x] 4.3 Integrate refresh/shutdown invalidation with the controller and prevent old completions from removing or replacing newer entries; verify delayed-completion tests, the existing thumbnail failure test, and disk-backed reload after eviction, and document the distinction between encoded preview retention and Flutter's decoded image cache.

## 5. Integration and Memory Verification

- [x] 5.1 Run at least 100 apply/render/clear cycles and 20 engine start/render/stop cycles with events drained on X11 and a Wayland fixture or compositor; verify playback/native-window resources return to baseline after clear, reapply acquires a new window, replacement retains the native window/EGL surface identities, and all engine-owned counts return to baseline after engine stop; record initialization-error and multi-output cleanup results.
- [x] 5.2 Capture before/after memory samples every 5 seconds for uninterrupted looping video, lifecycle cycling, and browsing over 1000 previews using the design's warm-up/window settings; record RSS/PSS/anonymous memory, available GPU metrics, cache/resource counts, and fixed workload metadata in the change's verification notes, and investigate growth triggers before claiming the reported symptom is resolved.
- [x] 5.3 Run Rust formatting, workspace Clippy with warnings denied, workspace coverage tests, package-script/manifest validation, and Flutter analysis/coverage tests as specified by `.github/workflows/ci.yml`; verify successful check results and explicitly report any unavailable hardware checks. Before any push, all checks in the CI `quality` job must pass, including required CI coverage uploads.
- [x] 5.4 Run `openspec validate fix-wallpaper-memory-leaks --strict`, update task completion only for work actually verified, and record remaining runtime limitations; verify the completed artifacts and implementation agree without rewriting unrelated pending changes or main specifications.

## Implementation Notes

2026-10-07 scope clarification: clear destroys the wallpaper's native window after playback cleanup; replacement preserves the current window and initialized EGL surface. Proposal, specs, design, and pending tasks consistently reflect these distinct operations. Normal context unbinding is handled by rebinding before cleanup; hypothetical permanent context loss does not block this change.

2026-10-07: Tasks 1.1–1.3 implemented. Four native EGL regressions passed under Xvfb, including direct invalid-handle queries, shared/externally initialized display survival, and three initialization fault checkpoints. The full engine suite passed (25 tests, with X11 enabled; live Wayland enumeration was not enabled). Rust formatting, engine Clippy with warnings denied, and strict OpenSpec validation passed.

2026-10-07: The user explicitly directs continuation of the confirmed lifecycle and cache fixes without making hypothetical permanent GPU-context loss a blocker. Design/task 2.2 now cover normal original-context binding, initialization rollback, and recoverable cleanup errors. Implementation resumed; clear destroys windows and replacement reuses them.

2026-10-07: Session teardown, backend lifecycle cleanup, generation filtering, and bounded previews are implemented. Native X11 tests cover rollback of a NUL-path initialization error, original-context MPV destruction, retry, surface identity preservation, and 100 apply/clear cycles. A real nested Niri compositor passed 100 rendered apply/clear cycles across 20 engine starts/stops; native surface/context counts return to zero on each engine shutdown. Wayland wire fixtures verify destructor ordering, named/fallback output hotplug, duplicate cleanup, pending replacement reuse, and stale configure/closure/frame events. Flutter tests cover budgets, generations, request saturation, failures, and controller invalidation. Controlled 13-minute before/after memory probes are running; task 5.2 remains unmarked until completed and analyzed.

2026-10-07: All six controlled memory probes completed (157 samples each); deterministic cache counts and a follow-up allocator probe distinguish released references/native resources from retained process pages. Verification results and limitations are recorded in verification.md. All local quality checks and strict validation passed; no push or CI coverage upload was attempted. The reported real-GPU uninterrupted symptom is not claimed fully resolved.
