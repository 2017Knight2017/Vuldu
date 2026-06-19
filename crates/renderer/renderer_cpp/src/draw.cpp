#include <cstring>
#include "renderer.h" 
#include "renderer/src/bridge.rs.h"

void VulkanRenderer::createFramebuffers() {
	this->swapChainFramebuffers.resize(this->swapChainImageViews.size());
	for (size_t i = 0; i < this->swapChainImageViews.size(); i++) {
	    std::array<VkImageView, 2> attachments = {
	        this->swapChainImageViews[i],
			this->depthImageView
	    };

	    VkFramebufferCreateInfo framebufferInfo{};
	    framebufferInfo.sType = VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO;
	    framebufferInfo.renderPass = this->renderPass;
	    framebufferInfo.attachmentCount = static_cast<uint32_t>(attachments.size());
		framebufferInfo.pAttachments = attachments.data();
	    framebufferInfo.width = this->swapChainExtent.width;
	    framebufferInfo.height = this->swapChainExtent.height;
	    framebufferInfo.layers = 1;

		VkResult framebufferResult = vkCreateFramebuffer(this->device, &framebufferInfo, nullptr, &this->swapChainFramebuffers[i]);
	    if (framebufferResult != VK_SUCCESS) {
	        throw std::runtime_error("failed to create framebuffer!");
	    }
	}
}

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

void VulkanRenderer::updateUniformBuffer(const UniformBufferObject* ubo_ptr, uint32_t currentImage) {
	if (ubo_ptr == nullptr) return;
	memcpy(this->uniformBuffersMapped[currentImage], ubo_ptr, sizeof(UniformBufferObject));
	
	auto* ubo = reinterpret_cast<UniformBufferObject*>(this->uniformBuffersMapped[currentImage]);
    ubo->proj[5] *= -1.0f;
}

void VulkanRenderer::setPaletteIndex(uint32_t idx) {
	this->currentPaletteIndex = idx % MAX_PAL;
};

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

	updateUniformBuffer(ubo_ptr, this->currentFrame);

	vkResetFences(this->device, 1, &this->inFlightFences[this->currentFrame]);

	vkResetCommandBuffer(currentCommandBuffer, 0);

    VkCommandBufferBeginInfo beginInfo{};
    beginInfo.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO;
    beginInfo.flags = VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT;

    if (vkBeginCommandBuffer(currentCommandBuffer, &beginInfo) != VK_SUCCESS) {
        throw std::runtime_error("Failed to begin recording command buffer!");
    }

    VkRenderPassBeginInfo renderPassInfo{};
    renderPassInfo.sType = VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO;
    renderPassInfo.renderPass = this->renderPass;
    renderPassInfo.framebuffer = this->swapChainFramebuffers[this->currentImageIndex];
    renderPassInfo.renderArea.offset = {0, 0};
    renderPassInfo.renderArea.extent = this->swapChainExtent;

    std::array<VkClearValue, 2> clearValues{};
    clearValues[0].color = {{0.1f, 0.1f, 0.1f, 1.0f}};
    clearValues[1].depthStencil = {1.0f, 0};

    renderPassInfo.clearValueCount = static_cast<uint32_t>(clearValues.size());
    renderPassInfo.pClearValues = clearValues.data();

    vkCmdBeginRenderPass(currentCommandBuffer, &renderPassInfo, VK_SUBPASS_CONTENTS_INLINE);

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

void VulkanRenderer::drawSprite(uint32_t textureId, uint32_t width, uint32_t height, 
                                uint32_t lightLevel, int16_t leftOffset, int16_t topOffset, 
                                float x, float y, float z) {
    VkCommandBuffer currentCommandBuffer = this->commandBuffers[this->currentFrame];

    vkCmdBindPipeline(currentCommandBuffer, VK_PIPELINE_BIND_POINT_GRAPHICS, this->spritePipeline);

    vkCmdBindDescriptorSets(
        currentCommandBuffer, 
        VK_PIPELINE_BIND_POINT_GRAPHICS, 
        this->spritePipelineLayout, 
        0, 1, 
        &this->descriptorSets[this->currentFrame], 
        0, nullptr
    );
    
    SpritePushConstants constants{};
    constants.paletteIndex = this->currentPaletteIndex;
    constants.lightLevel = lightLevel;
    constants.textureId = textureId;
    constants.spriteWidth = static_cast<float>(width);
    constants.spriteHeight = static_cast<float>(height);
    constants.leftOffset = static_cast<float>(leftOffset);
    constants.topOffset = static_cast<float>(topOffset);
    constants.worldPos[0] = x;
    constants.worldPos[1] = y;
    constants.worldPos[2] = z;
    constants.worldPos[3] = 1.0f;

    vkCmdPushConstants(
        currentCommandBuffer,
        this->spritePipelineLayout,
        VK_SHADER_STAGE_VERTEX_BIT | VK_SHADER_STAGE_FRAGMENT_BIT,
        0,
        sizeof(SpritePushConstants),
        &constants
    );

    vkCmdDraw(this->commandBuffers[this->currentFrame], 6, 1, 0, 0);
}

void VulkanRenderer::endFrame() {
	VkCommandBuffer currentCommandBuffer = this->commandBuffers[this->currentFrame];

    vkCmdEndRenderPass(currentCommandBuffer);

    if (vkEndCommandBuffer(currentCommandBuffer) != VK_SUCCESS) {
        throw std::runtime_error("Failed to record command buffer!");
    }

    VkSubmitInfo submitInfo{};
    submitInfo.sType = VK_STRUCTURE_TYPE_SUBMIT_INFO;

    VkSemaphore waitSemaphores[] = { this->imageAvailableSemaphores[this->currentFrame] };
    VkPipelineStageFlags waitStages[] = { VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT };
    submitInfo.waitSemaphoreCount = 1;
    submitInfo.pWaitSemaphores = waitSemaphores;
    submitInfo.pWaitDstStageMask = waitStages;

    submitInfo.commandBufferCount = 1;
    submitInfo.pCommandBuffers = &currentCommandBuffer;

    VkSemaphore signalSemaphores[] = { this->renderFinishedSemaphores[this->currentImageIndex] };
    submitInfo.signalSemaphoreCount = 1;
    submitInfo.pSignalSemaphores = signalSemaphores;

    vkResetFences(this->device, 1, &this->inFlightFences[currentFrame]);
    if (vkQueueSubmit(this->graphicsQueue, 1, &submitInfo, this->inFlightFences[currentFrame]) != VK_SUCCESS) {
        throw std::runtime_error("Failed to submit draw command buffer!");
    }

	VkPresentInfoKHR presentInfo{};
	presentInfo.sType = VK_STRUCTURE_TYPE_PRESENT_INFO_KHR;
	presentInfo.waitSemaphoreCount = 1;
	presentInfo.pWaitSemaphores = signalSemaphores;

	VkSwapchainKHR swapChains[] = {this->swapChain};
	presentInfo.swapchainCount = 1;
	presentInfo.pSwapchains = swapChains;
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

void VulkanRenderer::drawLevel() {
    VkCommandBuffer currentCommandBuffer = this->commandBuffers[this->currentFrame];

    if (this->vertexBuffer != VK_NULL_HANDLE && this->vertexCount > 0) {
        vkCmdBindPipeline(currentCommandBuffer, VK_PIPELINE_BIND_POINT_GRAPHICS, this->levelPipeline);
        
        vkCmdBindDescriptorSets(
            currentCommandBuffer, 
            VK_PIPELINE_BIND_POINT_GRAPHICS, 
            this->levelPipelineLayout, 
            0, 1, 
            &this->descriptorSets[this->currentFrame], 
            0, nullptr
        );
        
        VkBuffer vertexBuffers[] = {this->vertexBuffer};
        VkDeviceSize offsets[] = {0};

        vkCmdBindVertexBuffers(currentCommandBuffer, 0, 1, vertexBuffers, offsets);
        vkCmdBindIndexBuffer(currentCommandBuffer, this->indexBuffer, 0, VK_INDEX_TYPE_UINT16);

        LevelPushConstants constants{};
        constants.paletteIndex = this->currentPaletteIndex;

		vkCmdPushConstants(
            currentCommandBuffer,
            this->levelPipelineLayout,
            VK_SHADER_STAGE_FRAGMENT_BIT,
            0,                    
            sizeof(int),
            &constants 
        );

        vkCmdDrawIndexed(currentCommandBuffer, this->indexCount, 1, 0, 0, 0);
    }
}
