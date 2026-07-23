#include <cstring>
#include <iostream>
#include "renderer.h"
#include "renderer/src/bridge.rs.h"
#include "utils.h"

void VulkanRenderer::createBuffer(VkDeviceSize bufferSize, VkBufferUsageFlags usage, VkMemoryPropertyFlags properties, VkBuffer& buffer, VkDeviceMemory& bufferMemory) {
	VkBufferCreateInfo bufferInfo{};
	bufferInfo.sType = VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO;
	bufferInfo.size = bufferSize;
	bufferInfo.usage = usage;
	bufferInfo.sharingMode = VK_SHARING_MODE_EXCLUSIVE;

	VkResult bufferResult = vkCreateBuffer(this->device, &bufferInfo, nullptr, &buffer);
	if (bufferResult != VK_SUCCESS) {
        throw std::runtime_error("failed to create vertex buffer!");
    }

	VkMemoryRequirements memRequirements;
	vkGetBufferMemoryRequirements(this->device, buffer, &memRequirements);

	VkMemoryAllocateInfo allocInfo{};
	allocInfo.sType = VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO;
	allocInfo.allocationSize = memRequirements.size;
	allocInfo.memoryTypeIndex = findMemoryType(this->physicalDevice, memRequirements.memoryTypeBits, properties);
	
	VkResult memoryAllocationResult = vkAllocateMemory(this->device, &allocInfo, nullptr, &bufferMemory);
	if (memoryAllocationResult != VK_SUCCESS) {
	    throw std::runtime_error("failed to allocate vertex buffer memory!");
	}

	vkBindBufferMemory(this->device, buffer, bufferMemory, 0);
}

void VulkanRenderer::copyBuffer(VkBuffer srcBuffer, VkBuffer dstBuffer, VkDeviceSize size) {
	VkCommandBuffer commandBuffer = beginSingleTimeCommands();

	VkBufferCopy copyRegion{};
	copyRegion.srcOffset = 0; // Optional
	copyRegion.dstOffset = 0; // Optional
	copyRegion.size = size;
	vkCmdCopyBuffer(commandBuffer, srcBuffer, dstBuffer, 1, &copyRegion);

	endSingleTimeCommands(commandBuffer);
}

VkCommandBuffer VulkanRenderer::beginSingleTimeCommands() {
    VkCommandBufferAllocateInfo allocInfo{};
    allocInfo.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO;
    allocInfo.level = VK_COMMAND_BUFFER_LEVEL_PRIMARY;
    allocInfo.commandPool = this->commandPool;
    allocInfo.commandBufferCount = 1;

    VkCommandBuffer commandBuffer;
    vkAllocateCommandBuffers(this->device, &allocInfo, &commandBuffer);

    VkCommandBufferBeginInfo beginInfo{};
    beginInfo.sType = VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO;
    beginInfo.flags = VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT;

    vkBeginCommandBuffer(commandBuffer, &beginInfo);

    return commandBuffer;
}

void VulkanRenderer::endSingleTimeCommands(VkCommandBuffer commandBuffer) {
    vkEndCommandBuffer(commandBuffer);

    VkSubmitInfo submitInfo{};
    submitInfo.sType = VK_STRUCTURE_TYPE_SUBMIT_INFO;
    submitInfo.commandBufferCount = 1;
    submitInfo.pCommandBuffers = &commandBuffer;

    vkQueueSubmit(this->graphicsQueue, 1, &submitInfo, VK_NULL_HANDLE);
    vkQueueWaitIdle(this->graphicsQueue);

    vkFreeCommandBuffers(this->device, this->commandPool, 1, &commandBuffer);
}

void VulkanRenderer::uploadTextureArray(
    const TextureDescriptor* descriptors_ptr, 
    size_t descriptor_count, 
    const uint8_t* all_pixels_ptr, 
    size_t all_pixels_count, 
    const float* sky_widths_ptr, 
    size_t sky_widths_count
) {
    if (descriptors_ptr == nullptr) {
        throw std::runtime_error("descriptors_ptr is empty!");
    }
    if (all_pixels_ptr == nullptr) {
        throw std::runtime_error("all_pixels_ptr is empty!");
    }
    this->skyWidths.resize(sky_widths_count);
    memcpy(this->skyWidths.data(), sky_widths_ptr, sizeof(float) * sky_widths_count);

    std::vector<TextureDescriptor> descriptors(descriptors_ptr, descriptors_ptr + descriptor_count);

    VkBuffer pixelStagingBuffer;
    VkDeviceMemory pixelStagingBufferMemory;
    createBuffer(all_pixels_count, VK_BUFFER_USAGE_TRANSFER_SRC_BIT, 
                 VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT, 
                 pixelStagingBuffer, pixelStagingBufferMemory);

    void* pixelsData;
    vkMapMemory(this->device, pixelStagingBufferMemory, 0, all_pixels_count, 0, &pixelsData);
        memcpy(pixelsData, all_pixels_ptr, all_pixels_count);
    vkUnmapMemory(this->device, pixelStagingBufferMemory);

    this->textureImages.resize(descriptor_count);
    this->textureImageViews.resize(descriptor_count);
    this->textureImageMemories.resize(descriptor_count);
    for (size_t i = 0; i < descriptor_count; i++) {
        createImage(
            descriptors[i].width, 
            descriptors[i].height, 
            VK_FORMAT_R8_UNORM,
            VK_IMAGE_USAGE_TRANSFER_DST_BIT | VK_IMAGE_USAGE_SAMPLED_BIT,
            VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT, 
            this->textureImages[i], 
            this->textureImageMemories[i]
        );
    }
    VkCommandBuffer commandBuffer = beginSingleTimeCommands();
    
    std::vector<VkImageMemoryBarrier> preCopyBarriers(descriptor_count);
    for (size_t i = 0; i < descriptor_count; i++) {
        preCopyBarriers[i].sType = VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER;
        preCopyBarriers[i].oldLayout = VK_IMAGE_LAYOUT_UNDEFINED;
        preCopyBarriers[i].newLayout = VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL;
        preCopyBarriers[i].srcQueueFamilyIndex = VK_QUEUE_FAMILY_IGNORED;
        preCopyBarriers[i].dstQueueFamilyIndex = VK_QUEUE_FAMILY_IGNORED;
        preCopyBarriers[i].image = this->textureImages[i];
        preCopyBarriers[i].subresourceRange.aspectMask = VK_IMAGE_ASPECT_COLOR_BIT;
        preCopyBarriers[i].subresourceRange.baseMipLevel = 0;
        preCopyBarriers[i].subresourceRange.levelCount = 1;
        preCopyBarriers[i].subresourceRange.baseArrayLayer = 0;
        preCopyBarriers[i].subresourceRange.layerCount = 1;
        preCopyBarriers[i].srcAccessMask = 0;
        preCopyBarriers[i].dstAccessMask = VK_ACCESS_TRANSFER_WRITE_BIT;
    }
    vkCmdPipelineBarrier(commandBuffer, VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT, VK_PIPELINE_STAGE_TRANSFER_BIT, 
                         0, 0, nullptr, 0, nullptr, static_cast<uint32_t>(preCopyBarriers.size()), preCopyBarriers.data());

    for (size_t i = 0; i < descriptor_count; i++) {
        const auto& desc = descriptors[i];
        VkBufferImageCopy region{};
        region.bufferOffset = desc.pixel_offset;
        region.imageSubresource.aspectMask = VK_IMAGE_ASPECT_COLOR_BIT;
        region.imageSubresource.layerCount = 1;
        region.imageExtent = { desc.width, desc.height, 1 };

        vkCmdCopyBufferToImage(commandBuffer, pixelStagingBuffer, this->textureImages[i], 
                               VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL, 1, &region);
    }

    std::vector<VkImageMemoryBarrier> postCopyBarriers(descriptor_count);
    for (size_t i = 0; i < descriptor_count; i++) {
        postCopyBarriers[i].sType = VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER;
        postCopyBarriers[i].oldLayout = VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL;
        postCopyBarriers[i].newLayout = VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL;
        postCopyBarriers[i].srcQueueFamilyIndex = VK_QUEUE_FAMILY_IGNORED;
        postCopyBarriers[i].dstQueueFamilyIndex = VK_QUEUE_FAMILY_IGNORED;
        postCopyBarriers[i].image = this->textureImages[i];
        postCopyBarriers[i].subresourceRange.aspectMask = VK_IMAGE_ASPECT_COLOR_BIT;
        postCopyBarriers[i].subresourceRange.baseMipLevel = 0;
        postCopyBarriers[i].subresourceRange.levelCount = 1;
        postCopyBarriers[i].subresourceRange.baseArrayLayer = 0;
        postCopyBarriers[i].subresourceRange.layerCount = 1;
        postCopyBarriers[i].srcAccessMask = VK_ACCESS_TRANSFER_WRITE_BIT;
        postCopyBarriers[i].dstAccessMask = VK_ACCESS_SHADER_READ_BIT;
    }
    vkCmdPipelineBarrier(commandBuffer, VK_PIPELINE_STAGE_TRANSFER_BIT, VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT, 
                         0, 0, nullptr, 0, nullptr, static_cast<uint32_t>(postCopyBarriers.size()), postCopyBarriers.data());
        
    endSingleTimeCommands(commandBuffer);

    for (size_t i = 0; i < descriptor_count; i++) {
        createImageView(
            this->device, 
            this->textureImages[i], 
            VK_FORMAT_R8_UNORM, 
            VK_IMAGE_ASPECT_COLOR_BIT,
            &this->textureImageViews[i]
        );
    }

    vkDestroyBuffer(this->device, pixelStagingBuffer, nullptr);
    vkFreeMemory(this->device, pixelStagingBufferMemory, nullptr);

    std::vector<VkDescriptorImageInfo> imageInfos(descriptor_count);
    for (size_t i = 0; i < descriptor_count; i++) {
        imageInfos[i].imageLayout = VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL;
        imageInfos[i].imageView = this->textureImageViews[i];
        imageInfos[i].sampler = this->textureSampler;
    }

    for (size_t i = 0; i < MAX_FRAMES_IN_FLIGHT; i++) {
        VkWriteDescriptorSet descriptorWrite{};
        descriptorWrite.sType = VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET;
        descriptorWrite.dstSet = this->descriptorSets[i];
        descriptorWrite.dstBinding = 3;
        descriptorWrite.dstArrayElement = 0;
        descriptorWrite.descriptorType = VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER;
        descriptorWrite.descriptorCount = static_cast<uint32_t>(descriptors.size());
        descriptorWrite.pImageInfo = imageInfos.data();

        vkUpdateDescriptorSets(this->device, 1, &descriptorWrite, 0, nullptr);
    }

    std::cout << "Bound " << descriptor_count << " textures to Bindless Set" << std::endl;
}

void VulkanRenderer::createImage(uint32_t width, uint32_t height, VkFormat format, VkImageUsageFlags usage, VkMemoryPropertyFlags properties, VkImage& image, VkDeviceMemory& imageMemory) {
    VkImageCreateInfo imageInfo{};
    imageInfo.sType = VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO;
    imageInfo.imageType = VK_IMAGE_TYPE_2D;
    imageInfo.extent.width = width;
    imageInfo.extent.height = height;
    imageInfo.extent.depth = 1;
    imageInfo.mipLevels = 1;
    imageInfo.arrayLayers = 1;
    imageInfo.format = format;
    imageInfo.tiling = VK_IMAGE_TILING_OPTIMAL;
    imageInfo.initialLayout = VK_IMAGE_LAYOUT_UNDEFINED;
    imageInfo.usage = usage;
    imageInfo.samples = VK_SAMPLE_COUNT_1_BIT;
    imageInfo.sharingMode = VK_SHARING_MODE_EXCLUSIVE;

    if (vkCreateImage(this->device, &imageInfo, nullptr, &image) != VK_SUCCESS) {
        throw std::runtime_error("failed to create image!");
    }

    VkMemoryRequirements memRequirements;
    vkGetImageMemoryRequirements(this->device, image, &memRequirements);

    VkMemoryAllocateInfo allocInfo{};
    allocInfo.sType = VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO;
    allocInfo.allocationSize = memRequirements.size;
    allocInfo.memoryTypeIndex = findMemoryType(this->physicalDevice, memRequirements.memoryTypeBits, properties);

    if (vkAllocateMemory(this->device, &allocInfo, nullptr, &imageMemory) != VK_SUCCESS) {
        throw std::runtime_error("failed to allocate image memory!");
    }

    vkBindImageMemory(this->device, image, imageMemory, 0);
}

void VulkanRenderer::createTextureSamplers() {
	VkSamplerCreateInfo textureSamplerInfo{};

	textureSamplerInfo.sType = VK_STRUCTURE_TYPE_SAMPLER_CREATE_INFO;
	textureSamplerInfo.magFilter = VK_FILTER_NEAREST;
	textureSamplerInfo.minFilter = VK_FILTER_NEAREST;
	textureSamplerInfo.addressModeU = VK_SAMPLER_ADDRESS_MODE_REPEAT;
	textureSamplerInfo.addressModeV = VK_SAMPLER_ADDRESS_MODE_REPEAT;
	textureSamplerInfo.addressModeW = VK_SAMPLER_ADDRESS_MODE_REPEAT;
	textureSamplerInfo.anisotropyEnable = VK_TRUE;

	VkPhysicalDeviceProperties properties{};
	vkGetPhysicalDeviceProperties(this->physicalDevice, &properties);

	textureSamplerInfo.maxAnisotropy = properties.limits.maxSamplerAnisotropy;
	textureSamplerInfo.borderColor = VK_BORDER_COLOR_INT_OPAQUE_BLACK;
	textureSamplerInfo.unnormalizedCoordinates = VK_FALSE;
	textureSamplerInfo.mipmapMode = VK_SAMPLER_MIPMAP_MODE_NEAREST;
	textureSamplerInfo.mipLodBias = 0.0f;
	textureSamplerInfo.minLod = 0.0f;
	textureSamplerInfo.maxLod = 0.0f;
	
	VkResult textureSamplerResult = vkCreateSampler(this->device, &textureSamplerInfo, nullptr, &this->textureSampler);
	if (textureSamplerResult != VK_SUCCESS) {
        throw std::runtime_error("failed to create texture sampler!");
    }
}

void VulkanRenderer::uploadPalettes(const float* palettes_ptr) {
    VkDeviceSize bufferSize = MAX_PAL * 256 * 4 * sizeof(float);

    VkBuffer stagingBuffer;
    VkDeviceMemory stagingBufferMemory;
    
    createBuffer(
        bufferSize, 
        VK_BUFFER_USAGE_TRANSFER_SRC_BIT, 
        VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT, 
        stagingBuffer, 
        stagingBufferMemory
    );

    void* data;
    vkMapMemory(this->device, stagingBufferMemory, 0, bufferSize, 0, &data);
    memcpy(data, palettes_ptr, bufferSize);
    vkUnmapMemory(this->device, stagingBufferMemory);

    createBuffer(
        bufferSize, 
        VK_BUFFER_USAGE_STORAGE_BUFFER_BIT | VK_BUFFER_USAGE_TRANSFER_DST_BIT, 
        VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT, 
        this->paletteBuffer, 
        this->paletteBufferMemory 
    );

    copyBuffer(stagingBuffer, this->paletteBuffer, bufferSize);

    vkDestroyBuffer(this->device, stagingBuffer, nullptr);
    vkFreeMemory(this->device, stagingBufferMemory, nullptr);

	for (size_t i = 0; i < MAX_FRAMES_IN_FLIGHT; i++) {
        VkDescriptorBufferInfo bufferInfo{};
        bufferInfo.buffer = this->paletteBuffer;
        bufferInfo.offset = 0;
        bufferInfo.range = bufferSize;

        VkWriteDescriptorSet descriptorWrite{};
        descriptorWrite.sType = VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET;
        descriptorWrite.dstSet = this->descriptorSets[i];
        descriptorWrite.dstBinding = 1;
        descriptorWrite.descriptorCount = 1;
        descriptorWrite.descriptorType = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER;
        descriptorWrite.pBufferInfo = &bufferInfo;

        vkUpdateDescriptorSets(this->device, 1, &descriptorWrite, 0, nullptr);
    }
}

void VulkanRenderer::uploadColormap(const uint8_t* colormap_ptr) {
    VkDeviceSize bufferSize = MAX_LIGHTLEVEL * 256 * sizeof(uint8_t);

    VkBuffer stagingBuffer;
    VkDeviceMemory stagingBufferMemory;
    
    createBuffer(
        bufferSize, 
        VK_BUFFER_USAGE_TRANSFER_SRC_BIT, 
        VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT, 
        stagingBuffer, 
        stagingBufferMemory
    );

    void* data;
    vkMapMemory(this->device, stagingBufferMemory, 0, bufferSize, 0, &data);
    memcpy(data, colormap_ptr, bufferSize);
    vkUnmapMemory(this->device, stagingBufferMemory);

    createBuffer(
        bufferSize, 
        VK_BUFFER_USAGE_STORAGE_BUFFER_BIT | VK_BUFFER_USAGE_TRANSFER_DST_BIT, 
        VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT, 
        this->colormapBuffer, 
        this->colormapBufferMemory 
    );

    copyBuffer(stagingBuffer, this->colormapBuffer, bufferSize);

    vkDestroyBuffer(this->device, stagingBuffer, nullptr);
    vkFreeMemory(this->device, stagingBufferMemory, nullptr);

	for (size_t i = 0; i < MAX_FRAMES_IN_FLIGHT; i++) {
        VkDescriptorBufferInfo bufferInfo{};
        bufferInfo.buffer = this->colormapBuffer;
        bufferInfo.offset = 0;
        bufferInfo.range = bufferSize;

        VkWriteDescriptorSet descriptorWrite{};
        descriptorWrite.sType = VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET;
        descriptorWrite.dstSet = this->descriptorSets[i];
        descriptorWrite.dstBinding = 2;
        descriptorWrite.descriptorCount = 1;
        descriptorWrite.descriptorType = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER;
        descriptorWrite.pBufferInfo = &bufferInfo;

        vkUpdateDescriptorSets(this->device, 1, &descriptorWrite, 0, nullptr);
    }
}