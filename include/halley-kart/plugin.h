#ifndef HK_PLUGIN_H
#define HK_PLUGIN_H

#define HK_PLUGIN_SECTION_NAME "HK-plugin"
#define HK_PLUGIN_NOTE_NAME "Halley Kart"
/**
 * The null-terminated symbol name of this plugin's get_procs function.
 * This note is required for all plugins.
 */
#define HK_PLUGIN_NOTE_TYPE_GET_PROCS 1
/**
 * If this note is present (with an empty descriptor),
 * then the plugin is an optional feature which is only loaded
 * if the needed shared objects are available for dynamic linking.
 */
#define HK_PLUGIN_NOTE_TYPE_OPTIONAL_DYNAMIC 2

struct hk_plugin_handle;
struct hk_plugin_manager;

struct hk_plugin_procs {
    const struct hk_plugin_procs* (*get_procs)(void);
    const char* id;
    const char* name;
    void* (*initialize)(
        struct hk_plugin_manager* mngr,
        struct hk_plugin_handle* plugin
    );
    void (*finish)(
        struct hk_plugin_manager* mngr,
        struct hk_plugin_handle* plugin,
        void* state
    );
};

#endif /* HK_PLUGIN_H */
