#pragma once
#include <vulkan/vulkan_core.h>
#include <vector>
#include <string>

std::vector<char> readFile(const std::string& filename);
VkShaderModule createShaderModule(VkDevice device, const std::vector<char>& code);
uint32_t findMemoryType(VkPhysicalDevice physicalDevice, uint32_t typeFilter, VkMemoryPropertyFlags properties);
VkImageView createImageView(VkDevice device, VkImage image, uint32_t mipLevels, VkFormat format, VkImageAspectFlags aspectFlags);
void generateMipmaps(VkCommandBuffer commandBuffer, VkImage image, uint32_t width, uint32_t height, uint32_t mipLevels);

template<typename T, typename F>
inline void destroyResource(VkDevice device, T& resource, F destroyFunction) {
    if (resource != VK_NULL_HANDLE) {
        destroyFunction(device, resource, nullptr);
        resource = VK_NULL_HANDLE;
    }
}
