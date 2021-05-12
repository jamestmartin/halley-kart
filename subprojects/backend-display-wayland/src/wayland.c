#define _POSIX_C_SOURCE 200809L

#include <wayland-client.h>
#include <wayland-client-protocol.h>
#include <xdg-shell-client-protocol.h>

#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>

struct hkwl_state {
    struct wl_compositor* compositor;
    struct wl_shm* shm;
    struct wl_surface* surface;
    struct xdg_surface* xdg_surface;
    struct xdg_toplevel* xdg_toplevel;
    struct xdg_wm_base* xdg_wm_base;
};

static void xdg_wm_base_ping(
    void* data,
    struct xdg_wm_base* xdg_wm_base,
    uint32_t serial
) {
    (void) data;
    xdg_wm_base_pong(xdg_wm_base, serial);
}

static const struct xdg_wm_base_listener xdg_wm_base_listener = {
    .ping = xdg_wm_base_ping,
};

static void registry_handle_global(
    void* data,
    struct wl_registry* registry,
    uint32_t id,
    const char* interface,
    uint32_t version
) {
    (void) version;
    struct hkwl_state* state = data;

    if (strcmp(interface, wl_compositor_interface.name) == 0) {
        state->compositor =
            wl_registry_bind(registry, id, &wl_compositor_interface, 4);
    } else if (strcmp(interface, wl_shm_interface.name) == 0) {
        state->shm =
            wl_registry_bind(registry, id, &wl_shm_interface, 1);
    } else if (strcmp(interface, xdg_wm_base_interface.name) == 0) {
        state->xdg_wm_base =
            wl_registry_bind(registry, id, &xdg_wm_base_interface, 1);
        xdg_wm_base_add_listener(
            state->xdg_wm_base,
            &xdg_wm_base_listener,
            state
        );
    }
}

static void registry_handle_global_remove(
    void* data,
    struct wl_registry* registry,
    uint32_t id
) {
    (void) data;
    (void) registry;
    (void) id;
}

static const struct wl_registry_listener registry_listener = {
    registry_handle_global,
    registry_handle_global_remove,
};

static int create_shm_file(void) {
    int fd = shm_open("/wl_shm", O_RDWR | O_CREAT | O_EXCL, 0600);
    if (fd < 0) {
        fprintf(stderr, "FATAL: failed to open shared memory for Wayland\n");
        exit(1);
    }
    shm_unlink("/wl_shm");
    return fd;
}

int allocate_shm_file(size_t size) {
    int fd = create_shm_file();
    int ret = ftruncate(fd, size);
    if (ret < 0) {
        close(fd);
        fprintf(stderr, "FATAL: failed to allocate shared memory for Wayland\n");
        exit(1);
    }
    return fd;
}

static void wl_buffer_release(void* data, struct wl_buffer* wl_buffer) {
    (void) data;
    wl_buffer_destroy(wl_buffer);
}

static const struct wl_buffer_listener wl_buffer_listener = {
    .release = wl_buffer_release,
};

static struct wl_buffer* draw_frame(struct hkwl_state* state) {
    const int width = 1920, height = 1080;
    const int stride = width * 4;
    const int shm_pool_size = height * stride * 2;

    int fd = allocate_shm_file(shm_pool_size);
    uint8_t* pool_data =
        mmap((void*) -1, shm_pool_size, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);

    struct wl_shm_pool* pool =
        wl_shm_create_pool(state->shm, fd, shm_pool_size);
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
    wl_buffer_add_listener(buffer, &wl_buffer_listener, NULL);
    return buffer;
}

static void xdg_surface_configure(
    void* data,
    struct xdg_surface* xdg_surface,
    uint32_t serial
) {
    struct hkwl_state* state = data;
    xdg_surface_ack_configure(xdg_surface, serial);

    struct wl_buffer* buffer = draw_frame(state);
    wl_surface_attach(state->surface, buffer, 0, 0);
    wl_surface_damage(state->surface, 0, 0, UINT32_MAX, UINT32_MAX);
    wl_surface_commit(state->surface);
}

static const struct xdg_surface_listener xdg_surface_listener = {
    .configure = xdg_surface_configure,
};

void hkwl_do_display(void) {
    struct wl_display* display = wl_display_connect(NULL);
    if (display == NULL) {
        fprintf(stderr, "FATAL: failed to connect to Wayland display\n");
        exit(1);
    }

    struct hkwl_state state;

    struct wl_registry* registry = wl_display_get_registry(display);
    wl_registry_add_listener(registry, &registry_listener, &state);

    wl_display_dispatch(display);
    wl_display_roundtrip(display);

    if (state.compositor == NULL) {
        fprintf(stderr, "FATAL: no Wayland compositor global\n");
        exit(1);
    }

    if (state.shm == NULL) {
        fprintf(stderr, "FATAL: no Wayland shared memory global\n");
        exit(1);
    }

    if (state.xdg_wm_base == NULL) {
        fprintf(stderr, "FATAL: no Wayland XDG WM base global\n");
        exit(1);
    }

    state.surface = wl_compositor_create_surface(state.compositor);

    state.xdg_surface =
        xdg_wm_base_get_xdg_surface(state.xdg_wm_base, state.surface);
    xdg_surface_add_listener(state.xdg_surface, &xdg_surface_listener, &state);

    state.xdg_toplevel = xdg_surface_get_toplevel(state.xdg_surface);
    xdg_toplevel_set_title(state.xdg_toplevel, "Halley Kart");
    xdg_toplevel_set_app_id(state.xdg_toplevel, "halley-kart");
    wl_surface_commit(state.surface);

    xdg_toplevel_destroy(state.xdg_toplevel);
    xdg_surface_destroy(state.xdg_surface);
    wl_surface_destroy(state.surface);
    wl_registry_destroy(registry);
    wl_display_disconnect(display);
}
