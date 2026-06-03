#include "renderer.h" 

void VulkanRenderer::recordCommandBuffer(uint32_t imageIndex) {
	VkCommandBufferBeginInfo beginInfo{};
	beginInfo.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO;
	beginInfo.flags = 0; // Optional
	beginInfo.pInheritanceInfo = nullptr; // Optional

	if (vkBeginCommandBuffer(this->commandBuffer, &beginInfo) != VK_SUCCESS) {
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

	vkCmdBeginRenderPass(this->commandBuffer, &renderPassInfo, VK_SUBPASS_CONTENTS_INLINE);
	
	vkCmdBindPipeline(this->commandBuffer, VK_PIPELINE_BIND_POINT_GRAPHICS, graphicsPipeline);

	VkViewport viewport{};
	viewport.x = 0.0f;
	viewport.y = 0.0f;
	viewport.width = (float) this->swapChainExtent.width;
	viewport.height = (float) this->swapChainExtent.height;
	viewport.minDepth = 0.0f;
	viewport.maxDepth = 1.0f;
	vkCmdSetViewport(this->commandBuffer, 0, 1, &viewport);

	VkRect2D scissor{};
	scissor.offset = {0, 0};
	scissor.extent = this->swapChainExtent;
	vkCmdSetScissor(this->commandBuffer, 0, 1, &scissor);

	vkCmdDraw(this->commandBuffer, 3, 1, 0, 0);

	vkCmdEndRenderPass(this->commandBuffer);

	if (vkEndCommandBuffer(this->commandBuffer) != VK_SUCCESS) {
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
	QueueFamilyIndices queueFamilyIndices = findQueueFamilies(this->physicalDevice, this->surface);

	VkCommandPoolCreateInfo poolInfo{};
	poolInfo.sType = VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO;
	poolInfo.flags = VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT;
	poolInfo.queueFamilyIndex = queueFamilyIndices.graphicsFamily.value();

	VkResult poolResult = vkCreateCommandPool(device, &poolInfo, nullptr, &commandPool);
	if (poolResult != VK_SUCCESS) {
	    throw std::runtime_error("failed to create command pool!");
	}
}

void VulkanRenderer::createCommandBuffer() {
	VkCommandBufferAllocateInfo allocInfo{};
	allocInfo.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO;
	allocInfo.commandPool = this->commandPool;
	allocInfo.level = VK_COMMAND_BUFFER_LEVEL_PRIMARY;
	allocInfo.commandBufferCount = 1;

	if (vkAllocateCommandBuffers(this->device, &allocInfo, &this->commandBuffer) != VK_SUCCESS) {
	    throw std::runtime_error("failed to allocate command buffers!");
	}
}

void VulkanRenderer::createSyncObjects() {
	VkSemaphoreCreateInfo semaphoreInfo{};
    semaphoreInfo.sType = VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO;

	VkFenceCreateInfo fenceInfo{};
	fenceInfo.sType = VK_STRUCTURE_TYPE_FENCE_CREATE_INFO;
	fenceInfo.flags = VK_FENCE_CREATE_SIGNALED_BIT;

	VkResult imageAvailableSemaphoreResult = vkCreateSemaphore(this->device, &semaphoreInfo, nullptr, &this->imageAvailableSemaphore);
	VkResult renderFinishedSemaphoreResult = vkCreateSemaphore(this->device, &semaphoreInfo, nullptr, &this->renderFinishedSemaphore);
	VkResult inFlightFenceResult = vkCreateFence(this->device, &fenceInfo, nullptr, &this->inFlightFence);
	if (imageAvailableSemaphoreResult != VK_SUCCESS 
    	|| renderFinishedSemaphoreResult != VK_SUCCESS
    	|| inFlightFenceResult != VK_SUCCESS) 
	{
    	throw std::runtime_error("failed to create semaphores!");
	}
}

void VulkanRenderer::drawFrame() {
	vkWaitForFences(this->device, 1, &this->inFlightFence, VK_TRUE, UINT64_MAX);
	vkResetFences(this->device, 1, &this->inFlightFence);

	uint32_t imageIndex;
    vkAcquireNextImageKHR(this->device, this->swapChain, UINT64_MAX, this->imageAvailableSemaphore, VK_NULL_HANDLE, &imageIndex);

	vkResetCommandBuffer(this->commandBuffer, 0);
	recordCommandBuffer(imageIndex);

	VkSubmitInfo submitInfo{};
	submitInfo.sType = VK_STRUCTURE_TYPE_SUBMIT_INFO;

	VkSemaphore waitSemaphores[] = {this->imageAvailableSemaphore};
	VkPipelineStageFlags waitStages[] = {VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT};
	submitInfo.waitSemaphoreCount = 1;
	submitInfo.pWaitSemaphores = waitSemaphores;
	submitInfo.pWaitDstStageMask = waitStages;
	submitInfo.commandBufferCount = 1;
	submitInfo.pCommandBuffers = &commandBuffer;

	VkSemaphore signalSemaphores[] = {this->renderFinishedSemaphore};
	submitInfo.signalSemaphoreCount = 1;
	submitInfo.pSignalSemaphores = signalSemaphores;

	if (vkQueueSubmit(this->graphicsQueue, 1, &submitInfo, this->inFlightFence) != VK_SUCCESS) {
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

	vkQueuePresentKHR(this->presentQueue, &presentInfo);
}