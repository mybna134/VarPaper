cmake_minimum_required(VERSION 3.24)
project(varpaper_scene_core LANGUAGES C CXX)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)
set(CMAKE_POLICY_VERSION_MINIMUM 3.5)

include(FetchContent)
file(READ "${CMAKE_CURRENT_SOURCE_DIR}/dependencies.json" VP_DEPENDENCIES)
foreach(name glslang spirv quickjs json stb kissfft glm spirvtools spirvheaders)
  string(JSON archive GET "${VP_DEPENDENCIES}" ${name} archive)
  string(JSON digest GET "${VP_DEPENDENCIES}" ${name} sha256)
  FetchContent_Declare(${name}
    URL "${CMAKE_CURRENT_SOURCE_DIR}/vendor/${archive}"
    URL_HASH "SHA256=${digest}"
    DOWNLOAD_EXTRACT_TIMESTAMP TRUE
    SOURCE_SUBDIR "varpaper-extract-only")
  # Populate snapshots only; select the required dependency targets below.
  FetchContent_MakeAvailable(${name})
endforeach()

set(ENABLE_GLSLANG_BINARIES OFF CACHE BOOL "" FORCE)
set(ENABLE_OPT ON CACHE BOOL "" FORCE)
set(ENABLE_SPVREMAPPER OFF CACHE BOOL "" FORCE)
set(BUILD_TESTING OFF CACHE BOOL "" FORCE)
set(SPIRV_CROSS_CLI OFF CACHE BOOL "" FORCE)
set(SPIRV_CROSS_ENABLE_TESTS OFF CACHE BOOL "" FORCE)
set(QJS_BUILD_EXAMPLES OFF CACHE BOOL "" FORCE)
set(ENABLE_SPIRV ON CACHE BOOL "" FORCE)
set(SPIRV_WERROR OFF CACHE BOOL "" FORCE)
set(SPIRV_SKIP_TESTS ON CACHE BOOL "" FORCE)
set(SPIRV_SKIP_EXECUTABLES ON CACHE BOOL "" FORCE)
set(SPIRV-Headers_SOURCE_DIR "${spirvheaders_SOURCE_DIR}" CACHE PATH "" FORCE)
add_subdirectory("${spirvtools_SOURCE_DIR}" "${spirvtools_BINARY_DIR}" EXCLUDE_FROM_ALL)
add_subdirectory("${glslang_SOURCE_DIR}" "${glslang_BINARY_DIR}" EXCLUDE_FROM_ALL)
add_subdirectory("${spirv_SOURCE_DIR}" "${spirv_BINARY_DIR}" EXCLUDE_FROM_ALL)
add_subdirectory("${quickjs_SOURCE_DIR}" "${quickjs_BINARY_DIR}" EXCLUDE_FROM_ALL)

find_package(PkgConfig REQUIRED)
pkg_check_modules(SCENE_SYSTEM REQUIRED IMPORTED_TARGET
  glew glfw3 sdl2 liblz4 freetype2 libavcodec libavformat libavutil libswresample mpv)

set(core "${CMAKE_CURRENT_SOURCE_DIR}/upstream/src/WallpaperEngine")
file(READ "${core}/Scripting/resources/builtins.js" WPENGINE_SCRIPT_BUILTINS)
configure_file("${core}/Scripting/resources/Builtins.generated.h.in"
  "${CMAKE_CURRENT_BINARY_DIR}/generated/WallpaperEngine/Scripting/Builtins.generated.h" @ONLY)

file(GLOB_RECURSE SCENE_SOURCES CONFIGURE_DEPENDS "${core}/*.cpp")
add_library(varpaper_scene_core SHARED ${SCENE_SOURCES} scene.cpp)
target_include_directories(varpaper_scene_core PUBLIC
  "${CMAKE_CURRENT_SOURCE_DIR}"
  "${CMAKE_CURRENT_SOURCE_DIR}/compat"
  "${CMAKE_CURRENT_SOURCE_DIR}/upstream/src"
  "${CMAKE_CURRENT_BINARY_DIR}/generated"
  "${glm_SOURCE_DIR}" "${json_SOURCE_DIR}/include" "${stb_SOURCE_DIR}"
  "${quickjs_SOURCE_DIR}" "${glslang_SOURCE_DIR}" "${spirv_SOURCE_DIR}")
set_target_properties(varpaper_scene_core PROPERTIES OUTPUT_NAME varpaper_scene)
target_compile_definitions(varpaper_scene_core PRIVATE GLM_ENABLE_EXPERIMENTAL NDEBUG)
target_link_options(varpaper_scene_core PRIVATE "LINKER:--no-undefined" "LINKER:--version-script=${CMAKE_CURRENT_SOURCE_DIR}/scene.exports")
set_property(TARGET varpaper_scene_core APPEND PROPERTY LINK_DEPENDS "${CMAKE_CURRENT_SOURCE_DIR}/scene.exports")
target_link_libraries(varpaper_scene_core PUBLIC
  PkgConfig::SCENE_SYSTEM glslang SPIRV spirv-cross-core spirv-cross-glsl qjs)

option(VARPAPER_NATIVE_TESTS "Build Scene native fixtures" ON)
if(VARPAPER_NATIVE_TESTS)
  enable_testing()
  add_executable(varpaper_scene_test tests/scene_test.cpp
    "${core}/Data/Utils/BinaryReader.cpp"
    "${core}/Data/Parsers/TextureParser.cpp"
    "${core}/Logging/Log.cpp"
    "${core}/Debugging/CallStack.cpp")
  target_link_libraries(varpaper_scene_test PRIVATE varpaper_scene_core)
  add_test(NAME scene-abi COMMAND varpaper_scene_test)
  set_tests_properties(scene-abi PROPERTIES ENVIRONMENT "SDL_AUDIODRIVER=dummy")
endif()
