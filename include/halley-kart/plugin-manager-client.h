#ifndef HK_PLUGIN_MANAGER_CLIENT_H
#define HK_PLUGIN_MANAGER_CLIENT_H

#include "backend.h"
#include "plugin.h"

extern struct hk_plugin_handle* hk_plugin_register(
    struct hk_plugin_manager* mngr,
    const struct hk_plugin_procs* procs
);

extern void hk_plugin_unregister(
    struct hk_plugin_manager* mngr,
    struct hk_plugin_handle* plugin
);

extern struct hk_backend_handle* hk_plugin_register_backend(
    struct hk_plugin_manager* mngr,
    struct hk_plugin_handle* plugin,
    const struct hk_backend_procs* procs
);

extern void hk_plugin_unregister_backend(
    struct hk_plugin_manager* mngr,
    struct hk_backend_handle* handle
);

#endif /* HK_PLUGIN_MANAGER_CLIENT_H */
