#ifndef HK_PLUGIN_MANAGER_H
#define HK_PLUGIN_MANAGER_H

#include <stddef.h>

#include "backend.h"
#include "plugin.h"
#include "plugin-manager-client.h"

struct hk_plugin_dll {
    void* dll_handle;
    const struct hk_plugin_procs* procs;
};

extern const struct hk_plugin_dll NULL_PLUGIN_DLL;

struct hk_plugin_dll hk_plugin_load_dll(const char* path);

void hk_plugin_unload_dll(struct hk_plugin_dll dll);

struct hk_plugin_manager* hk_plugin_manager_create(void);

void hk_plugin_manager_destroy(struct hk_plugin_manager* mngr);

size_t hk_plugin_enumerate_plugins(
    struct hk_plugin_manager* mngr,
    size_t len,
    struct hk_plugin_handle** plugins
);

size_t hk_plugin_enumerate_backends(
    struct hk_plugin_manager* mngr,
    enum hk_backend_type type,
    size_t len,
    struct hk_backend_handle** backends
);

#endif /* HK_PLUGIN_MANAGER_H */
