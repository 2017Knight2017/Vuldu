#pragma once
#include <vulkan/vulkan_core.h>
#include <vector>
#include <string>

std::vector<char> readFile(const std::string& filename);
VkShaderModule createShaderModule(VkDevice device, const std::vector<char>& code);
uint32_t findMemoryType(VkPhysicalDevice physicalDevice, uint32_t typeFilter, VkMemoryPropertyFlags properties);
VkImageView createImageView(VkDevice device, VkImage image, VkFormat format, VkImageAspectFlags aspectFlags);

template<typename T, typename F>
inline void destroyResource(VkDevice device, T& resource, F destroyFunction) {
    if (resource != VK_NULL_HANDLE) {
        destroyFunction(device, resource, nullptr);
        resource = VK_NULL_HANDLE;
    }
}
