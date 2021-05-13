#include <stdio.h>
#include <stdlib.h>

#include <vulkan/vulkan.h>

#include <halley-kart/backend-graphics.h>
#include <halley-kart/plugin-manager-client.h>

static const char* const PLUGIN_ID = "hkvk";
static const char* const PLUGIN_NAME = "HK Vulkan Graphics Backend";

struct plugin_state {
    struct hk_backend_handle* backend_handle;
};

struct backend_state {
    char padding[1];
};

static void do_graphics(void* backend) {
    (void) backend;

    static const VkApplicationInfo applicationInfo = {
        .sType = VK_STRUCTURE_TYPE_APPLICATION_INFO,
        .pApplicationName = "Halley Kart",
        .applicationVersion = VK_MAKE_VERSION(0, 1, 0),
        .pEngineName = "Halley Kart",
        .engineVersion = VK_MAKE_VERSION(0, 1, 0),
        .apiVersion = VK_API_VERSION_1_0,
    };

    static const VkInstanceCreateInfo createInfo = {
        .sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
        .pNext = NULL,
        .pApplicationInfo = &applicationInfo,
        .enabledLayerCount = 0,
        .enabledExtensionCount = 0,
    };

    VkInstance instance;
    if (vkCreateInstance(&createInfo, NULL, &instance) != VK_SUCCESS) {
        fprintf(stderr, "FATAL: failed to create Vulkan instance\n");
        exit(1);
    }

    vkDestroyInstance(instance, NULL);
}

static void* backend_initialize(void* plugin) {
    (void) plugin;
    struct backend_state* backend_state = malloc(sizeof(struct backend_state));

    return backend_state;
}

static void backend_finish(void* backend) {
    struct backend_state* backend_state = backend;

    free(backend_state);
}

static const struct hk_backend_graphics_procs hk_backend_graphics_procs = {
    .backend_procs = {
        .type = HK_BACKEND_GRAPHICS,
        .id = PLUGIN_ID,
        .name = PLUGIN_NAME,
        .initialize = backend_initialize,
        .finish = backend_finish,
    },
    .do_graphics = do_graphics,
};

static const struct hk_plugin_procs hk_plugin_procs;

static void* plugin_initialize(
    struct hk_plugin_manager* mngr,
    struct hk_plugin_handle* plugin
) {
    struct plugin_state* plugin_state = malloc(sizeof(struct plugin_state));

    plugin_state->backend_handle =
        hk_plugin_register_backend(
            mngr,
            plugin,
            &hk_backend_graphics_procs.backend_procs
        );

    return plugin_state;
}

static void plugin_finish(
    struct hk_plugin_manager* mngr,
    struct hk_plugin_handle* plugin,
    void* data
) {
    (void) plugin;
    struct plugin_state* plugin_state = data;

    hk_plugin_unregister_backend(mngr, plugin_state->backend_handle);
    
    free(plugin_state);
}

const struct hk_plugin_procs* hkvk_get_procs(void) {
    return &hk_plugin_procs;
}

static const struct hk_plugin_procs hk_plugin_procs = {
    .get_procs = hkvk_get_procs,
    .id = PLUGIN_ID,
    .name = PLUGIN_NAME,
    .initialize = plugin_initialize,
    .finish = plugin_finish,
};
