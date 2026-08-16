#if defined(_WIN32)
    #ifndef NOMINMAX
        #define NOMINMAX
    #endif

    #ifndef WIN32_LEAN_AND_MEAN
        #define WIN32_LEAN_AND_MEAN
    #endif

    #include <windows.h>

    #define VK_USE_PLATFORM_WIN32_KHR
#elif defined(__APPLE__)
    #define VK_USE_PLATFORM_METAL_EXT
#elif defined(__linux__)
    #define VK_USE_PLATFORM_WAYLAND_KHR
    #define VK_USE_PLATFORM_XLIB_KHR
#endif

#include <vulkan/vulkan.h>
#include <stdexcept>
#include <cstring>
#include "renderer.h"
#include "renderer/src/bridge.rs.h"

bool VulkanRenderer::checkValidationLayerSupport() {
    uint32_t layerCount;
    vkEnumerateInstanceLayerProperties(&layerCount, nullptr);

    std::vector<VkLayerProperties> availableLayers(layerCount);
    vkEnumerateInstanceLayerProperties(&layerCount, availableLayers.data());

    for (const char* layerName : validationLayers) {
        bool layerFound = false;
        for (const auto& layerProperties : availableLayers) {
            if (strcmp(layerName, layerProperties.layerName) == 0) {
                layerFound = true;
                break;
            }
        }
        if (!layerFound) return false;
    }
    return true;
}

std::vector<const char*> getRequiredExtensions(bool is_x11) {
    std::vector<const char*> extensions = { VK_KHR_SURFACE_EXTENSION_NAME };

    #if defined(_WIN32)
        extensions.push_back(VK_KHR_WIN32_SURFACE_EXTENSION_NAME);
    #elif defined(__APPLE__)
        extensions.push_back(VK_EXT_METAL_SURFACE_EXTENSION_NAME);
        extensions.push_back(VK_KHR_PORTABILITY_ENUMERATION_EXTENSION_NAME);
    #elif defined(__linux__)
        if (is_x11) 
            extensions.push_back(VK_KHR_XLIB_SURFACE_EXTENSION_NAME);
        else 
            extensions.push_back(VK_KHR_WAYLAND_SURFACE_EXTENSION_NAME);
    #endif

    return extensions;
}

void VulkanRenderer::createInstance(const WindowHandles& handles) {
    if (enableValidationLayers && !checkValidationLayerSupport()) {
        throw std::runtime_error("validation layers requested, but not available!");
    }
    VkApplicationInfo appInfo{};
    appInfo.sType = VK_STRUCTURE_TYPE_APPLICATION_INFO;
    appInfo.pApplicationName = "Vuldu";
    appInfo.applicationVersion = VK_MAKE_VERSION(1, 0, 0);
    appInfo.pEngineName = nullptr;
    appInfo.engineVersion = VK_MAKE_VERSION(1, 0, 0);
    appInfo.apiVersion = VK_API_VERSION_1_4;

    VkInstanceCreateInfo createInfo{};
    createInfo.sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO;
    createInfo.pApplicationInfo = &appInfo;
    #if defined(__APPLE__)
        createInfo.flags |= VK_INSTANCE_CREATE_ENUMERATE_PORTABILITY_BIT_KHR;
    #endif

    std::vector<const char*> requiredExtensions = getRequiredExtensions(handles.is_x11);
    if (enableValidationLayers) {
        requiredExtensions.push_back(VK_EXT_DEBUG_UTILS_EXTENSION_NAME);
    }

    createInfo.enabledExtensionCount = static_cast<uint32_t>(requiredExtensions.size());
    createInfo.ppEnabledExtensionNames = requiredExtensions.data();

    VkDebugUtilsMessengerCreateInfoEXT debugCreateInfo{};
    if (enableValidationLayers) {
        createInfo.enabledLayerCount = static_cast<uint32_t>(validationLayers.size());
        createInfo.ppEnabledLayerNames = validationLayers.data();
        populateDebugMessengerCreateInfo(debugCreateInfo);
        createInfo.pNext = (VkDebugUtilsMessengerCreateInfoEXT*) &debugCreateInfo;
    } else {
        createInfo.enabledLayerCount = 0;
    }

    VkResult instanceResult = vkCreateInstance(&createInfo, nullptr, &this->instance);
    if (instanceResult != VK_SUCCESS) {
        throw std::runtime_error("failed to create instance!");
    }
};

void VulkanRenderer::createSurface(const WindowHandles& handles) {
    VkResult surfaceResult = VK_ERROR_INITIALIZATION_FAILED;

    #if defined(_WIN32)
        VkWin32SurfaceCreateInfoKHR createInfo{};
        createInfo.sType = VK_STRUCTURE_TYPE_WIN32_SURFACE_CREATE_INFO_KHR;
        createInfo.hinstance = reinterpret_cast<HINSTANCE>(handles.display_ptr);
        createInfo.hwnd = reinterpret_cast<HWND>(handles.window_ptr);
        surfaceResult = vkCreateWin32SurfaceKHR(this->instance, &createInfo, nullptr, &this->surface);
    #elif defined(__APPLE__)
        VkMetalSurfaceCreateInfoEXT createInfo{};
        createInfo.sType = VK_STRUCTURE_TYPE_METAL_SURFACE_CREATE_INFO_EXT;
        createInfo.pLayer = reinterpret_cast<const CAMetalLayer*>(handles.window_ptr);
        surfaceResult = vkCreateMetalSurfaceEXT(this->instance, &createInfo, nullptr, &this->surface);
    #elif defined(__linux__)
        if (handles.is_x11) {
            VkXlibSurfaceCreateInfoKHR createInfo{};
            createInfo.sType = VK_STRUCTURE_TYPE_XLIB_SURFACE_CREATE_INFO_KHR;
            createInfo.dpy = reinterpret_cast<Display*>(handles.display_ptr);
            createInfo.window = reinterpret_cast<Window>(handles.window_ptr);
            surfaceResult = vkCreateXlibSurfaceKHR(this->instance, &createInfo, nullptr, &this->surface);
        } else {
            VkWaylandSurfaceCreateInfoKHR сreateInfo{};
            сreateInfo.sType = VK_STRUCTURE_TYPE_WAYLAND_SURFACE_CREATE_INFO_KHR;
            сreateInfo.display = reinterpret_cast<struct wl_display*>(handles.display_ptr);
            сreateInfo.surface = reinterpret_cast<struct wl_surface*>(handles.window_ptr);
            surfaceResult = vkCreateWaylandSurfaceKHR(this->instance, &сreateInfo, nullptr, &this->surface);
        }
    #endif
    
    if (surfaceResult != VK_SUCCESS) {
        throw std::runtime_error("failed to create surface!");
    }
};
