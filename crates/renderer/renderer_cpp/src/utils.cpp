#include <stdexcept>
#include <span>
#include <vector>
#include "utils.h"

uint32_t findMemoryType(vk::PhysicalDevice physicalDevice, uint32_t typeFilter, vk::MemoryPropertyFlags properties) {
	vk::PhysicalDeviceMemoryProperties memProperties = physicalDevice.getMemoryProperties();
	for (uint32_t i = 0; i < memProperties.memoryTypeCount; i++) {
		if ((typeFilter & (1 << i)) && (memProperties.memoryTypes[i].propertyFlags & properties) == properties) {
			return i;
		}
	}

	throw std::runtime_error("failed to find suitable memory type!");
}

vk::raii::ImageView createImageView(
	const vk::raii::Device& device, 
	vk::Image image, 
	vk::Format format, 
	vk::ImageAspectFlags aspectFlags
) {
	vk::ImageViewCreateInfo viewInfo;
	viewInfo
		.setImage(image)
		.setViewType(vk::ImageViewType::e2D)
		.setFormat(format)
		.setSubresourceRange({
			aspectFlags,
			0,
			1,
			0,
			1
		});

	return device.createImageView(viewInfo);
}

using Access = vk::AccessFlagBits2;
using Stage = vk::PipelineStageFlagBits2;

vk::ImageMemoryBarrier2 getBarrier(
	vk::ImageLayout oldLayout, 
	vk::ImageLayout newLayout
) noexcept {
	vk::ImageMemoryBarrier2 barrier;
	barrier
		.setOldLayout(oldLayout)
		.setNewLayout(newLayout)
		.setSubresourceRange({
			newLayout == vk::ImageLayout::eDepthStencilAttachmentOptimal 
			? vk::ImageAspectFlagBits::eDepth 
			: vk::ImageAspectFlagBits::eColor,
			0,
			1,
			0,
			1
		});

	if (oldLayout == vk::ImageLayout::eUndefined && newLayout == vk::ImageLayout::eColorAttachmentOptimal) {
		barrier
			.setSrcAccessMask(Access::eNoneKHR)
			.setDstAccessMask(Access::eColorAttachmentWrite)
			.setSrcStageMask(Stage::eNoneKHR)
			.setDstStageMask(Stage::eColorAttachmentOutput);
	} else if (oldLayout == vk::ImageLayout::eColorAttachmentOptimal && newLayout == vk::ImageLayout::ePresentSrcKHR) {
		barrier
			.setSrcAccessMask(Access::eColorAttachmentWrite)
			.setDstAccessMask(Access::eNoneKHR)
			.setSrcStageMask(Stage::eColorAttachmentOutput)
			.setDstStageMask(Stage::eBottomOfPipe);
	} else if (oldLayout == vk::ImageLayout::eUndefined && newLayout == vk::ImageLayout::eDepthStencilAttachmentOptimal) {
		barrier
			.setSrcAccessMask(Access::eNoneKHR)
			.setDstAccessMask(Access::eDepthStencilAttachmentRead | Access::eDepthStencilAttachmentWrite)
			.setSrcStageMask(Stage::eNoneKHR)
			.setDstStageMask(Stage::eEarlyFragmentTests | Stage::eLateFragmentTests);
	} else if (oldLayout == vk::ImageLayout::eTransferDstOptimal && newLayout == vk::ImageLayout::eShaderReadOnlyOptimal) {
		barrier
			.setSrcAccessMask(Access::eTransferWrite)
			.setDstAccessMask(Access::eShaderRead)
			.setSrcStageMask(Stage::eTransfer)
			.setDstStageMask(Stage::eFragmentShader);
	}

	return barrier;
} 

void changeSingleImageLayout(
	vk::CommandBuffer commandBuffer, 
	vk::ImageLayout oldLayout, 
	vk::ImageLayout newLayout, 
	vk::Image image
) noexcept {
	vk::ImageMemoryBarrier2 barrier = getBarrier(oldLayout, newLayout);
	barrier.setImage(image);

	vk::DependencyInfo dependencyInfo;
	dependencyInfo.setImageMemoryBarriers(barrier);

	commandBuffer.pipelineBarrier2(dependencyInfo);
}

void changeMultipleImageLayout(
	vk::CommandBuffer commandBuffer, 
	vk::ImageLayout oldLayout, 
	vk::ImageLayout newLayout, 
	std::span<const vk::raii::Image> images
) {
	vk::ImageMemoryBarrier2 templateBarrier = getBarrier(oldLayout, newLayout);

	std::vector<vk::ImageMemoryBarrier2> barriers;
	barriers.reserve(images.size());

	for (const auto& img : images) {
		vk::ImageMemoryBarrier2 barrier = templateBarrier;
		barrier.setImage(img);
		barriers.push_back(barrier);
	}

	vk::DependencyInfo dependencyInfo;
	dependencyInfo.setImageMemoryBarriers(barriers);

	commandBuffer.pipelineBarrier2(dependencyInfo);
}
