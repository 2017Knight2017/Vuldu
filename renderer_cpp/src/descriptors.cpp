#include "renderer.h"
#include "doom/src/FFI.rs.h"

void VulkanRenderer::createDescriptorSetLayout() {
	VkDescriptorSetLayoutBinding uboLayoutBinding{};
    uboLayoutBinding.binding = 0;
    uboLayoutBinding.descriptorType = VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER;
    uboLayoutBinding.descriptorCount = 1;
	uboLayoutBinding.stageFlags = VK_SHADER_STAGE_VERTEX_BIT;
	uboLayoutBinding.pImmutableSamplers = nullptr;

	VkDescriptorSetLayoutBinding samplerLayoutBinding{};
	samplerLayoutBinding.binding = 1;
	samplerLayoutBinding.descriptorCount = 1;
	samplerLayoutBinding.descriptorType = VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER;
	samplerLayoutBinding.pImmutableSamplers = nullptr;
	samplerLayoutBinding.stageFlags = VK_SHADER_STAGE_FRAGMENT_BIT;

	std::array<VkDescriptorSetLayoutBinding, 2> bindings = {uboLayoutBinding, samplerLayoutBinding};

	VkDescriptorSetLayoutCreateInfo layoutInfo{};
	layoutInfo.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO;
	layoutInfo.bindingCount = bindings.size();
	layoutInfo.pBindings = bindings.data();

	VkResult descriptorSetResult = vkCreateDescriptorSetLayout(this->device, &layoutInfo, nullptr, &this->descriptorSetLayout);
	if (descriptorSetResult != VK_SUCCESS) {
	    throw std::runtime_error("failed to create descriptor set layout!");
	}
}

void VulkanRenderer::createUniformBuffers() {
    VkDeviceSize bufferSize = sizeof(UniformBufferObject);

    uniformBuffers.resize(MAX_FRAMES_IN_FLIGHT);
    uniformBuffersMemory.resize(MAX_FRAMES_IN_FLIGHT);
    uniformBuffersMapped.resize(MAX_FRAMES_IN_FLIGHT);

    for (size_t i = 0; i < MAX_FRAMES_IN_FLIGHT; i++) {
        createBuffer(bufferSize, VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT, VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT, this->uniformBuffers[i], this->uniformBuffersMemory[i]);

        vkMapMemory(this->device, this->uniformBuffersMemory[i], 0, bufferSize, 0, &uniformBuffersMapped[i]);
    }
}

void VulkanRenderer::createDescriptorPool() {
	std::array<VkDescriptorPoolSize, 2> poolSizes{};
	poolSizes[0].type = VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER;
	poolSizes[0].descriptorCount = static_cast<uint32_t>(MAX_FRAMES_IN_FLIGHT);
	poolSizes[1].type = VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER;
	poolSizes[1].descriptorCount = static_cast<uint32_t>(MAX_FRAMES_IN_FLIGHT);
	
	VkDescriptorPoolCreateInfo poolInfo{};
	poolInfo.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO;
	poolInfo.poolSizeCount = static_cast<uint32_t>(poolSizes.size());
	poolInfo.pPoolSizes = poolSizes.data();
	poolInfo.maxSets = static_cast<uint32_t>(MAX_FRAMES_IN_FLIGHT);
	
	VkResult descriptorPoolResult = vkCreateDescriptorPool(this->device, &poolInfo, nullptr, &this->descriptorPool);
	if (descriptorPoolResult != VK_SUCCESS) {
	    throw std::runtime_error("failed to create descriptor pool!");
	}
}

void VulkanRenderer::allocateDescriptorSets() {
	std::vector<VkDescriptorSetLayout> layouts(MAX_FRAMES_IN_FLIGHT, descriptorSetLayout);
	VkDescriptorSetAllocateInfo allocInfo{};
	allocInfo.sType = VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO;
	allocInfo.descriptorPool = this->descriptorPool;
	allocInfo.descriptorSetCount = static_cast<uint32_t>(MAX_FRAMES_IN_FLIGHT);
	allocInfo.pSetLayouts = layouts.data();

	descriptorSets.resize(MAX_FRAMES_IN_FLIGHT);
	
	VkResult descriptorSetsResult = vkAllocateDescriptorSets(this->device, &allocInfo, this->descriptorSets.data());
	if (descriptorSetsResult != VK_SUCCESS) {
	    throw std::runtime_error("failed to allocate descriptor sets!");
	}
}

void VulkanRenderer::updateDescriptorSets() {
	for (size_t i = 0; i < MAX_FRAMES_IN_FLIGHT; i++) {
	    VkDescriptorBufferInfo bufferInfo{};
	    bufferInfo.buffer = this->uniformBuffers[i];
	    bufferInfo.offset = 0;
	    bufferInfo.range = sizeof(UniformBufferObject);

		VkDescriptorImageInfo imageInfo{};
    	imageInfo.imageLayout = VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL;
    	imageInfo.imageView = this->textureImageView;
    	imageInfo.sampler = this->textureSampler;

		std::array<VkWriteDescriptorSet, 2> descriptorWrites{};

		descriptorWrites[0].sType = VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET;
		descriptorWrites[0].dstSet = this->descriptorSets[i];
		descriptorWrites[0].dstBinding = 0;
		descriptorWrites[0].dstArrayElement = 0;
		descriptorWrites[0].descriptorType = VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER;
		descriptorWrites[0].descriptorCount = 1;
		descriptorWrites[0].pBufferInfo = &bufferInfo;

		descriptorWrites[1].sType = VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET;
		descriptorWrites[1].dstSet = this->descriptorSets[i];
		descriptorWrites[1].dstBinding = 1;
		descriptorWrites[1].dstArrayElement = 0;
		descriptorWrites[1].descriptorType = VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER;
		descriptorWrites[1].descriptorCount = 1;
		descriptorWrites[1].pImageInfo = &imageInfo;

		vkUpdateDescriptorSets(this->device, static_cast<uint32_t>(descriptorWrites.size()), descriptorWrites.data(), 0, nullptr);
	}

	
}