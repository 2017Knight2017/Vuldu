#include <cstring>
#include <stdexcept>
#include "renderer.h" 
#include "renderer/src/bridge.rs.h"

void VulkanRenderer::createCommandPool() {
	QueueFamilyIndices queueFamilyIndices = findQueueFamilies(this->physicalDevice);

	VkCommandPoolCreateInfo poolInfo{};
	poolInfo.sType = VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO;
	poolInfo.flags = VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT;
	poolInfo.queueFamilyIndex = queueFamilyIndices.graphicsFamily.value();

	VkResult poolResult = vkCreateCommandPool(device, &poolInfo, nullptr, &commandPool);
	if (poolResult != VK_SUCCESS) {
	    throw std::runtime_error("failed to create command pool!");
	}
}

void VulkanRenderer::createCommandBuffers() {
	this->commandBuffers.resize(MAX_FRAMES_IN_FLIGHT);

	VkCommandBufferAllocateInfo allocInfo{};
	allocInfo.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO;
	allocInfo.commandPool = this->commandPool;
	allocInfo.level = VK_COMMAND_BUFFER_LEVEL_PRIMARY;
	allocInfo.commandBufferCount = (uint32_t) this->commandBuffers.size();

    if (vkAllocateCommandBuffers(this->device, &allocInfo, this->commandBuffers.data()) != VK_SUCCESS) {
        throw std::runtime_error("failed to allocate command buffers!");
    }
}

void VulkanRenderer::createSyncObjects() {
	imageAvailableSemaphores.clear();
    renderFinishedSemaphores.clear();
    inFlightFences.clear();

	size_t imageCount = this->swapChainImages.size();

	imageAvailableSemaphores.resize(imageCount);
    renderFinishedSemaphores.resize(imageCount);
    inFlightFences.resize(MAX_FRAMES_IN_FLIGHT);

	VkSemaphoreCreateInfo semaphoreInfo{};
    semaphoreInfo.sType = VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO;

	VkFenceCreateInfo fenceInfo{};
	fenceInfo.sType = VK_STRUCTURE_TYPE_FENCE_CREATE_INFO;
	fenceInfo.flags = VK_FENCE_CREATE_SIGNALED_BIT;

	for (size_t i = 0; i < imageCount; i++) {
		VkResult imageAvailableSemaphoreResult = vkCreateSemaphore(this->device, &semaphoreInfo, nullptr, &this->imageAvailableSemaphores[i]);
		VkResult renderFinishedSemaphoreResult = vkCreateSemaphore(this->device, &semaphoreInfo, nullptr, &this->renderFinishedSemaphores[i]);
		if (imageAvailableSemaphoreResult != VK_SUCCESS || renderFinishedSemaphoreResult != VK_SUCCESS) {
			throw std::runtime_error("failed to create semaphores!");
		}
	}

	for (size_t i = 0; i < MAX_FRAMES_IN_FLIGHT; i++) {
		VkResult inFlightFenceResult = vkCreateFence(this->device, &fenceInfo, nullptr, &this->inFlightFences[i]);
		if (inFlightFenceResult != VK_SUCCESS) {
    		throw std::runtime_error("failed to create fences!");
		}
	}
}

void VulkanRenderer::updateUniformBuffer(const UniformBufferObject* ubo_ptr) {
	if (ubo_ptr == nullptr) return;
	memcpy(this->uniformBuffersMapped[this->currentFrame], ubo_ptr, sizeof(UniformBufferObject));
	
	auto* ubo = reinterpret_cast<UniformBufferObject*>(this->uniformBuffersMapped[this->currentFrame]);
    ubo->proj[5] *= -1.0f;
    ubo->proj[0] *= -1.0f;
}

void VulkanRenderer::updateObjectInstances(const ObjectInstance* instances_ptr, size_t instances_count) {
    if (instances_count == 0 || instances_ptr == nullptr) return;
    if (instances_count > MAX_OBJECTS) return;

    this->activeObjectsCount = static_cast<uint32_t>(instances_count);

	VkDeviceSize instanceBufferSize = sizeof(ObjectInstance) * instances_count;
    memcpy(this->objectInstanceBuffersMapped[this->currentFrame], instances_ptr, instanceBufferSize);   
}

void VulkanRenderer::updateUiInstances(const UiInstance* instances_ptr, size_t instances_count) {
    if (instances_count == 0 || instances_ptr == nullptr) return;
    if (instances_count > MAX_UI) return;

    this->activeUiCount = static_cast<uint32_t>(instances_count);

	VkDeviceSize instanceBufferSize = sizeof(UiInstance) * instances_count;
    memcpy(this->uiInstanceBuffersMapped[this->currentFrame], instances_ptr, instanceBufferSize);   
}

void VulkanRenderer::setPaletteIndex(uint32_t idx) {
	this->currentPaletteIndex = idx % MAX_PAL;
}

void VulkanRenderer::setSkyIndex(uint32_t idx) {
	this->currentSkyIndex = idx % MAX_SKY;
}

void VulkanRenderer::setGlobalTimer(uint32_t global_timer) {
    this->globalTimer = static_cast<float>(global_timer);
}

size_t getMaxSky() {
    return MAX_SKY;
}

void VulkanRenderer::startFrame(const UniformBufferObject* ubo_ptr) {
	VkCommandBuffer currentCommandBuffer = this->commandBuffers[this->currentFrame];
	vkWaitForFences(this->device, 1, &this->inFlightFences[this->currentFrame], VK_TRUE, UINT64_MAX);

	VkResult acquireNextImageResult = vkAcquireNextImageKHR(this->device, this->swapChain, UINT64_MAX, this->imageAvailableSemaphores[this->currentFrame], VK_NULL_HANDLE, &this->currentImageIndex);
	if (acquireNextImageResult == VK_ERROR_OUT_OF_DATE_KHR) {
        recreateSwapChain();
        return;
    } else if (acquireNextImageResult != VK_SUCCESS) {
        throw std::runtime_error("failed to acquire swap chain image!");
    }

	updateUniformBuffer(ubo_ptr);

	vkResetFences(this->device, 1, &this->inFlightFences[this->currentFrame]);
	vkResetCommandBuffer(currentCommandBuffer, 0);

    VkCommandBufferBeginInfo beginInfo{};
    beginInfo.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO;
    beginInfo.flags = VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT;

    if (vkBeginCommandBuffer(currentCommandBuffer, &beginInfo) != VK_SUCCESS) {
        throw std::runtime_error("Failed to begin recording command buffer!");
    }

    changeImageLayout(
        currentCommandBuffer, 
        VK_IMAGE_LAYOUT_UNDEFINED, 
        VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        {&this->depthImage, 1} 
    );
    changeImageLayout(
        currentCommandBuffer, 
        VK_IMAGE_LAYOUT_UNDEFINED, 
        VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL,
        {&this->swapChainImages[this->currentImageIndex], 1} 
    );
    beginRendering(currentCommandBuffer);

	VkViewport viewport{};
    viewport.x = 0.0f;
    viewport.y = 0.0f;
    viewport.width = (float) this->swapChainExtent.width;
    viewport.height = (float) this->swapChainExtent.height;
    viewport.minDepth = 0.0f;
    viewport.maxDepth = 1.0f;
    vkCmdSetViewport(currentCommandBuffer, 0, 1, &viewport);

    VkRect2D scissor{};
    scissor.offset = {0, 0};
    scissor.extent = this->swapChainExtent;
    vkCmdSetScissor(currentCommandBuffer, 0, 1, &scissor);
}

void VulkanRenderer::drawLevel() {
    VkCommandBuffer currentCommandBuffer = this->commandBuffers[this->currentFrame];

    if (this->levelVertexBuffer != VK_NULL_HANDLE && this->levelVertexCount > 0) {
        vkCmdBindPipeline(currentCommandBuffer, VK_PIPELINE_BIND_POINT_GRAPHICS, this->levelPipeline);
        
        vkCmdBindDescriptorSets(
            currentCommandBuffer, 
            VK_PIPELINE_BIND_POINT_GRAPHICS, 
            this->levelPipelineLayout, 
            0, 1, 
            &this->descriptorSets[this->currentFrame], 
            0, nullptr
        );
        
        VkBuffer vertexBuffers[] = {this->levelVertexBuffer};
        VkDeviceSize offsets[] = {0};

        vkCmdBindVertexBuffers(currentCommandBuffer, 0, 1, vertexBuffers, offsets);
        vkCmdBindIndexBuffer(currentCommandBuffer, this->levelIndexBuffer, 0, VK_INDEX_TYPE_UINT32);

        WindowSize currentResolution = get_winit_window_size(this->window_raw_ptr);

        PushConstants constants{};
        constants.paletteIndex = this->currentPaletteIndex;
        constants.resolution[0] = static_cast<float>(currentResolution.width);
        constants.resolution[1] = static_cast<float>(currentResolution.height);
        constants.skyIndex = this->currentSkyIndex;
        constants.skyWidth = this->skyWidths[this->currentSkyIndex];
        constants.globalTimer = this->globalTimer;

		vkCmdPushConstants(
            currentCommandBuffer,
            this->levelPipelineLayout,
            VK_SHADER_STAGE_FRAGMENT_BIT,
            0,
            sizeof(PushConstants),
            &constants 
        );

        vkCmdDrawIndexed(currentCommandBuffer, this->levelIndexCount, 1, 0, 0, 0);
    }
}

void VulkanRenderer::drawObjects() {
    VkCommandBuffer currentCommandBuffer = this->commandBuffers[this->currentFrame];

    if (this->objectVertexBuffer != VK_NULL_HANDLE && this->objectIndexCount > 0) {
        
        vkCmdBindPipeline(currentCommandBuffer, VK_PIPELINE_BIND_POINT_GRAPHICS, this->spritePipeline);
        
        vkCmdBindDescriptorSets(
            currentCommandBuffer, 
            VK_PIPELINE_BIND_POINT_GRAPHICS, 
            this->spritePipelineLayout, 
            0, 1, 
            &this->descriptorSets[this->currentFrame], 
            0, nullptr
        );
        
        VkBuffer vertexBuffers[] = {this->objectVertexBuffer, this->objectInstanceBuffers[this->currentFrame]};
        VkDeviceSize offsets[] = {0, 0};
        vkCmdBindVertexBuffers(currentCommandBuffer, 0, 2, vertexBuffers, offsets);
        
        vkCmdBindIndexBuffer(currentCommandBuffer, this->objectIndexBuffer, 0, VK_INDEX_TYPE_UINT32);

        PushConstants constants{};
        constants.paletteIndex = this->currentPaletteIndex;

        vkCmdPushConstants(
            currentCommandBuffer,
            this->spritePipelineLayout,
            VK_SHADER_STAGE_FRAGMENT_BIT,
            0,                    
            sizeof(uint32_t),
            &constants 
        );

        vkCmdDrawIndexed(currentCommandBuffer, this->objectIndexCount, this->activeObjectsCount, 0, 0, 0);
    }
}

void VulkanRenderer::drawUi() {
    VkCommandBuffer currentCommandBuffer = this->commandBuffers[this->currentFrame];

    if (this->uiVertexBuffer != VK_NULL_HANDLE && this->uiIndexCount > 0) {
        
        vkCmdBindPipeline(currentCommandBuffer, VK_PIPELINE_BIND_POINT_GRAPHICS, this->uiPipeline);
        
        vkCmdBindDescriptorSets(
            currentCommandBuffer, 
            VK_PIPELINE_BIND_POINT_GRAPHICS, 
            this->uiPipelineLayout, 
            0, 1, 
            &this->descriptorSets[this->currentFrame], 
            0, nullptr
        );
        
        VkBuffer vertexBuffers[] = {this->uiVertexBuffer, this->uiInstanceBuffers[this->currentFrame]};
        VkDeviceSize offsets[] = {0, 0};
        vkCmdBindVertexBuffers(currentCommandBuffer, 0, 2, vertexBuffers, offsets);
        
        vkCmdBindIndexBuffer(currentCommandBuffer, this->uiIndexBuffer, 0, VK_INDEX_TYPE_UINT32);

        PushConstants constants{};
        constants.paletteIndex = this->currentPaletteIndex;

        vkCmdPushConstants(
            currentCommandBuffer,
            this->uiPipelineLayout,
            VK_SHADER_STAGE_FRAGMENT_BIT,
            0,                    
            sizeof(uint32_t),
            &constants 
        );

        vkCmdDrawIndexed(currentCommandBuffer, this->uiIndexCount, this->activeUiCount, 0, 0, 0);
    }
}

void VulkanRenderer::endFrame() {
	VkCommandBuffer currentCommandBuffer = this->commandBuffers[this->currentFrame];

    vkCmdEndRendering(currentCommandBuffer);
    changeImageLayout(
        currentCommandBuffer, 
        VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL, 
        VK_IMAGE_LAYOUT_PRESENT_SRC_KHR,
        {&this->swapChainImages[this->currentImageIndex], 1} 
    );

    if (vkEndCommandBuffer(currentCommandBuffer) != VK_SUCCESS) {
        throw std::runtime_error("Failed to record command buffer!");
    }

    VkPipelineStageFlags waitStages[] = { VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT };
    VkSubmitInfo submitInfo{};
    submitInfo.sType = VK_STRUCTURE_TYPE_SUBMIT_INFO;
    submitInfo.waitSemaphoreCount = 1;
    submitInfo.pWaitSemaphores = &this->imageAvailableSemaphores[this->currentFrame];
    submitInfo.pWaitDstStageMask = waitStages;
    submitInfo.commandBufferCount = 1;
    submitInfo.pCommandBuffers = &currentCommandBuffer;
    submitInfo.signalSemaphoreCount = 1;
    submitInfo.pSignalSemaphores = &this->renderFinishedSemaphores[this->currentImageIndex];

    vkResetFences(this->device, 1, &this->inFlightFences[this->currentFrame]);
    if (vkQueueSubmit(this->graphicsQueue, 1, &submitInfo, this->inFlightFences[this->currentFrame]) != VK_SUCCESS) {
        throw std::runtime_error("Failed to submit draw command buffer!");
    }

	VkPresentInfoKHR presentInfo{};
	presentInfo.sType = VK_STRUCTURE_TYPE_PRESENT_INFO_KHR;
	presentInfo.waitSemaphoreCount = 1;
	presentInfo.pWaitSemaphores = &this->renderFinishedSemaphores[this->currentImageIndex];
	presentInfo.swapchainCount = 1;
	presentInfo.pSwapchains = &this->swapChain;
	presentInfo.pImageIndices = &this->currentImageIndex;
	presentInfo.pResults = nullptr;  // Optional

	VkResult presentResult = vkQueuePresentKHR(this->presentQueue, &presentInfo);

	if (presentResult == VK_ERROR_OUT_OF_DATE_KHR || presentResult == VK_SUBOPTIMAL_KHR) {
	    recreateSwapChain();
	} else if (presentResult != VK_SUCCESS) {
	    throw std::runtime_error("failed to present swap chain image!");
	}
    
    currentFrame = (currentFrame + 1) % MAX_FRAMES_IN_FLIGHT;
}


void VulkanRenderer::beginRendering(VkCommandBuffer currentCommandBuffer) {
    VkClearValue clearColor{{{0.1f, 0.1f, 0.1f, 1.0f}}};
    VkClearValue clearDepth{{{1.0f, 0}}};

    VkRenderingAttachmentInfo depthAttachment{};
    depthAttachment.sType = VK_STRUCTURE_TYPE_RENDERING_ATTACHMENT_INFO;
    depthAttachment.imageView = this->depthImageView;
    depthAttachment.imageLayout = VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL;
	depthAttachment.resolveMode = VK_RESOLVE_MODE_NONE;
	depthAttachment.resolveImageView = VK_NULL_HANDLE;
	depthAttachment.resolveImageLayout = VK_IMAGE_LAYOUT_UNDEFINED;
	depthAttachment.loadOp = VK_ATTACHMENT_LOAD_OP_CLEAR;
	depthAttachment.storeOp = VK_ATTACHMENT_STORE_OP_STORE;
	depthAttachment.clearValue = clearDepth;

	VkRenderingAttachmentInfo colorAttachment{};
    colorAttachment.sType = VK_STRUCTURE_TYPE_RENDERING_ATTACHMENT_INFO;
    colorAttachment.imageView = this->swapChainImageViews[this->currentImageIndex];
	colorAttachment.imageLayout = VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL;
	colorAttachment.resolveMode = VK_RESOLVE_MODE_NONE;
	colorAttachment.resolveImageView = VK_NULL_HANDLE;
	colorAttachment.resolveImageLayout = VK_IMAGE_LAYOUT_UNDEFINED;
	colorAttachment.loadOp = VK_ATTACHMENT_LOAD_OP_CLEAR;
	colorAttachment.storeOp = VK_ATTACHMENT_STORE_OP_STORE;
	colorAttachment.clearValue = clearColor;

    VkRenderingInfo renderingInfo{};
    renderingInfo.sType = VK_STRUCTURE_TYPE_RENDERING_INFO_KHR;
    renderingInfo.renderArea = {
        {0, 0}, 
        this->swapChainExtent
    };
    renderingInfo.layerCount = 1;
    renderingInfo.viewMask = 0;
    renderingInfo.colorAttachmentCount = 1;
    renderingInfo.pColorAttachments = &colorAttachment;
    renderingInfo.pDepthAttachment = &depthAttachment;

    vkCmdBeginRendering(currentCommandBuffer, &renderingInfo);
}
