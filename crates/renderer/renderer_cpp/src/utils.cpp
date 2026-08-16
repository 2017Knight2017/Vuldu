#include <stdexcept>
#include <span>
#include <vector>
#include "utils.h"

VkShaderModule createShaderModule(VkDevice device, std::span<const uint32_t> code) {
	VkShaderModuleCreateInfo createInfo{};
	createInfo.sType = VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO;
	createInfo.codeSize = code.size_bytes();
	createInfo.pCode = code.data();
	
	VkShaderModule shaderModule;
	VkResult shaderModuleResult = vkCreateShaderModule(device, &createInfo, nullptr, &shaderModule);
	if (shaderModuleResult != VK_SUCCESS) {
	    throw std::runtime_error("failed to create shader module!");
	}

	return shaderModule;
}

uint32_t findMemoryType(VkPhysicalDevice physicalDevice, uint32_t typeFilter, VkMemoryPropertyFlags properties) {
	VkPhysicalDeviceMemoryProperties memProperties;
	vkGetPhysicalDeviceMemoryProperties(physicalDevice, &memProperties);
	for (uint32_t i = 0; i < memProperties.memoryTypeCount; i++) {
	    if ((typeFilter & (1 << i)) && (memProperties.memoryTypes[i].propertyFlags & properties) == properties) {
	        return i;
	    }
	}

	throw std::runtime_error("failed to find suitable memory type!");
}

void createImageView(
	VkDevice device, 
	VkImage image, 
	VkFormat format, 
	VkImageAspectFlags aspectFlags,
	VkImageView* dstView
) {
    VkImageViewCreateInfo viewInfo{};
    viewInfo.sType = VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO;
    viewInfo.image = image;
    viewInfo.viewType = VK_IMAGE_VIEW_TYPE_2D;
    viewInfo.format = format;
    viewInfo.subresourceRange.aspectMask = aspectFlags;
    viewInfo.subresourceRange.baseMipLevel = 0;
    viewInfo.subresourceRange.levelCount = 1;
    viewInfo.subresourceRange.baseArrayLayer = 0;
    viewInfo.subresourceRange.layerCount = 1;

    if (vkCreateImageView(device, &viewInfo, nullptr, dstView) != VK_SUCCESS) {
        throw std::runtime_error("failed to create image view!");
    }
}

VkImageMemoryBarrier2 getBarrier(
	VkImageLayout oldLayout, 
	VkImageLayout newLayout
) {
	VkImageMemoryBarrier2 barrier{};
	barrier.sType = VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER_2;
	barrier.oldLayout = oldLayout;
	barrier.newLayout = newLayout;
	barrier.subresourceRange.baseMipLevel = 0;
	barrier.subresourceRange.levelCount = 1;
	barrier.subresourceRange.baseArrayLayer = 0;
	barrier.subresourceRange.layerCount = 1;

	if (newLayout == VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL) {
		barrier.subresourceRange.aspectMask = VK_IMAGE_ASPECT_DEPTH_BIT;
	} else {
		barrier.subresourceRange.aspectMask = VK_IMAGE_ASPECT_COLOR_BIT;
	}

	if (oldLayout == VK_IMAGE_LAYOUT_UNDEFINED && newLayout == VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL) {
		barrier.srcAccessMask = VK_ACCESS_2_NONE_KHR;
		barrier.dstAccessMask = VK_ACCESS_2_COLOR_ATTACHMENT_WRITE_BIT;

		barrier.srcStageMask = VK_PIPELINE_STAGE_2_NONE_KHR;
		barrier.dstStageMask = VK_PIPELINE_STAGE_2_COLOR_ATTACHMENT_OUTPUT_BIT;
	} else if (oldLayout == VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL && newLayout == VK_IMAGE_LAYOUT_PRESENT_SRC_KHR) {
		barrier.srcAccessMask = VK_ACCESS_2_COLOR_ATTACHMENT_WRITE_BIT;
		barrier.dstAccessMask = VK_ACCESS_2_NONE_KHR;

		barrier.srcStageMask = VK_PIPELINE_STAGE_2_COLOR_ATTACHMENT_OUTPUT_BIT;
		barrier.dstStageMask = VK_PIPELINE_STAGE_2_BOTTOM_OF_PIPE_BIT;
	} else if (oldLayout == VK_IMAGE_LAYOUT_UNDEFINED && newLayout == VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL) {
        barrier.srcAccessMask = VK_ACCESS_2_NONE_KHR;
        barrier.dstAccessMask = VK_ACCESS_2_DEPTH_STENCIL_ATTACHMENT_READ_BIT | VK_ACCESS_2_DEPTH_STENCIL_ATTACHMENT_WRITE_BIT;

        barrier.srcStageMask = VK_PIPELINE_STAGE_2_NONE_KHR;
        barrier.dstStageMask = VK_PIPELINE_STAGE_2_EARLY_FRAGMENT_TESTS_BIT | VK_PIPELINE_STAGE_2_LATE_FRAGMENT_TESTS_BIT;
    } else if (oldLayout == VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL && newLayout == VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL) {
		barrier.srcAccessMask = VK_ACCESS_2_TRANSFER_WRITE_BIT;
		barrier.dstAccessMask = VK_ACCESS_2_SHADER_READ_BIT;

		barrier.srcStageMask = VK_PIPELINE_STAGE_2_TRANSFER_BIT;
		barrier.dstStageMask = VK_PIPELINE_STAGE_2_FRAGMENT_SHADER_BIT;
	}

	return barrier;
}

void changeImageLayout(
	VkCommandBuffer commandBuffer, 
	VkImageLayout oldLayout, 
	VkImageLayout newLayout, 
	std::span<const VkImage> images
) {
	VkImageMemoryBarrier2 templateBarrier = getBarrier(oldLayout, newLayout);

	std::vector<VkImageMemoryBarrier2> barriers;
    barriers.reserve(images.size());

	for (VkImage img : images) {
        VkImageMemoryBarrier2 barrier = templateBarrier;
        barrier.image = img;
        barriers.push_back(barrier);
    }

	VkDependencyInfo dependencyInfo{};
	dependencyInfo.sType = VK_STRUCTURE_TYPE_DEPENDENCY_INFO;
	dependencyInfo.imageMemoryBarrierCount = barriers.size();
	dependencyInfo.pImageMemoryBarriers = barriers.data();

	vkCmdPipelineBarrier2(commandBuffer, &dependencyInfo);
} 
