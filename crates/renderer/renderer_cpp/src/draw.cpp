#include <cstring>
#include <stdexcept>
#include "renderer.h" 
#include "renderer/src/bridge.rs.h"

void VulkanRenderer::createCommandPool() {
	QueueFamilyIndices queueFamilyIndices = findQueueFamilies(this->physicalDevice, this->surface);

	vk::CommandPoolCreateInfo poolInfo;
	poolInfo
		.setFlags(vk::CommandPoolCreateFlagBits::eResetCommandBuffer)
		.setQueueFamilyIndex(queueFamilyIndices.graphicsFamily.value());

	this->commandPool = this->device.createCommandPool(poolInfo);
}

void VulkanRenderer::createCommandBuffers() {
	vk::CommandBufferAllocateInfo allocInfo{
		this->commandPool,
		vk::CommandBufferLevel::ePrimary,
		MAX_FRAMES_IN_FLIGHT
	};

	auto commandBuffers = this->device.allocateCommandBuffers(allocInfo);

	for (uint32_t i = 0; i < MAX_FRAMES_IN_FLIGHT; ++i) {
		this->commandBuffers[i] = std::move(commandBuffers[i]);
	}
}

void VulkanRenderer::createSyncObjects() {
	size_t imageCount = this->swapchainRes->images.size();

	this->renderFinishedSemaphores.reserve(imageCount);

	for (size_t i = 0; i < imageCount; ++i) {
		this->renderFinishedSemaphores.emplace_back(this->device, vk::SemaphoreCreateInfo{});
	}

	for (size_t i = 0; i < MAX_FRAMES_IN_FLIGHT; ++i) {
		this->imageAvailableSemaphores[i] = this->device.createSemaphore({});
		this->inFlightFences[i] = this->device.createFence({vk::FenceCreateFlagBits::eSignaled});
	}
}

void VulkanRenderer::updateObjectInstances(rust::Slice<const ObjectInstance> instances) {
	if (instances.empty() || instances.size() > MAX_OBJECTS) return;

	this->objectInst.activeCount = static_cast<uint32_t>(instances.size());

	vk::DeviceSize instanceBufferSize = sizeof(ObjectInstance) * instances.size();
	memcpy(this->objectInst.mapped[this->currentFrame], instances.data(), instanceBufferSize);   
}

void VulkanRenderer::updateUiInstances(rust::Slice<const UiInstance> instances) {
	if (instances.empty() || instances.size() > MAX_UI) return;

	this->uiInst.activeCount = static_cast<uint32_t>(instances.size());

	vk::DeviceSize instanceBufferSize = sizeof(UiInstance) * instances.size();
	memcpy(this->uiInst.mapped[this->currentFrame], instances.data(), instanceBufferSize);   
}

void VulkanRenderer::updateWeaponInstance(const UiInstance& instance) {
	memcpy(this->weaponInst.mapped[this->currentFrame], &instance, sizeof(UiInstance));   
}

void VulkanRenderer::setPaletteIndex(uint32_t idx) {
	this->currentPaletteIndex = idx % MAX_PAL;
}

uint32_t VulkanRenderer::getPaletteIndex() {
	return this->currentPaletteIndex;
}

void VulkanRenderer::setSkyIndex(uint32_t idx) {
	this->currentSkyIndex = idx % MAX_SKY;
}

void VulkanRenderer::setGlobalTimer(uint32_t global_timer) {
	this->globalTimer = static_cast<float>(global_timer);
}

void VulkanRenderer::setCameraYaw(float camera_yaw) {
	this->cameraYaw = camera_yaw;
}

void VulkanRenderer::setFlags(uint32_t flags_to_invert) {
	this->flags ^= flags_to_invert;
}

size_t getMaxSky() {
	return MAX_SKY;
}

size_t getAnimInfoSize() {
	return ANIM_INFO_SIZE;
}

void VulkanRenderer::startFrame(const MVP& mvp) {
	if (!this->swapchainRes.has_value()) return;
	vk::CommandBuffer currentCommandBuffer = this->commandBuffers[this->currentFrame];

	vk::Result waitResult = this->device.waitForFences(
		*this->inFlightFences[this->currentFrame], 
		VK_TRUE, 
		std::numeric_limits<uint64_t>::max()
	);

	if (waitResult != vk::Result::eSuccess) {
		throw std::runtime_error("failed to acquire swapchain image!");
	}

	this->currentImageIndex = this->swapchainRes->swapchain.acquireNextImage(
		std::numeric_limits<uint64_t>::max(), 
		this->imageAvailableSemaphores[this->currentFrame], 
		VK_NULL_HANDLE
	).value;

	memcpy(this->MVPBuffersMapped[this->currentFrame], &mvp, sizeof(MVP));

	this->device.resetFences(*this->inFlightFences[this->currentFrame]);
	currentCommandBuffer.reset({});

	currentCommandBuffer.begin({
		vk::CommandBufferUsageFlagBits::eOneTimeSubmit
	});

	changeSingleImageLayout(
		currentCommandBuffer, 
		vk::ImageLayout::eUndefined, 
		vk::ImageLayout::eDepthStencilAttachmentOptimal,
		this->swapchainRes->depthImage 
	);
	changeSingleImageLayout(
		currentCommandBuffer, 
		vk::ImageLayout::eUndefined, 
		vk::ImageLayout::eColorAttachmentOptimal,
		this->swapchainRes->images[this->currentImageIndex]
	);
	beginRendering(currentCommandBuffer);

	vk::Viewport viewport;
	viewport
		.setWidth(static_cast<float>(this->swapchainRes->extent.width))
		.setHeight(static_cast<float>(this->swapchainRes->extent.height))
		.setMinDepth(0.0f)
		.setMaxDepth(1.0f);
	
	currentCommandBuffer.setViewport(0, viewport);

	vk::Rect2D scissor;
	scissor.setExtent(this->swapchainRes->extent);
	
	currentCommandBuffer.setScissor(0, scissor);
}

void VulkanRenderer::beginRendering(vk::CommandBuffer currentCommandBuffer) noexcept {
	vk::ClearValue clearColor{{0.1f, 0.1f, 0.1f, 1.0f}};
	vk::ClearValue clearDepth{{1.0f, 0}};

	vk::RenderingAttachmentInfo depthAttachment;
	depthAttachment
		.setImageView(*this->swapchainRes->depthImageView)
		.setImageLayout(vk::ImageLayout::eDepthStencilAttachmentOptimal)
		.setLoadOp(vk::AttachmentLoadOp::eClear)
		.setClearValue(clearDepth);

	vk::RenderingAttachmentInfo colorAttachment;
	colorAttachment
		.setImageView(*this->swapchainRes->imageViews[this->currentImageIndex])
		.setImageLayout(vk::ImageLayout::eColorAttachmentOptimal)
		.setLoadOp(vk::AttachmentLoadOp::eClear)
		.setClearValue(clearColor);

	vk::RenderingInfo renderingInfo{
		{},
		{{}, this->swapchainRes->extent},
		1,
		0,
		colorAttachment,
		&depthAttachment,
	};

	currentCommandBuffer.beginRendering(renderingInfo);
}

void VulkanRenderer::draw(
	PipelineType pipelineType,
	const MeshBuffers& mesh,
	const InstanceBuffers& inst,
	const void* constants,
	uint32_t constantsSize
) noexcept {
	vk::CommandBuffer currentCommandBuffer = this->commandBuffers[this->currentFrame];
	
	currentCommandBuffer.bindPipeline(
		vk::PipelineBindPoint::eGraphics, 
		this->pipelines[pipelineType]
	);
		
	currentCommandBuffer.bindDescriptorSets(
		vk::PipelineBindPoint::eGraphics, 
		this->pipelineLayouts[pipelineType], 
		0, 
		*this->descriptorSets[this->currentFrame], 
		{}
	);

	currentCommandBuffer.bindVertexBuffers(
		0, 
		{mesh.vertexBuffer, inst.buffers[this->currentFrame]}, 
		{0, 0}
	);
	
	currentCommandBuffer.bindIndexBuffer(mesh.indexBuffer, 0, vk::IndexType::eUint32);

	currentCommandBuffer.pushConstants(
		this->pipelineLayouts[pipelineType],
		SHADER_STAGE_FLAGS[pipelineType],
		0,
		constantsSize,
		constants
	);

	currentCommandBuffer.drawIndexed(
		mesh.indexCount, 
		inst.activeCount, 
		0, 
		0, 
		0
	);
}

void VulkanRenderer::draw(
	PipelineType pipelineType,
	const MeshBuffers& mesh,
	const void* constants,
	uint32_t constantsSize
) noexcept {
	vk::CommandBuffer currentCommandBuffer = this->commandBuffers[this->currentFrame];
	
	currentCommandBuffer.bindPipeline(
		vk::PipelineBindPoint::eGraphics, 
		this->pipelines[pipelineType]
	);
		
	currentCommandBuffer.bindDescriptorSets(
		vk::PipelineBindPoint::eGraphics, 
		this->pipelineLayouts[pipelineType], 
		0, 
		*this->descriptorSets[this->currentFrame], 
		{}
	);

	currentCommandBuffer.bindVertexBuffers(0, *mesh.vertexBuffer, {0});
	currentCommandBuffer.bindIndexBuffer(mesh.indexBuffer, 0, vk::IndexType::eUint32);

	currentCommandBuffer.pushConstants(
		this->pipelineLayouts[pipelineType],
		SHADER_STAGE_FLAGS[pipelineType],
		0,
		constantsSize,
		constants
	);

	currentCommandBuffer.drawIndexed(
		mesh.indexCount, 
		1, 
		0, 
		0, 
		0
	);
}

void VulkanRenderer::drawLevel() noexcept {
	if (!this->swapchainRes.has_value()) return;
	if (this->levelMesh.vertexBuffer == VK_NULL_HANDLE || this->levelMesh.indexCount == 0) return;

	LevelPushConstants constants{
		{
			static_cast<float>(this->swapchainRes->extent.width),
			static_cast<float>(this->swapchainRes->extent.height)
		},
		this->currentPaletteIndex,
		this->currentSkyIndex,
		PIXELS_IN_PANORAMA / this->skyWidths[this->currentSkyIndex],
		this->globalTimer,
		this->cameraYaw,
		this->flags
	};

	draw(
		PipelineType::Level, 
		this->levelMesh, 
		&constants, 
		sizeof(constants) 
	);
}

void VulkanRenderer::drawObjects() noexcept {
	if (!this->swapchainRes.has_value()) return;
	if (this->objectMesh.vertexBuffer == VK_NULL_HANDLE || this->objectMesh.indexCount == 0) return;
	
	ObjectPushConstants constants{
		this->currentPaletteIndex,
		this->flags
	};

	draw(
		PipelineType::Object, 
		this->objectMesh, 
		this->objectInst,
		&constants, 
		sizeof(constants) 
	);
}

void VulkanRenderer::drawUi() noexcept {
	if (!this->swapchainRes.has_value()) return;
	if (this->uiMesh.vertexBuffer == VK_NULL_HANDLE || this->uiMesh.indexCount == 0) return;

	draw(
		PipelineType::Ui, 
		this->uiMesh, 
		this->uiInst,
		&this->currentPaletteIndex, 
		sizeof(uint32_t) 
	);
}

void VulkanRenderer::drawWeapon() noexcept {
	if (!this->swapchainRes.has_value()) return;
	if (this->weaponMesh.vertexBuffer == VK_NULL_HANDLE || this->weaponMesh.indexCount == 0) return;

	draw(
		PipelineType::Weapon, 
		this->weaponMesh, 
		this->weaponInst,
		&this->currentPaletteIndex, 
		sizeof(uint32_t) 
	);
}

void VulkanRenderer::endFrame() {
	if (!this->swapchainRes.has_value()) return;

	vk::CommandBuffer currentCommandBuffer = this->commandBuffers[this->currentFrame];

	currentCommandBuffer.endRendering();
	
	changeSingleImageLayout(
		currentCommandBuffer, 
		vk::ImageLayout::eAttachmentOptimal, 
		vk::ImageLayout::ePresentSrcKHR,
		this->swapchainRes->images[this->currentImageIndex] 
	);

	currentCommandBuffer.end();

	vk::PipelineStageFlags waitStages = vk::PipelineStageFlagBits::eColorAttachmentOutput;
	vk::SubmitInfo submitInfo{
		*this->imageAvailableSemaphores[this->currentFrame],
		waitStages,
		currentCommandBuffer,
		*this->renderFinishedSemaphores[this->currentImageIndex]
	};

	this->device.resetFences(*this->inFlightFences[this->currentFrame]);
	this->graphicsQueue.submit(submitInfo, this->inFlightFences[this->currentFrame]);

	vk::PresentInfoKHR presentInfo{
		*this->renderFinishedSemaphores[this->currentImageIndex],
		*this->swapchainRes->swapchain,
		this->currentImageIndex
	};

	vk::Result presentResult = this->presentQueue.presentKHR(presentInfo);

	if (presentResult != vk::Result::eSuccess &&
		presentResult != vk::Result::eErrorOutOfDateKHR &&
		presentResult != vk::Result::eSuboptimalKHR
	) {
		throw std::runtime_error("failed to present swap chain image!");
	}
	
	this->currentFrame = (this->currentFrame + 1) % MAX_FRAMES_IN_FLIGHT;
}
