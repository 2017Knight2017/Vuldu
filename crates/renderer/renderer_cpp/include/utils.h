#pragma once
#include <vulkan/vulkan_raii.hpp>
#include <span>

uint32_t findMemoryType(vk::PhysicalDevice physicalDevice, uint32_t typeFilter, vk::MemoryPropertyFlags properties);
vk::raii::ImageView createImageView(
	const vk::raii::Device& device, 
	vk::Image image, 
	vk::Format format, 
	vk::ImageAspectFlags aspectFlags
);
void changeSingleImageLayout(
	vk::CommandBuffer currentCommandBuffer, 
	vk::ImageLayout oldLayout, 
	vk::ImageLayout newLayout, 
	vk::Image image
) noexcept;
void changeMultipleImageLayout(
	vk::CommandBuffer currentCommandBuffer, 
	vk::ImageLayout oldLayout, 
	vk::ImageLayout newLayout, 
	std::span<const vk::raii::Image> images
);
