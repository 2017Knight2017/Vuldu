#pragma once
#include <vulkan/vulkan.h>
#include <memory>
#include <vector>
//#include "doom/src/FFI.rs.h"

struct WindowHandles;

class VulkanRenderer {
public:
    VulkanRenderer();
    ~VulkanRenderer();
    void initVulkan(const WindowHandles& handles, size_t window_raw_ptr);

private:
    VkInstance instance;
    VkSurfaceKHR surface;
    VkPhysicalDevice physicalDevice;
    VkDevice device;
    VkQueue graphicsQueue;
    VkQueue presentQueue;
    VkSwapchainKHR swapChain;
    VkFormat swapChainImageFormat;
    VkExtent2D swapChainExtent;
    std::vector<VkImage> swapChainImages;
    size_t m_window_raw_ptr;
    void createInstance();
    void createSurface(const WindowHandles& handles);
    void pickPhysicalDevice();
    void createLogicalDevice();
    void createSwapChain();
};

std::unique_ptr<VulkanRenderer> create_renderer();