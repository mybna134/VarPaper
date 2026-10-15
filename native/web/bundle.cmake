# Shared base-bundle inventory. Chromium runtime files are installed only by
# the application's component manager, never by these CMake install rules.
include_guard(GLOBAL)
set(VARPAPER_WEB_SOURCE_DIR "${CMAKE_CURRENT_LIST_DIR}")
set(VARPAPER_WEB_HOST_NAME "varpaper-web-host")
set(VARPAPER_WEB_MANIFEST "${VARPAPER_WEB_SOURCE_DIR}/runtime.json")
set(VARPAPER_WEB_LICENSES
  "${VARPAPER_WEB_SOURCE_DIR}/JSON-LICENSE"
  "${VARPAPER_WEB_SOURCE_DIR}/CEF-LICENSE"
)

function(varpaper_install_web_host binary)
  # The host executable also runs CEF subprocesses; there is no second helper.
  install(PROGRAMS "${binary}" RENAME "${VARPAPER_WEB_HOST_NAME}"
    DESTINATION libexec COMPONENT Runtime)
  install(FILES "${VARPAPER_WEB_MANIFEST}"
    DESTINATION data COMPONENT Runtime)
  install(FILES ${VARPAPER_WEB_LICENSES}
    DESTINATION data/licenses COMPONENT Runtime)
endfunction()
