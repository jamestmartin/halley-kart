#include <stdlib.h>
#include <stdio.h>
#include <unistd.h>

#include <xcb/xcb.h>

#include <halley-kart/backend-display.h>
#include <halley-kart/plugin-manager-client.h>

static const char* const PLUGIN_ID = "hkx11";
static const char* const PLUGIN_NAME = "HK X11 Display Backend";

struct plugin_state {
    struct hk_backend_handle* backend_handle;
};

struct client_state {
    xcb_connection_t* xcb_connection;
    int preferred_screen;
    xcb_screen_t* xcb_screen;
    struct window_state* window_state;
};

struct window_state {
    struct client_state* client_state;
    xcb_window_t xcb_window;
};

void* display_create(void* client) {
    struct client_state* client_state = client;
    struct window_state* window_state  = malloc(sizeof(struct window_state));
    window_state->client_state = client_state;

    window_state->xcb_window = xcb_generate_id(client_state->xcb_connection);
    xcb_create_window(
        client_state->xcb_connection,
        XCB_COPY_FROM_PARENT,
        window_state->xcb_window,
        client_state->xcb_screen->root,
        0, 0,
        800, 600,
        1,
        XCB_WINDOW_CLASS_INPUT_OUTPUT,
        client_state->xcb_screen->root_visual,
        0, NULL
    );

    xcb_map_window(client_state->xcb_connection, window_state->xcb_window);
    xcb_flush(client_state->xcb_connection);

    sleep(3);

    return window_state;
}

void display_destroy(void* window) {
    struct window_state* window_state = window;
    struct client_state* client_state = window_state->client_state;

    xcb_destroy_window(client_state->xcb_connection, window_state->xcb_window);
    xcb_flush(client_state->xcb_connection);

    free(window_state);
}

static void* backend_initialize(void* plugin) {
    (void) plugin;

    struct client_state* client_state = malloc(sizeof(struct client_state));

    client_state->xcb_connection =
        xcb_connect(NULL, &client_state->preferred_screen);
    if (xcb_connection_has_error(client_state->xcb_connection)) {
        fprintf(stderr, "FATAL: failed to connect to x11 display\n");
        exit(1);
    }

    int screen_nbr = client_state->preferred_screen;
    xcb_screen_iterator_t iter =
        xcb_setup_roots_iterator(xcb_get_setup(client_state->xcb_connection));
    for (; iter.rem; --screen_nbr, xcb_screen_next(&iter)) {
        if (screen_nbr == 0) {
            client_state->xcb_screen = iter.data;
            break;
        }
    }

    return client_state;
}

static void backend_finish(void* client) {
    struct client_state* client_state = client;

    xcb_disconnect(client_state->xcb_connection);

    free(client_state);
}

static const struct hk_backend_display_procs hk_backend_display_procs = {
    .backend_procs = {
        .type = HK_BACKEND_DISPLAY,
        .id = PLUGIN_ID,
        .name = PLUGIN_NAME,
        .initialize = backend_initialize,
        .finish = backend_finish,
    },
    .display_create = display_create,
    .display_destroy = display_destroy,
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
            &hk_backend_display_procs.backend_procs
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

const struct hk_plugin_procs* hkx11_get_procs(void) {
    return &hk_plugin_procs;
}

static const struct hk_plugin_procs hk_plugin_procs = {
    .get_procs = hkx11_get_procs,
    .id = PLUGIN_ID,
    .name = PLUGIN_NAME,
    .initialize = plugin_initialize,
    .finish = plugin_finish,
};
