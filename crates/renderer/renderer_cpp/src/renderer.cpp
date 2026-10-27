#include "renderer.h"

std::unique_ptr<VulkanRenderer> createRenderer() {
	return std::make_unique<VulkanRenderer>();
}

VulkanRenderer::VulkanRenderer() {}
VulkanRenderer::~VulkanRenderer() {
	if (this->device != VK_NULL_HANDLE) {
		this->device.waitIdle();
	}
}

void VulkanRenderer::initVulkan(const WindowHandles& handles, uint32_t width, uint32_t height) {
	createInstance(handles);
	setupDebugMessenger();
	createSurface(handles);
	pickPhysicalDevice();
	createLogicalDevice();
	createSwapChain(width, height);
	createDescriptorSetLayout();
	createPipelines();
	createMVPBuffer();
	createInstanceBuffers();
	createDescriptorPool();
	createDescriptorSets(); 
	createCommandPool();
	createDepthResources();
	createTextureSamplers();
	createCommandBuffers();
	createSyncObjects();
}

void VulkanRenderer::recreateSwapChain(uint32_t width, uint32_t height) {
	this->device.waitIdle();

	swapchainRes.reset();

	createSwapChain(width, height);
	createDepthResources();

	this->currentFrame = 0;
}
