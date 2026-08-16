#pragma once
#include <vulkan/vulkan_core.h>
#include <span>

VkShaderModule createShaderModule(VkDevice device, std::span<const uint32_t> code);
uint32_t findMemoryType(VkPhysicalDevice physicalDevice, uint32_t typeFilter, VkMemoryPropertyFlags properties);
void createImageView(VkDevice device, VkImage image, VkFormat format, VkImageAspectFlags aspectFlags, VkImageView* dstView);
void changeImageLayout(
	VkCommandBuffer currentCommandBuffer, 
	VkImageLayout oldLayout, 
	VkImageLayout newLayout, 
	std::span<const VkImage> images
);

template<typename T, typename F>
inline void destroyResource(VkDevice device, T& resource, F destroyFunction) {
    if (resource != VK_NULL_HANDLE) {
        destroyFunction(device, resource, nullptr);
        resource = VK_NULL_HANDLE;
    }
}
