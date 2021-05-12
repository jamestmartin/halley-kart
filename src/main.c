#include <dlfcn.h>

#include <stdlib.h>
#include <stdio.h>
#include <string.h>

struct backend_dependency_dlls {
    void* asound;
    void* vulkan;
    void* wayland_client;
};

struct backend_dlls {
    void* alsa;
    void* vulkan;
    void* wayland;
};

static void* load_backend_dependency_dll(char* dll, char* lib, char* backend) {
    void* handle = dlopen(dll, RTLD_NOW | RTLD_LOCAL);
    if (handle == NULL) {
        fprintf(stderr, "WARN: error loading %s: %s\n", dll, dlerror());
        fprintf(
            stderr,
            "WARN: %s not present; %s will not be available\n",
            lib,
            backend
        );
    }
    return handle;
}

static struct backend_dependency_dlls load_backend_dependencies(void) {
    void* libasound =
        load_backend_dependency_dll(
            "libasound.so.2",
            "libasound",
            "ALSA audio backend"
        );
    void* libvulkan =
        load_backend_dependency_dll(
            "libvulkan.so.1",
            "libvulkan",
            "Vulkan graphics backend"
        );
    void* libwayland_client =
        load_backend_dependency_dll(
            "libwayland-client.so.0",
            "libwayland-client",
            "Wayland display backend"
        );

    struct backend_dependency_dlls dlls = {
        .asound = libasound,
        .vulkan = libvulkan,
        .wayland_client = libwayland_client,
    };
    return dlls;
}

static void* load_backend_dll(char* dll, char* lib, char* backend, void* dep) {
    void* handle = NULL;
    if (dep != NULL) {
        handle = dlopen(dll, RTLD_NOW | RTLD_LOCAL);
        // HACK: hard-coded search paths for backend shared objects
        // I don't like this solution, but at least it's *a* solution.
        // glibc's dlopen treats any name with a slash *anywhere* as a path,
        // so I can't try to dlopen `halley-kart/dll.so`,
        // and trying to use ld `-rname` makes ld expect the library path to
        // exist at compile-time (so it can add it to the link path),
        // and there's no option to make it *just* set the ELF LD_READPATH.
        // I could maybe make meson run a patchelf command,
        // but then I need patchelf to be installed to build,
        // and having to patch the file right after it's created is weird.
        const char* path1 = "/usr/local/lib/x86_64-linux-gnu/halley-kart";
        const char* path2 = "/usr/lib/x86_64-linux-gnu/halley-kart";
        if (handle == NULL) {
            size_t dll2_size = sizeof(char) * (strlen(path1) + strlen(dll) + 2);
            char* dll2 = (char*) malloc(dll2_size);
            snprintf(dll2, dll2_size, "%s/%s", path1, dll);
            handle = dlopen(dll2, RTLD_NOW | RTLD_LOCAL);
            free(dll2);
        }
        if (handle == NULL) {
            size_t dll3_size = sizeof(char) * (strlen(path2) + strlen(dll) + 2);
            char* dll3 = (char*) malloc(dll3_size);
            snprintf(dll3, dll3_size, "%s/%s", path2, dll);
            handle = dlopen(dll3, RTLD_NOW | RTLD_LOCAL);
            free(dll3);
        }
        if (handle == NULL) {
            fprintf(stderr, "WARN: error loading %s\n", lib);
        }
    }
    if (handle == NULL) {
        fprintf(stderr, "WARN: %s not available\n", backend);
    }
    return handle;
}

static struct backend_dlls load_backends(struct backend_dependency_dlls deps) {
    void* alsa =
        load_backend_dll(
            "hk-backend-audio-alsa.so",
            "hk-backend-audio-alsa",
            "ALSA audio backend",
            deps.asound
        );
    void* vulkan =
        load_backend_dll(
            "hk-backend-graphics-vulkan.so",
            "hk-backend-graphics-vulkan",
            "Vulkan graphics backend",
            deps.vulkan
        );
    void* wayland =
        load_backend_dll(
            "hk-backend-display-wayland.so",
            "hk-backend-display-wayland",
            "Wayland display backend",
            deps.wayland_client
        );

    struct backend_dlls dlls = {
        .alsa = alsa,
        .vulkan = vulkan,
        .wayland = wayland,
    };
    return dlls;
}

static void unload_dll(char* name, void* handle) {
    if (handle != NULL && dlclose(handle) != 0) {
        fprintf(stderr, "WARN: error unloading %s: %s\n", name, dlerror());
    }
}

static void unload_backends(struct backend_dlls dlls) {
    unload_dll("hk-backend-audio-alsa.so", dlls.alsa);
    unload_dll("hk-backend-graphics-vulkan.so", dlls.vulkan);
    unload_dll("hk-backend-display-wayland.so", dlls.wayland);
}

static void unload_backend_dependencies(struct backend_dependency_dlls dlls) {
    unload_dll("libasound.so.2", dlls.asound);
    unload_dll("libvulkan.so.1", dlls.vulkan);
    unload_dll("libwayland-client.so.0", dlls.wayland_client);
}

int main(void) {
    struct backend_dependency_dlls backend_dependency_dlls
        = load_backend_dependencies();
    struct backend_dlls backend_dlls
        = load_backends(backend_dependency_dlls);

    if (backend_dlls.alsa == NULL) {
        fprintf(stderr, "FATAL: no audio backend available\n");
        exit(1);
    }
    if (backend_dlls.vulkan == NULL) {
        fprintf(stderr, "FATAL: no graphics backend available\n");
        exit(1);
    }
    if (backend_dlls.wayland == NULL) {
        fprintf(stderr, "FATAL: no display backend available\n");
        exit(1);
    }

#pragma GCC diagnostic push
// ISO C does not allow casting `void*` to a function pointer,
// which is inherently incompatible with `dlsym`.
#pragma GCC diagnostic ignored "-Wpedantic"
    void (*do_audio)(void);
    do_audio = dlsym(backend_dlls.alsa, "hkalsa_do_audio");
    if (do_audio == NULL) {
        fprintf(stderr, "FATAL: audio backend is missing symbols\n");
        exit(1);
    }

    void (*do_display)(void);
    do_display = dlsym(backend_dlls.wayland, "hkwl_do_display");
    if (do_audio == NULL) {
        fprintf(stderr, "FATAL: display backend is missing symbols\n");
        exit(1);
    }

    void (*do_graphics)(void);
    do_graphics = (void (*)(void)) dlsym(backend_dlls.vulkan, "hkvk_do_graphics");
    if (do_graphics == NULL) {
        fprintf(stderr, "FATAL: graphics backend is missing symbols\n");
        exit(1);
    }
#pragma GCC diagnostic pop

    do_audio();
    do_display();
    do_graphics();

    unload_backends(backend_dlls);
    unload_backend_dependencies(backend_dependency_dlls);

    exit(0);
}
