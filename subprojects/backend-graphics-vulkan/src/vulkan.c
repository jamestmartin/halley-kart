#include <vulkan/vulkan.h>

#include <stdio.h>
#include <stdlib.h>

void hkvk_do_graphics(void) {
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
