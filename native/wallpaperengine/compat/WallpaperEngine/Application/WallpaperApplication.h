#pragma once

// VarPaper host context. The upstream application and its desktop drivers are
// deliberately not linked; Flutter/Rust retain lifecycle and surface ownership.
#include "WallpaperEngine/Application/ApplicationContext.h"
#include "WallpaperEngine/Data/Model/Project.h"
#include "WallpaperEngine/Data/Model/Wallpaper.h"
#include <GL/glew.h>
#include <map>

namespace WallpaperEngine::Application {
class WallpaperApplication {
public:
    explicit WallpaperApplication(ApplicationContext& context) : context_(context) {}
    ApplicationContext& getContext() const { return context_; }
    const std::map<std::string, Data::Model::ProjectUniquePtr>& getBackgrounds() const { return projects; }
    GLuint getDestinationFramebuffer() const { return framebuffer; }
    std::map<std::string, Data::Model::ProjectUniquePtr> projects;
    GLuint framebuffer = 0;
private:
    ApplicationContext& context_;
};
}
