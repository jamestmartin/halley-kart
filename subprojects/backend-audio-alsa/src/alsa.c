#include <stdlib.h>

#include <halley-kart/backend-audio.h>
#include <halley-kart/plugin-manager-client.h>

static const char* const PLUGIN_ID = "hkal";
static const char* const PLUGIN_NAME = "HK ALSA Audio Backend";

struct hkal_state {
    struct hk_backend_handle* backend_handle;
};

static void do_audio(void* state) {
    (void) state;
}

static void* backend_initialize(void* state) {
    return state;
}

static void backend_finish(void* state) {
    (void) state;
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
    struct hk_plugin_handle* plugin
) {
    struct hkal_state* state = malloc(sizeof(struct hkal_state));
    state->backend_handle =
        hk_plugin_register_backend(
            mngr,
            plugin,
            &hk_backend_audio_procs.backend_procs
        );
    return state;
}

static void plugin_finish(
    struct hk_plugin_manager* mngr,
    struct hk_plugin_handle* plugin,
    void* data
) {
    (void) plugin;
    struct hkal_state* state = data;
    hk_plugin_unregister_backend(mngr, state->backend_handle);
    free(state);
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
