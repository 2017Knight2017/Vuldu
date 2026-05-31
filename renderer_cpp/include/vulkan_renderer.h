#pragma once
#include <vulkan/vulkan.h>
#include <memory>
//#include "doom/src/FFI.rs.h"

struct WindowHandles;

class VulkanRenderer {
public:
    VulkanRenderer(uint32_t width, uint32_t height);
    ~VulkanRenderer();
    bool init_vulkan(const WindowHandles& handles);

private:
    VkInstance instance;
    VkSurfaceKHR surface;
    VkPhysicalDevice physicalDevice;
    VkDevice device;
    VkQueue graphicsQueue;
    void createInstance();
    void createSurface(const WindowHandles& handles);
    void pickPhysicalDevice();
    void createLogicalDevice();
};

std::unique_ptr<VulkanRenderer> create_renderer(uint32_t width, uint32_t height);