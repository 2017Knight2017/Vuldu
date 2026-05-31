#ifndef VK_USE_PLATFORM_WAYLAND_KHR
#define VK_USE_PLATFORM_WAYLAND_KHR
#endif

#ifdef DEBUG_MODE
const bool enableValidationLayers = true;
#else
const bool enableValidationLayers = false;
#endif

#include "vulkan_renderer.h"
#include "validation_layers.h"
#include "doom/src/FFI.rs.h"
#include <optional>

struct QueueFamilyIndices {
    std::optional<uint32_t> graphicsFamily;

    bool isComplete() {
        return graphicsFamily.has_value();
    }
};

const std::vector<const char*> extensions = {
    VK_KHR_SURFACE_EXTENSION_NAME, 
    VK_KHR_WAYLAND_SURFACE_EXTENSION_NAME 
};

std::unique_ptr<VulkanRenderer> create_renderer(uint32_t width, uint32_t height) {
    return std::make_unique<VulkanRenderer>(width, height);
}

VulkanRenderer::VulkanRenderer(uint32_t width, uint32_t height) {
    this->instance = VK_NULL_HANDLE;
    this->physicalDevice = VK_NULL_HANDLE;
    this->device = VK_NULL_HANDLE;
    this->surface = VK_NULL_HANDLE;
    this->graphicsQueue = VK_NULL_HANDLE;
}

VulkanRenderer::~VulkanRenderer() {
    if (this->surface != VK_NULL_HANDLE) vkDestroySurfaceKHR(this->instance, this->surface, nullptr);
    if (this->device != VK_NULL_HANDLE) vkDestroyDevice(this->device, nullptr);
    if (this->instance != VK_NULL_HANDLE) vkDestroyInstance(this->instance, nullptr);
}

bool VulkanRenderer::init_vulkan(const WindowHandles& handles) {
    createInstance();
    createSurface(handles);
    pickPhysicalDevice();
    createLogicalDevice();

    return true;
}

void VulkanRenderer::createInstance() {
    if (enableValidationLayers && !checkValidationLayerSupport()) {
        throw std::runtime_error("validation layers requested, but not available!");
    }
    VkApplicationInfo appInfo{};
    appInfo.sType = VK_STRUCTURE_TYPE_APPLICATION_INFO;
    appInfo.pApplicationName = "Hello Triangle";
    appInfo.applicationVersion = VK_MAKE_VERSION(1, 0, 0);
    appInfo.pEngineName = nullptr;
    appInfo.engineVersion = VK_MAKE_VERSION(1, 0, 0);
    appInfo.apiVersion = VK_API_VERSION_1_4;

    VkInstanceCreateInfo сreateInfo{};
    сreateInfo.sType = VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO;
    сreateInfo.pApplicationInfo = &appInfo;
    сreateInfo.enabledExtensionCount = static_cast<uint32_t>(extensions.size());
    сreateInfo.ppEnabledExtensionNames = extensions.data();
    if (enableValidationLayers) {
        сreateInfo.enabledLayerCount = static_cast<uint32_t>(validationLayers.size());
        сreateInfo.ppEnabledLayerNames = validationLayers.data();
    } else {
        сreateInfo.enabledLayerCount = 0;
    }

    VkResult instanceResult = vkCreateInstance(&сreateInfo, nullptr, &this->instance);
    if (instanceResult != VK_SUCCESS) {
        throw std::runtime_error("failed to create instance!");
    }
};

QueueFamilyIndices findQueueFamilies(VkPhysicalDevice device) {
    QueueFamilyIndices indices;

    uint32_t queueFamilyCount = 0;
    vkGetPhysicalDeviceQueueFamilyProperties(device, &queueFamilyCount, nullptr);

    std::vector<VkQueueFamilyProperties> queueFamilies(queueFamilyCount);
    vkGetPhysicalDeviceQueueFamilyProperties(device, &queueFamilyCount, queueFamilies.data());

    int i = 0;
    for (const auto& queueFamily : queueFamilies) {
        if (indices.isComplete()) break;

        if (queueFamily.queueFlags & VK_QUEUE_GRAPHICS_BIT) {
            indices.graphicsFamily = i;
        }

        i++;
    }

    return indices;
}

bool isDeviceSuitable(VkPhysicalDevice device) {
    QueueFamilyIndices indices = findQueueFamilies(device);

    return indices.isComplete();
}

void VulkanRenderer::pickPhysicalDevice() {
    uint32_t deviceCount = 0;
    vkEnumeratePhysicalDevices(this->instance, &deviceCount, nullptr);
    if (deviceCount == 0) {
        throw std::runtime_error("failed to find GPUs with Vulkan support!");
    }

    std::vector<VkPhysicalDevice> devices(deviceCount);
    vkEnumeratePhysicalDevices(instance, &deviceCount, devices.data());

    for (const auto& device : devices) {
        if (isDeviceSuitable(device)) {
            this->physicalDevice = device;
            break;
        }
    }

    if (this->physicalDevice == VK_NULL_HANDLE) {
        throw std::runtime_error("failed to find a suitable GPU!");
    }
};

void VulkanRenderer::createLogicalDevice() {
    QueueFamilyIndices indices = findQueueFamilies(this->physicalDevice);

    VkDeviceQueueCreateInfo queueCreateInfo{};
    queueCreateInfo.sType = VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO;
    queueCreateInfo.queueFamilyIndex = indices.graphicsFamily.value();
    queueCreateInfo.queueCount = 1;

    float queuePriority = 1.0f;
    queueCreateInfo.pQueuePriorities = &queuePriority;

    VkPhysicalDeviceFeatures deviceFeatures{};
    VkDeviceCreateInfo createInfo{};
    createInfo.sType = VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO;
    createInfo.pQueueCreateInfos = &queueCreateInfo;
    createInfo.queueCreateInfoCount = 1;
    createInfo.pEnabledFeatures = &deviceFeatures;

    VkResult deviceResult = vkCreateDevice(this->physicalDevice, &createInfo, nullptr, &this->device);
    if (deviceResult != VK_SUCCESS) {
        throw std::runtime_error("failed to create logical device!");
    }

    vkGetDeviceQueue(this->device, indices.graphicsFamily.value(), 0, &this->graphicsQueue);
};

void VulkanRenderer::createSurface(const WindowHandles& handles) {
    VkWaylandSurfaceCreateInfoKHR сreateInfo{};
    сreateInfo.sType = VK_STRUCTURE_TYPE_WAYLAND_SURFACE_CREATE_INFO_KHR;
    сreateInfo.display = reinterpret_cast<struct wl_display*>(handles.display_ptr);
    сreateInfo.surface = reinterpret_cast<struct wl_surface*>(handles.window_ptr);
    
    VkResult surfaceResult = vkCreateWaylandSurfaceKHR(this->instance, &сreateInfo, nullptr, &this->surface);
    if (surfaceResult != VK_SUCCESS) {
        throw std::runtime_error("failed to create surface!");
    }
};
