#include "scene.h"
#include <GL/glew.h>
#include <algorithm>
#include <atomic>
#include <cmath>
#include <cstring>
#include <ctime>
#include <sstream>
#include <mutex>
#include <stdexcept>
#include <thread>
#include "WallpaperEngine/Application/WallpaperApplication.h"
#include "WallpaperEngine/Audio/Drivers/SDLAudioDriver.h"
#include "WallpaperEngine/Data/Parsers/ProjectParser.h"
#include "WallpaperEngine/Render/Wallpapers/CScene.h"

using namespace WallpaperEngine;
float g_Time = 0, g_TimeLast = 0, g_Daytime = 0;
double g_AnimationTime = 0;
Application::ApplicationContext::ApplicationContext(int argc, char **argv) : m_argc(argc), m_argv(argv) {}
namespace {
std::atomic_size_t live_count{0};
// Upstream shader/script registries and frame uniforms are process globals.
// Serialize native operations while each handle keeps its render-thread owner.
std::mutex native_mutex;
void dimensions(uint32_t w, uint32_t h) {
    if (w < 8 || h < 8 || w > 16384 || h > 16384 || uint64_t(w) * h > 33554432)
        throw std::runtime_error("Scene dimensions exceed framebuffer budget");
}
void diagnostic(char *out, size_t capacity, const char *message) noexcept {
    if (!out || !capacity) return;
    const auto length = std::min(capacity - 1, std::strlen(message));
    std::memcpy(out, message, length); out[length] = 0;
}
template<class F> int boundary(char *out, size_t capacity, F &&operation) noexcept {
    diagnostic(out, capacity, "");
    try { std::lock_guard lock(native_mutex); operation(); return 0; }
    catch (const std::exception &e) { diagnostic(out, capacity, e.what()); }
    catch (...) { diagnostic(out, capacity, "Unknown native Scene failure"); }
    return -1;
}
struct HostAdapter final : FileSystem::Adapters::Adapter {
    vp_scene_host host;
    explicit HostAdapter(vp_scene_host h) : host(h) {}
    bool exists(const std::filesystem::path &path) const override {
        return host.exists(host.userdata, path.generic_string().c_str()) == 1;
    }
    Data::Utils::ReadStreamSharedPtr open(const std::filesystem::path &path) const override {
        const uint8_t *bytes = nullptr; size_t length = 0;
        const int status = host.read(host.userdata, path.generic_string().c_str(), &bytes, &length);
        struct Buffer {
            vp_scene_host host; const uint8_t *bytes; size_t length;
            ~Buffer() { if (bytes) host.release(host.userdata, bytes, length); }
        } buffer{host, bytes, length};
        if (status || (!bytes && length) || length > 256 * 1024 * 1024)
            throw std::runtime_error("Cannot read bounded Scene resource: " + path.generic_string());
        return std::make_shared<std::istringstream>(length ? std::string(reinterpret_cast<const char*>(bytes), length) : std::string{});
    }
    std::filesystem::path physicalPath(const std::filesystem::path &) const override {
        throw std::runtime_error("Scene resource requires an unavailable physical path");
    }
};
struct HostFactory final : FileSystem::Adapters::Factory {
    vp_scene_host host;
    explicit HostFactory(vp_scene_host h) : host(h) {}
    bool handlesMountpoint(const std::filesystem::path &p) const override { return p == "varpaper-host"; }
    FileSystem::Adapters::AdapterSharedPtr create(const std::filesystem::path &) const override {
        return std::make_shared<HostAdapter>(host);
    }
};
struct Mouse final : Input::MouseInput {
    glm::dvec2 point{0, 0};
    void update() override {}
    glm::dvec2 position() const override { return point; }
    Input::MouseClickStatus leftClick() const override { return Input::Released; }
    Input::MouseClickStatus rightClick() const override { return Input::Released; }
};
struct HostOutput final : Render::Drivers::Output::Output {
    HostOutput(Application::ApplicationContext &c, Render::Drivers::VideoDriver &d) : Output(c, d) {}
    void size(uint32_t w, uint32_t h) { m_fullWidth = w; m_fullHeight = h; }
    void reset() override {}
    bool renderVFlip() const override { return false; }
    bool renderMultiple() const override { return false; }
    bool haveImageBuffer() const override { return false; }
    void *getImageBuffer() const override { return nullptr; }
    uint32_t getImageBufferSize() const override { return 0; }
    void updateRender() const override {}
};
struct Driver final : Render::Drivers::VideoDriver {
    vp_scene_host host;
    HostOutput output;
    uint32_t frames = 0;
    Driver(Application::WallpaperApplication &app, Mouse &mouse, vp_scene_host h)
        : VideoDriver(app, mouse), host(h), output(app.getContext(), *this) {}
    Render::Drivers::Output::Output &getOutput() override { return output; }
    float getRenderTime() const override { return g_Time; }
    bool closeRequested() override { return false; }
    void resizeWindow(glm::ivec2 s) override { output.size(s.x, s.y); }
    void resizeWindow(glm::ivec4 s) override { output.size(s.z, s.w); }
    void showWindow() override {}
    void hideWindow() override {}
    glm::ivec2 getFramebufferSize() const override { return {output.getFullWidth(), output.getFullHeight()}; }
    uint32_t getFrameCounter() const override { return frames; }
    void *getProcAddress(const char *name) const override { return host.gl_proc(host.userdata, name); }
    void dispatchEventQueue() override {}
};
struct NoMedia final : Media::MediaSource {
    NoMedia() : MediaSource(std::chrono::milliseconds(1000)) { m_mediaInfo = {}; }
    void performUpdate() override {}
};
}
struct vp_scene {
    std::thread::id thread = std::this_thread::get_id();
    static Application::ApplicationContext make_context() {
        Application::ApplicationContext context{0, nullptr};
        context.state.general.keepRunning = true;
        context.state.audio.enabled = true;
        context.state.audio.volume = 0;
        context.state.mouse.enabled = true;
        return context;
    }
    Application::ApplicationContext context = make_context();
    Application::WallpaperApplication app{context};
    Mouse mouse;
    Driver driver;
    NoMedia media;
    Render::Drivers::Detectors::FullScreenDetector fullscreen{context};
    Audio::Drivers::Detectors::AudioPlayingDetector detector{context, fullscreen};
    Audio::Drivers::Recorders::PlaybackRecorder recorder;
    Audio::Drivers::SDLAudioDriver audio_driver{context, detector, recorder};
    Audio::AudioContext audio{audio_driver};
    Render::RenderContext render{driver, app, media};
    std::unique_ptr<Render::Wallpapers::CScene> scene;
    explicit vp_scene(vp_scene_host h) : driver(app, mouse, h) { ++live_count; }
    ~vp_scene() { --live_count; }
    void check() const {
        if (thread != std::this_thread::get_id()) throw std::runtime_error("Scene called from another thread");
        if (!glGetString(GL_VERSION)) throw std::runtime_error("Scene requires its OpenGL context current");
    }
};
extern "C" int vp_scene_create(const vp_scene_host *host, const vp_scene_config *cfg,
                               vp_scene **out, char *error, size_t capacity) {
    if (out) *out = nullptr;
    return boundary(error, capacity, [&] {
        if (!out || !host || !cfg || !cfg->manifest_json || !host->read || !host->release || !host->exists || !host->gl_proc)
            throw std::invalid_argument("Invalid Scene host/configuration");
        if (strnlen(cfg->manifest_json, 1024 * 1024 + 1) > 1024 * 1024) throw std::runtime_error("Scene manifest exceeds budget");
        if (!std::isfinite(cfg->volume)) throw std::invalid_argument("Invalid Scene volume");
        dimensions(cfg->width, cfg->height);
        if (!glGetString(GL_VERSION)) throw std::runtime_error("Scene requires a current OpenGL context");
        glewExperimental = GL_TRUE;
        const auto glew_status = glewInit();
        // EGL has no GLX display; GLEW nevertheless loads the desktop GL entry points.
        if (glew_status != GLEW_OK && glew_status != GLEW_ERROR_NO_GLX_DISPLAY)
            throw std::runtime_error(reinterpret_cast<const char*>(glewGetErrorString(glew_status)));
        GLint major = 0, minor = 0;
        glGetIntegerv(GL_MAJOR_VERSION, &major);
        glGetIntegerv(GL_MINOR_VERSION, &minor);
        if (major < 3 || (major == 3 && minor < 3) || !GLEW_VERSION_3_3)
            throw std::runtime_error("Scene requires OpenGL 3.3; target reports " + std::to_string(major) + "." + std::to_string(minor));
        auto instance = std::make_unique<vp_scene>(*host);
        instance->driver.output.size(cfg->width, cfg->height);
        instance->mouse.point = {cfg->width / 2.0, cfg->height / 2.0};
        auto fs = std::make_unique<FileSystem::Container>();
        fs->registerAdapterFactory(std::make_unique<HostFactory>(*host));
        fs->mount("varpaper-host", "/");
        auto project = Data::Parsers::ProjectParser::parse(Data::JSON::JSON(nlohmann::json::parse(cfg->manifest_json)),
            std::make_unique<Assets::AssetLocator>(std::move(fs)));
        if (project->type != Data::Model::Project::Type_Scene) throw std::runtime_error("Expected a Scene project");
        const auto scaling = static_cast<Render::WallpaperState::TextureUVsScaling>(cfg->scaling);
        if (cfg->scaling > 3) throw std::runtime_error("Invalid Scene scaling mode");
        instance->app.projects.emplace("scene", std::move(project));
        g_Time = g_TimeLast = 0; g_AnimationTime = 0;
        instance->scene = std::make_unique<Render::Wallpapers::CScene>(*instance->app.projects.at("scene")->wallpaper, instance->render,
            instance->audio, scaling, Data::Assets::TextureFlags_ClampUVs);
        instance->audio_driver.setPlayback(false, cfg->muted, cfg->volume);
        *out = instance.release();
    });
}
extern "C" int vp_scene_render(vp_scene *s, uint32_t w, uint32_t h, uint32_t fbo,
    double time, double delta, const float spectrum[64], char *error, size_t capacity) {
    return boundary(error, capacity, [&] {
        if (!s) throw std::invalid_argument("Missing Scene");
        s->check(); dimensions(w, h);
        if (!std::isfinite(time) || !std::isfinite(delta) || time < 0 || delta < 0 || delta > 1)
            throw std::invalid_argument("Invalid Scene clock");
        s->driver.output.size(w, h);
        g_AnimationTime = time;
        g_Time = time; g_TimeLast = time - delta;
        const auto now = std::time(nullptr);
        std::tm local{};
        if (localtime_r(&now, &local)) g_Daytime = float(local.tm_hour * 3600 + local.tm_min * 60 + local.tm_sec) / 86400.0f;
        for (int i = 0; i < 64; ++i) s->recorder.audio64[i] = spectrum && std::isfinite(spectrum[i]) ? std::clamp(spectrum[i], 0.0f, 1.0f) : 0;
        for (int i = 0; i < 32; ++i) s->recorder.audio32[i] = (s->recorder.audio64[i*2] + s->recorder.audio64[i*2+1]) / 2;
        for (int i = 0; i < 16; ++i) s->recorder.audio16[i] = (s->recorder.audio32[i*2] + s->recorder.audio32[i*2+1]) / 2;
        ++s->driver.frames;
        s->scene->setDestinationFramebuffer(fbo);
        const auto &color = s->scene->getScene().colors.clear->value->getVec3();
        glClearColor(color.r, color.g, color.b, 1);
        s->scene->render({0, 0, w, h}, false);
    });
}
extern "C" int vp_scene_set_audio(vp_scene *s, int paused, int muted, float volume, char *error, size_t capacity) {
    return boundary(error, capacity, [&] {
        if (!s) throw std::invalid_argument("Missing Scene");
        s->check();
        if (!std::isfinite(volume)) throw std::invalid_argument("Invalid Scene volume");
        s->audio_driver.setPlayback(paused, muted, volume);
    });
}
extern "C" int vp_scene_destroy(vp_scene *s, char *error, size_t capacity) {
    return boundary(error, capacity, [&] { if (s) { s->check(); delete s; } });
}
extern "C" size_t vp_scene_live_count(void) { return live_count.load(); }

extern "C" uint32_t vp_scene_abi_version(void) { return 1; }
