#include <stdlib.h>

#include <halley-kart/backend-audio.h>
#include <halley-kart/plugin-manager-client.h>

static const char* const PLUGIN_ID = "hkal";
static const char* const PLUGIN_NAME = "HK ALSA Audio Backend";

struct plugin_state {
    struct hk_backend_handle* backend_handle;
};

struct backend_state {
    char padding[1];
};

static void do_audio(void* backend) {
    (void) backend;
}

static void* backend_initialize(void* plugin) {
    (void) plugin;
    struct backend_state* backend_state = malloc(sizeof(backend_state));
    return backend_state;
}

static void backend_finish(void* backend) {
    struct backend_state* backend_state = backend;
    free(backend_state);
}

static const struct hk_backend_audio_procs hk_backend_audio_procs = {
    .backend_procs = {
        .type = HK_BACKEND_AUDIO,
        .id = PLUGIN_ID,
        .name = PLUGIN_NAME,
        .initialize = backend_initialize,
        .finish = backend_finish,
    },
    .do_audio = do_audio,
};

static const struct hk_plugin_procs hk_plugin_procs;

static void* plugin_initialize(
    struct hk_plugin_manager* mngr,
    struct hk_plugin_handle* plugin_handle
) {
    struct plugin_state* plugin_state = malloc(sizeof(struct plugin_state));
    
    plugin_state->backend_handle =
        hk_plugin_register_backend(
            mngr,
            plugin_handle,
            &hk_backend_audio_procs.backend_procs
        );

    return plugin_state;
}

static void plugin_finish(
    struct hk_plugin_manager* mngr,
    struct hk_plugin_handle* plugin_handle,
    void* plugin
) {
    (void) plugin_handle;
    struct plugin_state* plugin_state = plugin;

    hk_plugin_unregister_backend(mngr, plugin_state->backend_handle);

    free(plugin_state);
}

const struct hk_plugin_procs* hkal_get_procs(void) {
    return &hk_plugin_procs;
}

static const struct hk_plugin_procs hk_plugin_procs = {
    .get_procs = hkal_get_procs,
    .id = PLUGIN_ID,
    .name = PLUGIN_NAME,
    .initialize = plugin_initialize,
    .finish = plugin_finish,
};
