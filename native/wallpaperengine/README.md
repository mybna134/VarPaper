# Scene native component

The source subset comes from linux-wallpaperengine revision
`b016d7d1fdcf4e5fd2f9c9fa420a8aaa07fee02d`. It omits the upstream application,
CLI, desktop drivers and browser. The VarPaper host supplies resources,
output dimensions, OpenGL context and frame times. `upstream-source.json`
records local adaptations and the source inventory; `dependencies.json`
pins dependency archives and SHA-256 checksums. Both sources and their license
notices remain in this directory. These snapshots build without the reference
checkout or network access.

This component is in development. The application currently reports Scene/Web
projects as requiring a renderer. Native fixture success is not evidence that
desktop integration, audio capture, SceneScript or Web support is complete.

System development dependencies: CMake 3.24+, C++20 compiler, Python 3, pkg-config,
GLEW, GLFW3, SDL2, LZ4, FreeType, FFmpeg (avcodec, avformat, avutil, swresample),
libmpv, and an OpenGL 3.3 implementation. Xvfb is used for the native fixture.

```sh
python3 native/wallpaperengine/verify_sources.py
cmake -S native/wallpaperengine -B build/native-scene -DCMAKE_BUILD_TYPE=Release
cmake --build build/native-scene --target varpaper_scene_test -j4
xvfb-run -a ctest --test-dir build/native-scene --output-on-failure
```

`scene.h` defines the C ABI. Handles are bound to the creating thread; calls
require the same OpenGL context current, including destruction. The host keeps
its callback userdata alive until handle destruction. Successful reads transfer
byte ownership temporarily, and the native adapter calls the host release
callback exactly once. Callbacks must contain all Rust panics/C++ exceptions.
The host owns error buffers; native failures are captured and copied into them.
The host pauses frame/tick dispatch when paused and calls `vp_scene_set_audio`
to suspend the actual SDL audio device. Frame time is supplied per instance.

Shader dependencies retain their full compiler and optimizer support. Compiler
warnings in the fixed third-party SPIRV-Tools version are not promoted to errors
because newer GCC versions diagnose its internal timer implementation.

The native fixture additionally loads a locally generated PCM WAV, repeats
sound pause/resume/destruction and fails a second sound after an earlier worker
starts. It checks rejection of unknown/light objects, interrupts an unbounded
SceneScript at initialization, and compares text frames at supplied times 0 and
2 seconds. These checks cover the tested paths only; they do not complete the
Scene feature matrix or audio-spectrum capture. QuickJS has a 32 MiB memory
limit, 512 KiB stack limit and 20 ms execution budget. Scene timers use the
host's animation clock; `Date` retains the system clock.
