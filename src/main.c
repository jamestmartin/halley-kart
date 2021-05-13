#define _DEFAULT_SOURCE

#include <dirent.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <unistd.h>

#include <halley-kart/backend-audio.h>
#include <halley-kart/backend-display.h>
#include <halley-kart/backend-graphics.h>
#include <halley-kart/plugin-manager.h>

static size_t indexof(size_t len, void* arr[], void* value) {
    for (size_t i = 0; i < len; i++) {
        if (arr[i] == value) {
            return i;
        }
    }
    return (size_t) -1;
}

int main(int argc, char** argv) {
    static const char* const usage = "usage: halley-kart <plugins directory>\n";
    if (argc < 2) {
        fprintf(stderr, "not enough arguments\n%s", usage);
        return 1;
    }
    if (argc > 2) {
        fprintf(stderr, "too many arguments\n%s", usage);
        return 1;
    }
    char* plugins_path = argv[1];
    DIR* plugins_dir = opendir(plugins_path);
    if (plugins_dir == NULL) {
        fprintf(
            stderr,
            "failed to open plugins directory %s: %s\n%s",
            plugins_path,
            strerror(errno),
            usage
        );
        return 1;
    }

    struct hk_plugin_manager* mngr = hk_plugin_manager_create();

    struct dirent* dirent;

    size_t num_entries = 0;
    while ((dirent = readdir(plugins_dir)) != NULL) {
        if (dirent->d_type != DT_REG) {
            continue;
        }
        num_entries++;
    }
    rewinddir(plugins_dir);

    struct hk_plugin_dll plugin_dlls[num_entries];
    struct hk_plugin_handle* plugin_handles[num_entries];
    void* plugin_states[num_entries];
    size_t num_plugins = 0;
    while (
        (dirent = readdir(plugins_dir)) != NULL
        && num_plugins < num_entries
    ) {
        if (dirent->d_type != DT_REG) {
            continue;
        }
        size_t path_len = strlen(plugins_path) + strlen(dirent->d_name) + 1;
        char path[path_len];
        memcpy(path, plugins_path, strlen(plugins_path));
        memcpy(
            path + strlen(plugins_path),
            dirent->d_name,
            strlen(dirent->d_name)
        );
        path[strlen(plugins_path) + strlen(dirent->d_name)] = '\0';
        struct hk_plugin_dll dll = hk_plugin_load_dll(path);
        if (dll.dll_handle == NULL) {
            continue;
        }
        plugin_dlls[num_plugins] = dll;
        plugin_handles[num_plugins] = hk_plugin_register(mngr, dll.procs);
        plugin_states[num_plugins] =
            dll.procs->initialize(mngr, plugin_handles[num_plugins]);
        num_plugins++;
    }

    closedir(plugins_dir);

    size_t backends_len = 10;
    size_t num_backends;
    struct hk_backend_handle** backends = malloc(32 * sizeof(struct hk_backend_handle*));

    num_backends = hk_plugin_enumerate_backends(mngr, HK_BACKEND_AUDIO, backends_len, backends);
    num_backends = hk_plugin_enumerate_backends(mngr, HK_BACKEND_DISPLAY, backends_len, backends);
    for (size_t i = 0; i < num_backends; i++) {
        struct hk_backend_handle* backend = backends[i];
        const struct hk_backend_display_procs* procs =
            (const struct hk_backend_display_procs*) hk_backend_get_procs(backend);
        size_t plugin_index =
            indexof(num_plugins, (void**) plugin_handles, hk_backend_get_plugin(backend));
        void* plugin_state = plugin_states[plugin_index];
        void* backend_state = procs->backend_procs.initialize(plugin_state);
        void* display_state = procs->display_create(backend_state);
        sleep(3);
        procs->display_destroy(display_state);
        procs->backend_procs.finish(backend_state);
    }

    num_backends = hk_plugin_enumerate_backends(mngr, HK_BACKEND_GRAPHICS, backends_len, backends);

    free(backends);

    for (size_t i = 0; i < num_plugins; i++) {
        plugin_dlls[i].procs->finish(mngr, plugin_handles[i], plugin_states[i]);
        hk_plugin_unregister(mngr, plugin_handles[i]);
        hk_plugin_unload_dll(plugin_dlls[i]);
    }

    hk_plugin_manager_destroy(mngr);

    exit(0);
}
