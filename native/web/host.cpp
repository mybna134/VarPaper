// VarPaper's CEF owner. No desktop windows or GL contexts live in this process.
#include "include/cef_app.h"
#include "include/cef_browser.h"
#include "include/cef_client.h"
#include "include/cef_parser.h"
#include "include/cef_process_message.h"
#include "include/cef_request_context.h"
#include "include/cef_resource_handler.h"
#include "include/cef_scheme.h"
#include "include/cef_v8.h"
#include "include/wrapper/cef_helpers.h"
#include <algorithm>
#include <array>
#include <cctype>
#include <cerrno>
#include <chrono>
#include <cstdlib>
#include <cstring>
#include <fcntl.h>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <map>
#include <mutex>
#include <nlohmann/json.hpp>
#include <optional>
#include <poll.h>
#include <signal.h>
#include <sys/prctl.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/wait.h>
#include <unistd.h>
#include <utility>

using Json = nlohmann::json;
namespace fs = std::filesystem;
constexpr size_t kMaxFrame = 64 * 1024 * 1024;
constexpr size_t kMaxJson = 65536;
constexpr auto kOrigin = "https://wallpaper.varpaper.local/";

bool io(int fd, void *bytes, size_t size, bool writing) {
  auto *ptr = static_cast<unsigned char *>(bytes);
  while (size) {
    auto n =
        writing ? send(fd, ptr, size, MSG_NOSIGNAL) : recv(fd, ptr, size, 0);
    if (n < 0 && errno == EINTR)
      continue;
    if (n <= 0)
      return false;
    ptr += n;
    size -= n;
  }
  return true;
}
uint32_t little(const std::array<unsigned char, 4> &b) {
  return uint32_t(b[0]) | (uint32_t(b[1]) << 8) | (uint32_t(b[2]) << 16) |
         (uint32_t(b[3]) << 24);
}
bool respond(int fd, Json response,
             const std::vector<unsigned char> &pixels = {}) {
  response["bytes"] = pixels.size();
  response["ipc_version"] = 1;
  auto data = response.dump();
  if (data.size() > kMaxJson || pixels.size() > kMaxFrame)
    return false;
  uint32_t n = data.size();
  std::array<unsigned char, 4> header{
      static_cast<unsigned char>(n), static_cast<unsigned char>(n >> 8),
      static_cast<unsigned char>(n >> 16), static_cast<unsigned char>(n >> 24)};
  return io(fd, header.data(), header.size(), true) &&
         io(fd, data.data(), data.size(), true) &&
         (pixels.empty() || io(fd, const_cast<unsigned char *>(pixels.data()),
                               pixels.size(), true));
}
std::string encode_path(const std::string &path) {
  std::string out;
  for (unsigned char c : path) {
    if (std::isalnum(c) || c == '/' || c == '-' || c == '_' || c == '.')
      out += c;
    else {
      static constexpr char hex[] = "0123456789ABCDEF";
      out += '%';
      out += hex[c >> 4];
      out += hex[c & 15];
    }
  }
  return out;
}
fs::path resource_path(const fs::path &root, const std::string &url) {
  if (!url.starts_with(kOrigin))
    throw std::runtime_error("External request blocked: " + url.substr(0, 256));
  auto encoded = url.substr(std::strlen(kOrigin));
  encoded = encoded.substr(0, encoded.find_first_of("?#"));
  auto name =
      CefURIDecode(encoded, false,
                   static_cast<cef_uri_unescape_rule_t>(
                       UU_SPACES | UU_URL_SPECIAL_CHARS_EXCEPT_PATH_SEPARATORS |
                       UU_PATH_SEPARATORS))
          .ToString();
  if (name.empty())
    name = "index.html";
  if (name.size() > 4096 || name.find('\0') != std::string::npos ||
      name.find('\\') != std::string::npos)
    throw std::runtime_error("Unsafe project resource path");
  fs::path relative(name);
  for (const auto &part : relative)
    if (part == ".." || part == "/")
      throw std::runtime_error("Project resource escapes root");
  auto resolved = fs::canonical(root / relative);
  auto under = resolved.lexically_relative(root);
  if (under.empty() || *under.begin() == ".." || !fs::is_regular_file(resolved))
    throw std::runtime_error("Project resource escapes root or is not a file");
  if (fs::file_size(resolved) > kMaxFrame)
    throw std::runtime_error("Project resource exceeds size budget");
  return resolved;
}
struct PageState {
  std::mutex mutex;
  std::string error;
  void fail(std::string message) {
    std::lock_guard lock(mutex);
    if (error.empty())
      error = std::move(message);
  }
  std::string failure() {
    std::lock_guard lock(mutex);
    return error;
  }
};
// Resolve every component through owned directory descriptors so a rename or
// symlink swap between validation and opening cannot redirect the read.
struct Descriptor {
  int fd;
  explicit Descriptor(int value = -1) : fd(value) {}
  Descriptor(Descriptor &&other) noexcept : fd(std::exchange(other.fd, -1)) {}
  Descriptor(const Descriptor &) = delete;
  ~Descriptor() {
    if (fd >= 0)
      close(fd);
  }
};
Descriptor open_beneath(int base, const fs::path &relative, bool directory) {
  int current = fcntl(base, F_DUPFD_CLOEXEC, 0);
  if (current < 0)
    throw std::system_error(errno, std::generic_category(),
                            "Cannot retain project root");
  Descriptor owner(current);
  auto end = relative.end();
  for (auto it = relative.begin(); it != end; ++it) {
    if (*it == ".")
      continue;
    if (*it == ".." || it->is_absolute())
      throw std::runtime_error("Project resource escapes root");
    auto next = it;
    ++next;
    int flags = O_RDONLY | O_CLOEXEC | O_NOFOLLOW | O_NONBLOCK;
    if (next != end || directory)
      flags = O_PATH | O_DIRECTORY | O_CLOEXEC | O_NOFOLLOW;
    int opened = openat(owner.fd, it->c_str(), flags);
    if (opened < 0)
      throw std::system_error(errno, std::generic_category(),
                              "Project resource open failed/blocked");
    close(owner.fd);
    owner.fd = opened;
  }
  return owner;
}
Descriptor open_root(const fs::path &root) {
  Descriptor filesystem(open("/", O_PATH | O_DIRECTORY | O_CLOEXEC));
  if (filesystem.fd < 0)
    throw std::system_error(errno, std::generic_category(),
                            "Cannot open filesystem root");
  return open_beneath(filesystem.fd, root.relative_path(), true);
}
class Resource final : public CefResourceHandler {
public:
  explicit Resource(fs::path path, int status = 200, int fd = -1,
                    int64_t length = 0)
      : path_(std::move(path)), status_(status), file_(fd), length_(length) {}
  bool Open(CefRefPtr<CefRequest>, bool &handle,
            CefRefPtr<CefCallback>) override {
    handle = true;
    return status_ != 200 || file_.fd >= 0;
  }
  void GetResponseHeaders(CefRefPtr<CefResponse> response, int64_t &length,
                          CefString &) override {
    length = status_ == 200 ? length_ : 0;
    response->SetStatus(status_);
    auto ext = path_.extension().string();
    std::transform(ext.begin(), ext.end(), ext.begin(), [](unsigned char c) {
      return static_cast<char>(std::tolower(c));
    });
    const std::map<std::string, std::string> mime{
        {".html", "text/html"},
        {".htm", "text/html"},
        {".js", "application/javascript"},
        {".mjs", "application/javascript"},
        {".css", "text/css"},
        {".json", "application/json"},
        {".png", "image/png"},
        {".jpg", "image/jpeg"},
        {".jpeg", "image/jpeg"},
        {".svg", "image/svg+xml"},
        {".gif", "image/gif"},
        {".webp", "image/webp"},
        {".woff", "font/woff"},
        {".woff2", "font/woff2"},
        {".mp4", "video/mp4"},
        {".webm", "video/webm"},
        {".mp3", "audio/mpeg"},
        {".ogg", "audio/ogg"},
        {".wav", "audio/wav"}};
    auto it = mime.find(ext);
    response->SetMimeType(it == mime.end() ? "application/octet-stream"
                                           : it->second);
    response->SetHeaderByName("Access-Control-Allow-Origin", kOrigin, true);
    response->SetHeaderByName(
        "Content-Security-Policy",
        "default-src 'self' data: blob:; script-src 'self' data: blob: "
        "'unsafe-inline' 'unsafe-eval'; "
        "style-src 'self' data: blob: 'unsafe-inline'; connect-src 'self'; "
        "object-src 'none'; "
        "base-uri 'self'; form-action 'none'; report-uri "
        "/__varpaper_policy_report",
        true);
  }
  bool Read(void *out, int requested, int &count,
            CefRefPtr<CefResourceReadCallback>) override {
    count = 0;
    if (file_.fd < 0 || requested <= 0 || position_ >= length_)
      return false;
    requested =
        static_cast<int>(std::min<int64_t>(requested, length_ - position_));
    ssize_t amount;
    do {
      amount = read(file_.fd, out, requested);
    } while (amount < 0 && errno == EINTR);
    if (amount > 0) {
      count = static_cast<int>(amount);
      position_ += amount;
    }
    return count > 0;
  }
  void Cancel() override {
    if (file_.fd >= 0)
      close(std::exchange(file_.fd, -1));
  }

private:
  fs::path path_;
  int status_;
  Descriptor file_;
  int64_t length_;
  int64_t position_ = 0;
  IMPLEMENT_REFCOUNTING(Resource);
};
class Factory final : public CefSchemeHandlerFactory {
public:
  Factory(fs::path root, std::shared_ptr<PageState> state)
      : root_(std::move(root)), state_(std::move(state)),
        directory_(open_root(root_)) {}
  CefRefPtr<CefResourceHandler> Create(CefRefPtr<CefBrowser>,
                                       CefRefPtr<CefFrame>, const CefString &,
                                       CefRefPtr<CefRequest> request) override {
    if (request->GetMethod() == "POST" &&
        request->GetURL() ==
            std::string(kOrigin) + "__varpaper_policy_report") {
      state_->fail("External request blocked by page policy");
      return new Resource({}, 204);
    }
    try {
      auto path = resource_path(root_, request->GetURL().ToString());
      auto file =
          open_beneath(directory_.fd, path.lexically_relative(root_), false);
      struct stat info{};
      if (fstat(file.fd, &info) != 0 || !S_ISREG(info.st_mode) ||
          info.st_size < 0 || uint64_t(info.st_size) > kMaxFrame)
        throw std::runtime_error("Invalid/over-budget project resource");
      auto result =
          CefRefPtr<Resource>(new Resource(path, 200, file.fd, info.st_size));
      file.fd = -1;
      return result;
    } catch (const fs::filesystem_error &e) {
      if (e.code() == std::errc::no_such_file_or_directory)
        return new Resource({}, 404);
      state_->fail(e.what());
      return new Resource({}, 403);
    } catch (const std::exception &e) {
      state_->fail(e.what());
      return new Resource({}, 403);
    }
  }

private:
  fs::path root_;
  std::shared_ptr<PageState> state_;
  Descriptor directory_;
  IMPLEMENT_REFCOUNTING(Factory);
};
class Page final : public CefClient,
                   public CefRenderHandler,
                   public CefLifeSpanHandler,
                   public CefLoadHandler,
                   public CefRequestHandler,
                   public CefResourceRequestHandler,
                   public CefDownloadHandler,
                   public CefDisplayHandler,
                   public CefJSDialogHandler,
                   public CefDialogHandler,
                   public CefPermissionHandler {
public:
  Page(fs::path root, std::string entry, int width, int height,
       uint64_t generation)
      : root(std::move(root)), entry(std::move(entry)), width(width),
        height(height), generation(generation) {}
  CefRefPtr<CefRenderHandler> GetRenderHandler() override { return this; }
  CefRefPtr<CefLifeSpanHandler> GetLifeSpanHandler() override { return this; }
  CefRefPtr<CefLoadHandler> GetLoadHandler() override { return this; }
  CefRefPtr<CefRequestHandler> GetRequestHandler() override { return this; }
  CefRefPtr<CefDownloadHandler> GetDownloadHandler() override { return this; }
  CefRefPtr<CefDisplayHandler> GetDisplayHandler() override { return this; }
  CefRefPtr<CefJSDialogHandler> GetJSDialogHandler() override { return this; }
  CefRefPtr<CefDialogHandler> GetDialogHandler() override { return this; }
  CefRefPtr<CefPermissionHandler> GetPermissionHandler() override {
    return this;
  }
  bool OnJSDialog(CefRefPtr<CefBrowser>, const CefString &, JSDialogType,
                  const CefString &, const CefString &,
                  CefRefPtr<CefJSDialogCallback> callback, bool &) override {
    state->fail("Wallpaper dialogs blocked");
    callback->Continue(false, "");
    return true;
  }
  bool OnBeforeUnloadDialog(CefRefPtr<CefBrowser>, const CefString &, bool,
                            CefRefPtr<CefJSDialogCallback> callback) override {
    callback->Continue(true, "");
    return true;
  }
  bool OnFileDialog(CefRefPtr<CefBrowser>, FileDialogMode, const CefString &,
                    const CefString &, const std::vector<CefString> &,
                    const std::vector<CefString> &,
                    const std::vector<CefString> &,
                    CefRefPtr<CefFileDialogCallback> callback) override {
    state->fail("Wallpaper file picker blocked");
    callback->Cancel();
    return true;
  }
  bool OnRequestMediaAccessPermission(
      CefRefPtr<CefBrowser>, CefRefPtr<CefFrame>, const CefString &, uint32_t,
      CefRefPtr<CefMediaAccessCallback> callback) override {
    state->fail("Wallpaper device access blocked");
    callback->Continue(0);
    return true;
  }
  bool OnShowPermissionPrompt(
      CefRefPtr<CefBrowser>, uint64_t, const CefString &, uint32_t,
      CefRefPtr<CefPermissionPromptCallback> callback) override {
    state->fail("Wallpaper permission request blocked");
    callback->Continue(CEF_PERMISSION_RESULT_DENY);
    return true;
  }
  bool OnConsoleMessage(CefRefPtr<CefBrowser>, cef_log_severity_t level,
                        const CefString &message, const CefString &,
                        int) override {
    auto text = message.ToString();
    if (level >= LOGSEVERITY_ERROR &&
        (text.find("file://") != std::string::npos ||
         text.find("Content Security Policy") != std::string::npos))
      state->fail("External request blocked: " + text.substr(0, 1024));
    return true;
  }
  bool CanDownload(CefRefPtr<CefBrowser>, const CefString &,
                   const CefString &) override {
    state->fail("Wallpaper downloads blocked");
    return false;
  }
  CefRefPtr<CefResourceRequestHandler>
  GetResourceRequestHandler(CefRefPtr<CefBrowser>, CefRefPtr<CefFrame>,
                            CefRefPtr<CefRequest>, bool, bool,
                            const CefString &, bool &disable) override {
    disable = false;
    return this;
  }
  ReturnValue OnBeforeResourceLoad(CefRefPtr<CefBrowser>, CefRefPtr<CefFrame>,
                                   CefRefPtr<CefRequest> request,
                                   CefRefPtr<CefCallback>) override {
    auto url = request->GetURL().ToString();
    if (url.starts_with(kOrigin) || url.starts_with("data:") ||
        url.starts_with("blob:"))
      return RV_CONTINUE;
    state->fail("External request blocked: " + url.substr(0, 256));
    return RV_CANCEL;
  }
  bool OnBeforeBrowse(CefRefPtr<CefBrowser>, CefRefPtr<CefFrame>,
                      CefRefPtr<CefRequest> request, bool, bool) override {
    auto url = request->GetURL().ToString();
    if (url.starts_with(kOrigin))
      return false;
    state->fail("Navigation blocked: " + url.substr(0, 256));
    return true;
  }
  bool OnProcessMessageReceived(CefRefPtr<CefBrowser>, CefRefPtr<CefFrame>,
                                CefProcessId source,
                                CefRefPtr<CefProcessMessage> message) override {
    if (source != PID_RENDERER ||
        message->GetName() != "varpaper-blocked-network")
      return false;
    state->fail(
        "External connection API blocked: " +
        message->GetArgumentList()->GetString(0).ToString().substr(0, 128));
    return true;
  }
  void OnProtocolExecution(CefRefPtr<CefBrowser>, CefRefPtr<CefFrame>,
                           CefRefPtr<CefRequest>, bool &allow) override {
    state->fail("External protocol blocked");
    allow = false;
  }
  bool OnBeforePopup(CefRefPtr<CefBrowser>, CefRefPtr<CefFrame>, int,
                     const CefString &, const CefString &,
                     CefLifeSpanHandler::WindowOpenDisposition, bool,
                     const CefPopupFeatures &, CefWindowInfo &,
                     CefRefPtr<CefClient> &, CefBrowserSettings &,
                     CefRefPtr<CefDictionaryValue> &, bool *) override {
    return true;
  }
  void OnRenderProcessTerminated(CefRefPtr<CefBrowser>, TerminationStatus, int,
                                 const CefString &message) override {
    state->fail("Browser renderer exited: " + message.ToString());
  }
  void GetViewRect(CefRefPtr<CefBrowser>, CefRect &rect) override {
    rect = CefRect(0, 0, width, height);
  }
  void OnPaint(CefRefPtr<CefBrowser>, PaintElementType type, const RectList &,
               const void *buffer, int w, int h) override {
    if (type != PET_VIEW || !loaded || w != width || h != height || w <= 0 ||
        h <= 0 || uint64_t(w) * h * 4 > kMaxFrame)
      return;
    auto *ptr = static_cast<const unsigned char *>(buffer);
    pixels.assign(ptr, ptr + size_t(w) * h * 4);
    ++sequence;
  }
  void OnAfterCreated(CefRefPtr<CefBrowser> value) override {
    browser = value;
    browser->GetHost()->SetAudioMuted(muted);
    if (closing)
      browser->GetHost()->CloseBrowser(true);
  }
  void OnBeforeClose(CefRefPtr<CefBrowser>) override {
    browser = nullptr;
    closed = true;
    loaded = false;
  }
  void OnLoadEnd(CefRefPtr<CefBrowser> value, CefRefPtr<CefFrame> frame,
                 int status) override {
    if (!frame->IsMain())
      return;
    if (status >= 400) {
      state->fail("Web entry failed with HTTP " + std::to_string(status));
      return;
    }
    loaded = true;
    value->GetHost()->Invalidate(PET_VIEW);
  }
  void OnLoadError(CefRefPtr<CefBrowser>, CefRefPtr<CefFrame> frame,
                   ErrorCode code, const CefString &text,
                   const CefString &) override {
    if (frame->IsMain() && code != ERR_ABORTED)
      state->fail("Web entry load failed: " + text.ToString());
  }
  void open() {
    closed = false;
    loaded = false;
    closing = false;
    CefRequestContextSettings context_settings;
    context = CefRequestContext::CreateContext(context_settings, nullptr);
    context->RegisterSchemeHandlerFactory("https", "wallpaper.varpaper.local",
                                          new Factory(root, state));
    CefWindowInfo window;
    window.SetAsWindowless(0);
    CefBrowserSettings settings;
    settings.windowless_frame_rate = 60;
    if (!CefBrowserHost::CreateBrowser(
            window, this, std::string(kOrigin) + encode_path(entry), settings,
            nullptr, context)) {
      closed = true;
      throw std::runtime_error("Cannot create wallpaper browser");
    }
  }
  void audio(bool mute, double volume) {
    muted = mute;
    if (browser) {
      browser->GetHost()->SetAudioMuted(mute);
      browser->GetMainFrame()->ExecuteJavaScript(
          "document.querySelectorAll('audio,video').forEach(e=>e.volume=" +
              std::to_string(std::clamp(volume, 0.0, 1.0)) + ");",
          kOrigin, 0);
    }
  }
  fs::path root;
  std::string entry;
  int width, height;
  uint64_t generation, sequence = 0;
  bool closed = false, loaded = false, muted = true, closing = false;
  std::vector<unsigned char> pixels;
  std::shared_ptr<PageState> state = std::make_shared<PageState>();
  CefRefPtr<CefBrowser> browser;
  CefRefPtr<CefRequestContext> context;

private:
  IMPLEMENT_REFCOUNTING(Page);
};
class BlockNetwork final : public CefV8Handler {
  bool Execute(const CefString &name, CefRefPtr<CefV8Value>,
               const CefV8ValueList &, CefRefPtr<CefV8Value> &,
               CefString &exception) override {
    auto message = CefProcessMessage::Create("varpaper-blocked-network");
    message->GetArgumentList()->SetString(0, name);
    CefV8Context::GetCurrentContext()->GetFrame()->SendProcessMessage(
        PID_BROWSER, message);
    exception = "External connections are blocked for Web wallpapers";
    return true;
  }
  IMPLEMENT_REFCOUNTING(BlockNetwork);
};
class App final : public CefApp,
                  public CefBrowserProcessHandler,
                  public CefRenderProcessHandler {
public:
  bool ready = false;
  CefRefPtr<CefBrowserProcessHandler> GetBrowserProcessHandler() override {
    return this;
  }
  CefRefPtr<CefRenderProcessHandler> GetRenderProcessHandler() override {
    return this;
  }
  void OnContextInitialized() override { ready = true; }
  void OnContextCreated(CefRefPtr<CefBrowser>, CefRefPtr<CefFrame>,
                        CefRefPtr<CefV8Context> context) override {
    auto global = context->GetGlobal();
    CefRefPtr<BlockNetwork> handler = new BlockNetwork;
    for (const char *name : {"WebSocket", "WebTransport", "RTCPeerConnection",
                             "webkitRTCPeerConnection"}) {
      global->SetValue(name, CefV8Value::CreateFunction(name, handler),
                       static_cast<CefV8Value::PropertyAttribute>(
                           V8_PROPERTY_ATTRIBUTE_READONLY |
                           V8_PROPERTY_ATTRIBUTE_DONTDELETE));
    }
  }

private:
  void
  OnBeforeCommandLineProcessing(const CefString &,
                                CefRefPtr<CefCommandLine> command) override {
    command->AppendSwitch("disable-background-networking");
    command->AppendSwitch("disable-component-update");
    command->AppendSwitch("disable-breakpad");
    command->AppendSwitch("disable-crash-reporter");
    command->AppendSwitch("disable-extensions");
    command->AppendSwitch("disable-quic");
    command->AppendSwitch("dns-prefetch-disable");
    command->AppendSwitchWithValue("host-resolver-rules", "MAP * ~NOTFOUND");
    command->AppendSwitchWithValue("force-webrtc-ip-handling-policy",
                                   "disable_non_proxied_udp");
    command->AppendSwitch("disable-print-preview");
    command->AppendSwitchWithValue("autoplay-policy",
                                   "no-user-gesture-required");
    if (std::getenv("VARPAPER_TEST_CEF_SOFTWARE")) {
      command->AppendSwitchWithValue("use-gl", "angle");
      command->AppendSwitchWithValue("use-angle", "swiftshader");
      command->AppendSwitch("enable-unsafe-swiftshader");
    }
  }
  IMPLEMENT_REFCOUNTING(App);
};
void close_page(CefRefPtr<Page> page) {
  page->closing = true;
  if (page->browser)
    page->browser->GetHost()->CloseBrowser(true);
  auto deadline = std::chrono::steady_clock::now() + std::chrono::seconds(5);
  while (!page->closed && std::chrono::steady_clock::now() < deadline) {
    CefDoMessageLoopWork();
    usleep(1000);
  }
  if (!page->closed)
    throw std::runtime_error("Browser did not acknowledge shutdown");
  page->context = nullptr;
}
int main(int argc, char **argv) {
  signal(SIGPIPE, SIG_IGN);
  prctl(PR_SET_PDEATHSIG, SIGTERM);
  CefMainArgs args(argc, argv);
  CefRefPtr<App> app = new App;
  int subprocess = CefExecuteProcess(args, app, nullptr);
  if (subprocess >= 0)
    return subprocess;
  const char *runtime_env = std::getenv("VARPAPER_CEF_RUNTIME");
  if (!runtime_env)
    return 2;
  int fd = -1;
  for (int i = 1; i < argc; ++i)
    if (std::string(argv[i]).starts_with("--ipc-fd="))
      fd = std::stoi(std::string(argv[i]).substr(9));
  if (fd < 0 || getpgrp() != getpid())
    return 2;
  // Adopt orphaned zygote/worker descendants and reap them before exiting.
  if (prctl(PR_SET_CHILD_SUBREAPER, 1) != 0)
    return 2;
  fcntl(fd, F_SETFD, FD_CLOEXEC);
  timeval timeout{5, 0};
  setsockopt(fd, SOL_SOCKET, SO_RCVTIMEO, &timeout, sizeof(timeout));
  setsockopt(fd, SOL_SOCKET, SO_SNDTIMEO, &timeout, sizeof(timeout));
  CefSettings settings;
  settings.windowless_rendering_enabled = true;
  settings.no_sandbox = false;
  CefString(&settings.resources_dir_path) = runtime_env;
  CefString(&settings.locales_dir_path) =
      (fs::path(runtime_env) / "locales").string();
  CefString(&settings.browser_subprocess_path) =
      fs::canonical("/proc/self/exe").string();
  const auto cache = fs::path(runtime_env).parent_path() / "browser-cache" /
                     std::to_string(getpid());
  CefString(&settings.root_cache_path) = cache.string();
  settings.log_severity = LOGSEVERITY_DISABLE;
  if (!CefInitialize(args, settings, app, nullptr))
    return 3;
  auto initialized_deadline =
      std::chrono::steady_clock::now() + std::chrono::seconds(5);
  while (!app->ready &&
         std::chrono::steady_clock::now() < initialized_deadline) {
    CefDoMessageLoopWork();
    usleep(1000);
  }
  if (!app->ready) {
    CefShutdown();
    return 3;
  }
  std::map<uint64_t, CefRefPtr<Page>> pages;
  bool alive = true;
  while (alive) {
    CefDoMessageLoopWork();
    pollfd wait{fd, POLLIN, 0};
    auto ready = poll(&wait, 1, 2);
    if (ready < 0) {
      if (errno == EINTR)
        continue;
      break;
    }
    if (!ready)
      continue;
    std::array<unsigned char, 4> header{};
    if (!io(fd, header.data(), header.size(), false))
      break;
    auto size = little(header);
    if (!size || size > kMaxJson)
      break;
    std::string data(size, '\0');
    if (!io(fd, data.data(), size, false))
      break;
    Json response;
    std::vector<unsigned char> pixels;
    try {
      auto command = Json::parse(data);
      auto request = command.at("request").get<uint64_t>();
      auto id = command.value("id", uint64_t(0));
      auto generation = command.value("generation", uint64_t(0));
      response = {{"request", request},
                  {"id", id},
                  {"generation", generation},
                  {"ok", true}};
      auto op = command.at("op").get<std::string>();
      if (op == "shutdown") {
        alive = false;
      } else if (op == "create") {
        if (pages.size() >= 16 || pages.contains(id))
          throw std::runtime_error("Browser count/id budget exceeded");
        auto root = fs::canonical(command.at("root").get<std::string>());
        auto entry = command.at("entry").get<std::string>();
        resource_path(root, std::string(kOrigin) + encode_path(entry));
        int w = command.at("width"), h = command.at("height");
        if (w <= 0 || h <= 0 || uint64_t(w) * h * 4 > kMaxFrame)
          throw std::runtime_error("Viewport exceeds frame budget");
        CefRefPtr<Page> page = new Page(root, entry, w, h, generation);
        page->open();
        pages[id] = page;
      } else {
        auto it = pages.find(id);
        if (it == pages.end() || it->second->generation != generation)
          throw std::runtime_error("Stale browser generation");
        auto page = it->second;
        auto failure = page->state->failure();
        if (!failure.empty() && op != "close")
          throw std::runtime_error(failure);
        if (op == "frame") {
          int w = command.at("width"), h = command.at("height");
          if (w <= 0 || h <= 0 || uint64_t(w) * h * 4 > kMaxFrame)
            throw std::runtime_error("Viewport exceeds frame budget");
          if (w != page->width || h != page->height) {
            page->width = w;
            page->height = h;
            page->pixels.clear();
            if (page->browser)
              page->browser->GetHost()->WasResized();
          }
          response["width"] = w;
          response["height"] = h;
          response["sequence"] = page->sequence;
          if (command.value("sequence", uint64_t(0)) != page->sequence)
            pixels = page->pixels;
        } else if (op == "pause") {
          close_page(page);
        } else if (op == "resume") {
          if (page->closed)
            page->open();
        } else if (op == "audio") {
          page->audio(command.at("mute"), command.at("volume"));
        } else if (op == "close") {
          if (!page->closed)
            close_page(page);
          pages.erase(it);
        } else
          throw std::runtime_error("Unknown Web host command");
      }
    } catch (const std::exception &e) {
      response["ok"] = false;
      response["error"] = std::string(e.what()).substr(0, 4096);
    }
    if (!respond(fd, response, pixels))
      break;
  }
  try {
    for (auto &[id, page] : pages)
      if (!page->closed)
        close_page(page);
  } catch (const std::exception &e) {
    std::cerr << e.what() << '\n';
    _exit(4);
  }
  pages.clear();
  app = nullptr;
  CefShutdown();
  // CEF may leave its zygote alive after shutting down the last browser. As a
  // subreaper, terminate and reap our direct children; orphaned descendants
  // become direct children on the next pass. Never signal the host itself.
  auto child_deadline =
      std::chrono::steady_clock::now() + std::chrono::seconds(4);
  for (;;) {
    int result = waitpid(-1, nullptr, WNOHANG);
    if (result > 0)
      continue;
    if (result < 0 && errno == ECHILD)
      break;
    if (result < 0 && errno != EINTR)
      break;
    std::ifstream children("/proc/self/task/" + std::to_string(getpid()) +
                           "/children");
    pid_t child;
    while (children >> child)
      if (child > 0)
        kill(child, SIGKILL);
    if (std::chrono::steady_clock::now() >= child_deadline) {
      // This process owns a private process group; stop all remaining children.
      kill(0, SIGKILL);
      _exit(4);
    }
    usleep(1000);
  }
  std::error_code cleanup_error;
  fs::remove_all(cache, cleanup_error);
  close(fd);
  return 0;
}
