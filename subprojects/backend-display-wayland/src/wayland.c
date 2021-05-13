#define _POSIX_C_SOURCE 200112L

#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>

#include <wayland-client.h>
#include <xdg-shell-client-protocol.h>

#include <halley-kart/backend-display.h>
#include <halley-kart/plugin-manager-client.h>

static const char* const PLUGIN_ID = "hkwl";
static const char* const PLUGIN_NAME = "HK Wayland Display Backend";

struct client_state {
    struct wl_display* wl_display;
    struct wl_registry* wl_registry;
    uint32_t wl_compositor_name;
    struct wl_compositor* wl_compositor;
    uint32_t wl_shm_name;
    struct wl_shm* wl_shm;
    uint32_t xdg_wm_base_name;
    struct xdg_wm_base* xdg_wm_base;
};

static void registry_listen_global(
    void* data,
    struct wl_registry* wl_registry,
    uint32_t name,
    const char* interface,
    uint32_t version
) {
    (void) version;
    struct client_state* state = data;

    if (strcmp(interface, wl_compositor_interface.name) == 0) {
        state->wl_compositor_name = name;
        state->wl_compositor =
            wl_registry_bind(wl_registry, name, &wl_compositor_interface, 4);
    } else if (strcmp(interface, wl_shm_interface.name) == 0) {
        state->wl_shm_name = name;
        state->wl_shm =
            wl_registry_bind(wl_registry, name, &wl_shm_interface, 1);
    } else if (strcmp(interface, xdg_wm_base_interface.name) == 0) {
        state->xdg_wm_base_name = name;
        state->xdg_wm_base =
            wl_registry_bind(wl_registry, name, &xdg_wm_base_interface, 1);
    }
}

static void registry_listen_global_remove(
    void* data,
    struct wl_registry* wl_registry,
    uint32_t name
) {
    (void) wl_registry;
    struct client_state* state = data;

    if (name == state->wl_compositor_name) {
        fprintf(stderr, "FATAL: Wayland registry removed wl_compositor\n");
        exit(1);
    } else if (name == state->wl_shm_name) {
        fprintf(stderr, "FATAL: Wayland registry removed wl_shm\n");
        exit(1);
    } else if (name == state->xdg_wm_base_name) {
        fprintf(stderr, "FATAL: Wayland registry removed xdg_wm_base\n");
        exit(1);
    }
}

static const struct wl_registry_listener registry_listener = {
    registry_listen_global,
    registry_listen_global_remove,
};

static void xdg_wm_base_listen_ping(
    void* data,
    struct xdg_wm_base* xdg_wm_base,
    uint32_t serial
) {
    (void) data;

    xdg_wm_base_pong(xdg_wm_base, serial);
}

static const struct xdg_wm_base_listener wm_base_listener = {
    xdg_wm_base_listen_ping,
};

static void client_connect(struct client_state* state) {
    state->wl_display = wl_display_connect(NULL);
    if (state->wl_display == NULL) {
        fprintf(stderr, "FATAL: failed to connect to Wayland display\n");
        exit(1);
    }

    state->wl_registry = wl_display_get_registry(state->wl_display);
    wl_registry_add_listener(state->wl_registry, &registry_listener, state);
    // NOTE: blocking calls
    wl_display_dispatch(state->wl_display);
    wl_display_roundtrip(state->wl_display);

    if (state->wl_compositor == NULL) {
        fprintf(
            stderr,
            "FATAL: Wayland registry did not provide wl_compositor"
        );
        exit(1);
    }
    if (state->wl_shm == NULL) {
        fprintf(stderr, "FATAL: Wayland registry did not provide wl_shm\n");
        exit(1);
    }
    if (state->xdg_wm_base == NULL) {
        fprintf(
            stderr,
            "FATAL: Wayland registry did not provide xdg_wm_base\n"
        );
        exit(1);
    }

    xdg_wm_base_add_listener(state->xdg_wm_base, &wm_base_listener, state);
}

static void client_disconnect(struct client_state* state) {
    // No cleanup necessary here, because when I disconnect,
    // everything will be freed. Or, at least I hope that's how it works.
    wl_display_disconnect(state->wl_display);
}

struct window_state {
    struct client_state* client_state;
    struct wl_surface* wl_surface;
    struct xdg_surface* xdg_surface;
    struct xdg_toplevel* xdg_toplevel;
    struct wl_buffer* wl_buffer;
};

static void wl_buffer_release(void* data, struct wl_buffer* wl_buffer) {
    (void) data;
    wl_buffer_destroy(wl_buffer);
}

static const struct wl_buffer_listener wl_buffer_listener = {
    .release = wl_buffer_release,
};

static int create_shm_file(void) {
    int fd = shm_open("/wl_shm", O_RDWR | O_CREAT | O_EXCL, 0600);
    if (fd < 0) {
        fprintf(stderr, "FATAL: failed to create shared memory for Wayland\n");
        exit(1);
    }
    shm_unlink("/wl_shm");
    return fd;
}

static int allocate_shm_file(size_t size) {
    int fd = create_shm_file();
    int ret = ftruncate(fd, size);
    if (ret < 0) {
        close(fd);
        fprintf(stderr, "FATAL: failed to allocate shared memory for Wayland\n");
        exit(1);
    }
    return fd;
}

static struct wl_buffer* draw_frame(struct client_state* state) {
    const int width = 1920, height = 1080;
    const int stride = width * 4;
    const int shm_pool_size = height * stride * 2;

    int fd = allocate_shm_file(shm_pool_size);
    uint8_t* pool_data =
        mmap((void*) -1, shm_pool_size, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);

    struct wl_shm_pool* pool =
        wl_shm_create_pool(state->wl_shm, fd, shm_pool_size);
    struct wl_buffer* buffer =
        wl_shm_pool_create_buffer(
            pool,
            0,
            width,
            height,
            stride,
            WL_SHM_FORMAT_XRGB8888
    );
    wl_shm_pool_destroy(pool);
    close(fd);

    uint32_t *pixels = (uint32_t *)&pool_data[0];
    for (int y = 0; y < height; ++y) {
        for (int x = 0; x < width; ++x) {
            if ((x + y / 8 * 8) % 16 < 8) {
                pixels[y * width + x] = 0xFF666666;
            } else {
                pixels[y * width + x] = 0xFFEEEEEE;
            }
        }
    }

    munmap(pool_data, shm_pool_size);
    wl_buffer_add_listener(buffer, &wl_buffer_listener, state);
    return buffer;
}

static void xdg_surface_configure(
    void* data,
    struct xdg_surface* xdg_surface,
    uint32_t serial
) {
    struct window_state* state = data;
    xdg_surface_ack_configure(xdg_surface, serial);

    state->wl_buffer = draw_frame(state->client_state);
    wl_surface_attach(state->wl_surface, state->wl_buffer, 0, 0);
    wl_surface_commit(state->wl_surface);
}

static const struct xdg_surface_listener xdg_surface_listener = {
    .configure = xdg_surface_configure,
};

static void window_create(
    struct window_state* window_state
) {
    window_state->wl_surface =
        wl_compositor_create_surface(window_state->client_state->wl_compositor);
    window_state->xdg_surface =
        xdg_wm_base_get_xdg_surface(
            window_state->client_state->xdg_wm_base,
            window_state->wl_surface
        );
    xdg_surface_add_listener(
        window_state->xdg_surface,
        &xdg_surface_listener,
        &window_state
    );
    window_state->xdg_toplevel =
        xdg_surface_get_toplevel(window_state->xdg_surface);
    xdg_toplevel_set_title(window_state->xdg_toplevel, "Halley Kart");
    xdg_toplevel_set_app_id(window_state->xdg_toplevel, "halley-kart");
    wl_surface_commit(window_state->wl_surface);
}

static void window_destroy(struct window_state* window_state) {
    if (window_state->wl_buffer != NULL) {
        wl_buffer_destroy(window_state->wl_buffer);
    }
    xdg_toplevel_destroy(window_state->xdg_toplevel);
    xdg_surface_destroy(window_state->xdg_surface);
    wl_surface_destroy(window_state->wl_surface);
}

struct hkwl_state {
    struct hk_backend_handle* backend_handle;
    struct client_state* client_state;
    struct window_state* window_state;
};

static void* backend_initialize(void* data) {
    struct hkwl_state* state = data;
    state->client_state = calloc(1, sizeof(struct client_state));

    client_connect(state->client_state);

    return state;
}

static void* display_create(void* data) {
    struct hkwl_state* state = data;
    state->window_state = calloc(1, sizeof(struct window_state));
    state->window_state->client_state = state->client_state;

    window_create(state->window_state);

    return state->window_state;
}

static void display_destroy(void* data, void* display_handle) {
    struct hkwl_state* state = data;
    struct window_state* window_state = display_handle;

    window_destroy(window_state);

    free(window_state);
    state->window_state = NULL;
}

static void backend_finish(void* data) {
    struct hkwl_state* state = data;

    client_disconnect(state->client_state);

    if (state->window_state != NULL) {
        display_destroy(state, state->window_state);
    }
    free(state->client_state);
    state->client_state = NULL;
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
    struct hk_plugin_handle* plugin
) {
    struct hkwl_state* state = malloc(sizeof(struct hkwl_state));
    state->backend_handle =
        hk_plugin_register_backend(
            mngr,
            plugin,
            &hk_backend_display_procs.backend_procs
        );
    return state;
}

static void plugin_finish(
    struct hk_plugin_manager* mngr,
    struct hk_plugin_handle* plugin,
    void* data
) {
    (void) plugin;
    struct hkwl_state* state = data;
    if (state->client_state != NULL) {
        backend_finish(state);
    }
    hk_plugin_unregister_backend(mngr, state->backend_handle);
    free(state);
}

const struct hk_plugin_procs* hkwl_get_procs(void);

const struct hk_plugin_procs* hkwl_get_procs(void) {
    return &hk_plugin_procs;
}

static const struct hk_plugin_procs hk_plugin_procs = {
    .get_procs = hkwl_get_procs,
    .id = PLUGIN_ID,
    .name = PLUGIN_NAME,
    .initialize = plugin_initialize,
    .finish = plugin_finish,
};
