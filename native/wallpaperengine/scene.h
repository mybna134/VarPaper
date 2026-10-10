#pragma once
#include <stddef.h>
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
/* All calls and resource callbacks run on the creating render thread with
 * its OpenGL context current. Callbacks must never unwind across this ABI.
 * read returns owned bytes; release is called once even on parse failure.
 * exists returns 1/0, read returns 0 on success. Diagnostics are caller-owned. */
typedef struct vp_scene vp_scene;
typedef struct {
    void *userdata;
    int (*exists)(void *, const char *);
    int (*read)(void *, const char *, const uint8_t **, size_t *);
    void (*release)(void *, const uint8_t *, size_t);
    void *(*gl_proc)(void *, const char *);
} vp_scene_host;
typedef struct {
    const char *manifest_json;
    uint32_t width, height;
    uint32_t scaling; /* 0 default, 1 fit, 2 fill, 3 stretch */
    float volume;
    int muted;
} vp_scene_config;
int vp_scene_create(const vp_scene_host *, const vp_scene_config *, vp_scene **, char *, size_t);
int vp_scene_render(vp_scene *, uint32_t width, uint32_t height, uint32_t framebuffer,
                    double animation_time, double frame_delta, const float spectrum[64], char *, size_t);
int vp_scene_set_audio(vp_scene *, int paused, int muted, float volume, char *, size_t);
int vp_scene_destroy(vp_scene *, char *, size_t);
uint32_t vp_scene_abi_version(void);
size_t vp_scene_live_count(void);
#ifdef __cplusplus
}
#endif
