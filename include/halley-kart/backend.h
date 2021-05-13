#ifndef HK_BACKEND_H
#define HK_BACKEND_H

struct hk_backend_handle;

enum hk_backend_type {
    HK_BACKEND_AUDIO,
    HK_BACKEND_DISPLAY,
    HK_BACKEND_GRAPHICS,
};

struct hk_backend_procs {
    const enum hk_backend_type type;
    const char* id;
    const char* name;
    void* (*initialize)(void* plugin);
    void (*finish)(void* backend);
};

extern const struct hk_backend_procs* hk_backend_get_procs(
    const struct hk_backend_handle* handle
);

extern struct hk_plugin_handle* hk_backend_get_plugin(
    const struct hk_backend_handle* handle
);

#endif /* HK_BACKEND_H */
