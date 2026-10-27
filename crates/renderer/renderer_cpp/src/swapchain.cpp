#include <limits>
#include <algorithm>
#include "renderer.h"
#include "renderer/src/bridge.rs.h"

vk::SurfaceFormatKHR chooseSwapSurfaceFormat(const std::vector<vk::SurfaceFormatKHR>& availableFormats) {
	for (const auto& availableFormat : availableFormats) {
		if (availableFormat.format == vk::Format::eB8G8R8A8Srgb && availableFormat.colorSpace == vk::ColorSpaceKHR::eSrgbNonlinear) {
			return availableFormat;
		}
	}

	return availableFormats[0];
}

vk::PresentModeKHR chooseSwapPresentMode(const std::vector<vk::PresentModeKHR>& availablePresentModes) {
	for (const auto& availablePresentMode : availablePresentModes) {
		if (availablePresentMode == vk::PresentModeKHR::eMailbox) {
			return availablePresentMode;
		}
	}

	return vk::PresentModeKHR::eFifo;
}

vk::Extent2D chooseSwapExtent(const vk::SurfaceCapabilitiesKHR& capabilities, uint32_t width, uint32_t height) {
	if (capabilities.currentExtent.width != std::numeric_limits<uint32_t>::max()) {
		return capabilities.currentExtent;
	}

	vk::Extent2D actualExtent = {
		std::clamp(width, capabilities.minImageExtent.width, capabilities.maxImageExtent.width),
		std::clamp(height, capabilities.minImageExtent.height, capabilities.maxImageExtent.height)
	};

	return actualExtent;
}

void VulkanRenderer::createSwapChain(uint32_t width, uint32_t height) {
	SwapChainSupportDetails swapchainSupport = querySwapChainSupport(this->physicalDevice, this->surface);

	vk::SurfaceFormatKHR surfaceFormat = chooseSwapSurfaceFormat(swapchainSupport.formats);
	vk::PresentModeKHR presentMode = chooseSwapPresentMode(swapchainSupport.presentModes);
	vk::Extent2D extent = chooseSwapExtent(swapchainSupport.capabilities, width, height);

	uint32_t imageCount = swapchainSupport.capabilities.minImageCount + 1;
	if (swapchainSupport.capabilities.maxImageCount > 0 && imageCount > swapchainSupport.capabilities.maxImageCount) {
		imageCount = swapchainSupport.capabilities.maxImageCount;
	}

	vk::SwapchainCreateInfoKHR createInfo{
		{},
		this->surface,
		imageCount,
		surfaceFormat.format,
		surfaceFormat.colorSpace,
		extent,
		1,
		vk::ImageUsageFlagBits::eColorAttachment,
		{},
		{},
		{},
		swapchainSupport.capabilities.currentTransform,
		vk::CompositeAlphaFlagBitsKHR::eOpaque,
		presentMode,
		VK_TRUE
	};	

	QueueFamilyIndices indices = findQueueFamilies(this->physicalDevice, this->surface);
	uint32_t queueFamilyIndices[] = {indices.graphicsFamily.value(), indices.presentFamily.value()};

	if (indices.graphicsFamily != indices.presentFamily) {
		createInfo
			.setImageSharingMode(vk::SharingMode::eConcurrent)
			.setQueueFamilyIndices(queueFamilyIndices);
	} else {
		createInfo.setImageSharingMode(vk::SharingMode::eExclusive);
	}

	vk::raii::SwapchainKHR swapchain{this->device, createInfo};
	std::vector<vk::Image> swapchainImages = swapchain.getImages();

	std::vector<vk::raii::ImageView> swapchainImageViews;
	swapchainImageViews.reserve(swapchainImages.size());

	for (vk::Image image : swapchainImages) {
		swapchainImageViews.push_back(createImageView(
			this->device, 
			image, 
			surfaceFormat.format, 
			vk::ImageAspectFlagBits::eColor
		));
	}

	this->swapchainRes = SwapchainResources{
		std::move(swapchain),
		surfaceFormat.format,
		extent,
		std::move(swapchainImages),
		std::move(swapchainImageViews)
	};
}
