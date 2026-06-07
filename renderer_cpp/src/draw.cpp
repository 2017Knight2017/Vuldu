#include <glm/gtc/matrix_transform.hpp>
#include <cstring>
#include "renderer.h" 
#include "doom/src/FFI.rs.h"

void VulkanRenderer::recordCommandBuffer(uint32_t imageIndex) {
	VkCommandBuffer currentCommandBuffer = this->commandBuffers[this->currentFrame];

	VkCommandBufferBeginInfo beginInfo{};
	beginInfo.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO;
	beginInfo.flags = 0; // Optional
	beginInfo.pInheritanceInfo = nullptr; // Optional

	if (vkBeginCommandBuffer(currentCommandBuffer, &beginInfo) != VK_SUCCESS) {
	    throw std::runtime_error("failed to begin recording command buffer!");
	}

	VkRenderPassBeginInfo renderPassInfo{};
	renderPassInfo.sType = VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO;
	renderPassInfo.renderPass = this->renderPass;
	renderPassInfo.framebuffer = this->swapChainFramebuffers[imageIndex];
	renderPassInfo.renderArea.offset = {0, 0};
	renderPassInfo.renderArea.extent = this->swapChainExtent;
	
	VkClearValue clearColor = {{{0.0f, 0.0f, 0.0f, 1.0f}}};
	renderPassInfo.clearValueCount = 1;
	renderPassInfo.pClearValues = &clearColor;

	vkCmdBeginRenderPass(currentCommandBuffer, &renderPassInfo, VK_SUBPASS_CONTENTS_INLINE);
	
	vkCmdBindPipeline(currentCommandBuffer, VK_PIPELINE_BIND_POINT_GRAPHICS, graphicsPipeline);

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

	if (this->vertexBuffer != VK_NULL_HANDLE && this->vertexCount > 0) {
	    VkBuffer vertexBuffers[] = {this->vertexBuffer};
	    VkDeviceSize offsets[] = {0};

	    vkCmdBindVertexBuffers(currentCommandBuffer, 0, 1, vertexBuffers, offsets);
		vkCmdBindIndexBuffer(currentCommandBuffer, this->indexBuffer, 0, VK_INDEX_TYPE_UINT16);

		vkCmdBindDescriptorSets(currentCommandBuffer, VK_PIPELINE_BIND_POINT_GRAPHICS, this->pipelineLayout, 0, 1, &this->descriptorSets[currentFrame], 0, nullptr);
	    vkCmdDrawIndexed(currentCommandBuffer, static_cast<uint32_t>(INDICES.size()), 1, 0, 0, 0);
	}

	vkCmdEndRenderPass(currentCommandBuffer);

	if (vkEndCommandBuffer(currentCommandBuffer) != VK_SUCCESS) {
	    throw std::runtime_error("failed to record command buffer!");
	}
}

void VulkanRenderer::createFramebuffers() {
	this->swapChainFramebuffers.resize(this->swapChainImageViews.size());
	for (size_t i = 0; i < this->swapChainImageViews.size(); i++) {
	    VkImageView attachments[] = {
	        this->swapChainImageViews[i]
	    };

	    VkFramebufferCreateInfo framebufferInfo{};
	    framebufferInfo.sType = VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO;
	    framebufferInfo.renderPass = this->renderPass;
	    framebufferInfo.attachmentCount = 1;
	    framebufferInfo.pAttachments = attachments;
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

void VulkanRenderer::drawFrame(const UniformBufferObject* ubo_ptr) {
	vkWaitForFences(this->device, 1, &this->inFlightFences[this->currentFrame], VK_TRUE, UINT64_MAX);

	uint32_t imageIndex;
	VkResult acquireNextImageResult = vkAcquireNextImageKHR(this->device, this->swapChain, UINT64_MAX, this->imageAvailableSemaphores[this->currentFrame], VK_NULL_HANDLE, &imageIndex);
	if (acquireNextImageResult == VK_ERROR_OUT_OF_DATE_KHR) {
        recreateSwapChain();
        return;
    } else if (acquireNextImageResult != VK_SUCCESS) {
        throw std::runtime_error("failed to acquire swap chain image!");
    }

	vkResetFences(this->device, 1, &this->inFlightFences[this->currentFrame]);

	vkResetCommandBuffer(this->commandBuffers[this->currentFrame], 0);
	recordCommandBuffer(imageIndex);

	updateUniformBuffer(ubo_ptr, this->currentFrame);

	VkSubmitInfo submitInfo{};
	submitInfo.sType = VK_STRUCTURE_TYPE_SUBMIT_INFO;

	VkSemaphore waitSemaphores[] = {this->imageAvailableSemaphores[this->currentFrame]};
	VkPipelineStageFlags waitStages[] = {VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT};
	submitInfo.waitSemaphoreCount = 1;
	submitInfo.pWaitSemaphores = waitSemaphores;
	submitInfo.pWaitDstStageMask = waitStages;
	submitInfo.commandBufferCount = 1;
	submitInfo.pCommandBuffers = &this->commandBuffers[this->currentFrame];

	VkSemaphore signalSemaphores[] = {this->renderFinishedSemaphores[imageIndex]};
	submitInfo.signalSemaphoreCount = 1;
	submitInfo.pSignalSemaphores = signalSemaphores;

	if (vkQueueSubmit(this->graphicsQueue, 1, &submitInfo, this->inFlightFences[this->currentFrame]) != VK_SUCCESS) {
	    throw std::runtime_error("failed to submit draw command buffer!");
	}

	VkPresentInfoKHR presentInfo{};
	presentInfo.sType = VK_STRUCTURE_TYPE_PRESENT_INFO_KHR;
	presentInfo.waitSemaphoreCount = 1;
	presentInfo.pWaitSemaphores = signalSemaphores;

	VkSwapchainKHR swapChains[] = {this->swapChain};
	presentInfo.swapchainCount = 1;
	presentInfo.pSwapchains = swapChains;
	presentInfo.pImageIndices = &imageIndex;
	presentInfo.pResults = nullptr;  // Optional

	VkResult presentResult = vkQueuePresentKHR(this->presentQueue, &presentInfo);

	if (presentResult == VK_ERROR_OUT_OF_DATE_KHR || presentResult == VK_SUBOPTIMAL_KHR) {
	    recreateSwapChain();
	} else if (presentResult != VK_SUCCESS) {
	    throw std::runtime_error("failed to present swap chain image!");
	}

	currentFrame = (currentFrame + 1) % MAX_FRAMES_IN_FLIGHT;
}