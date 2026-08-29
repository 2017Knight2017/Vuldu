#include <cstring>
#include <stdexcept>
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
    if (descriptor_count == 0 || descriptors_ptr == nullptr)
        throw std::runtime_error("No textures to load!");

    if (descriptor_count > MAX_TEXTURES) 
        throw std::runtime_error("Too many textures to load! (>8192)");
    
    if (all_pixels_count == 0 || all_pixels_ptr == nullptr)
        throw std::runtime_error("Pixels vector is empty!");
    
    this->skyWidths.resize(sky_widths_count);
    memcpy(this->skyWidths.data(), sky_widths_ptr, sky_widths_count * sizeof(float));

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

    changeImageLayout(
        commandBuffer, 
        VK_IMAGE_LAYOUT_UNDEFINED, 
        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL, 
        this->textureImages
    );

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

    changeImageLayout(
        commandBuffer, 
        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL, 
        VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL, 
        this->textureImages
    );
       
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
        descriptorWrite.dstBinding = 4;
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

void VulkanRenderer::createBufferBinding(
	const void* data_ptr, 
	VkDeviceSize bufferSize, 
	VkBuffer& dstBuffer, 
	VkDeviceMemory& dstBufferMemory,
    uint32_t dstBinding
) {
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
    memcpy(data, data_ptr, bufferSize);
    vkUnmapMemory(this->device, stagingBufferMemory);

    VkBufferUsageFlags usage = VK_BUFFER_USAGE_TRANSFER_DST_BIT | VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT;

    createBuffer(
        bufferSize, 
        usage, 
        VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT, 
        dstBuffer, 
        dstBufferMemory
    );

    copyBuffer(stagingBuffer, dstBuffer, bufferSize);

    vkDestroyBuffer(this->device, stagingBuffer, nullptr);
    vkFreeMemory(this->device, stagingBufferMemory, nullptr);

	for (size_t i = 0; i < MAX_FRAMES_IN_FLIGHT; i++) {
        VkDescriptorBufferInfo bufferInfo{};
        bufferInfo.buffer = dstBuffer;
        bufferInfo.offset = 0;
        bufferInfo.range = bufferSize;

        VkWriteDescriptorSet descriptorWrite{};
        descriptorWrite.sType = VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET;
        descriptorWrite.dstSet = this->descriptorSets[i];
        descriptorWrite.dstBinding = dstBinding;
        descriptorWrite.descriptorCount = 1;
        descriptorWrite.descriptorType = VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER;
        descriptorWrite.pBufferInfo = &bufferInfo;

        vkUpdateDescriptorSets(this->device, 1, &descriptorWrite, 0, nullptr);
    }
}

void VulkanRenderer::createDataTexture(
    const void* data_ptr, 
    size_t width,
    size_t height,
    size_t colorSize,
    VkFormat format,
    VkImage& dstImage, 
    VkDeviceMemory& dstImageMemory,
    VkImageView& dstImageView,
    uint32_t dstBinding
) {
    VkDeviceSize imageSize = width * height * colorSize;

    VkBuffer stagingBuffer;
    VkDeviceMemory stagingBufferMemory;
    createBuffer(
        imageSize, 
        VK_BUFFER_USAGE_TRANSFER_SRC_BIT, 
        VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT, 
        stagingBuffer, 
        stagingBufferMemory
    );

    void* data;
    vkMapMemory(this->device, stagingBufferMemory, 0, imageSize, 0, &data);
    memcpy(data, data_ptr, imageSize);
    vkUnmapMemory(this->device, stagingBufferMemory);

    createImage(
        width, height, format, 
        VK_IMAGE_USAGE_TRANSFER_DST_BIT | VK_IMAGE_USAGE_SAMPLED_BIT, 
        VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT, 
        dstImage, dstImageMemory
    );

    VkCommandBuffer commandBuffer = beginSingleTimeCommands();

    changeImageLayout(
        commandBuffer, 
        VK_IMAGE_LAYOUT_UNDEFINED, 
        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL, 
        {&dstImage, 1}
    );
    
    VkBufferImageCopy region{};
    region.imageSubresource.aspectMask = VK_IMAGE_ASPECT_COLOR_BIT;
    region.imageSubresource.layerCount = 1;
    region.imageExtent = { 
        static_cast<uint32_t>(width), 
        static_cast<uint32_t>(height), 
        1 
    };

    vkCmdCopyBufferToImage(commandBuffer, stagingBuffer, dstImage, 
                           VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL, 1, &region);

    changeImageLayout(
        commandBuffer, 
        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL, 
        VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL, 
        {&dstImage, 1}
    );

    endSingleTimeCommands(commandBuffer);

    createImageView(this->device, dstImage, format, VK_IMAGE_ASPECT_COLOR_BIT, &dstImageView);
    
    vkDestroyBuffer(this->device, stagingBuffer, nullptr);
    vkFreeMemory(this->device, stagingBufferMemory, nullptr);

    for (size_t i = 0; i < MAX_FRAMES_IN_FLIGHT; i++) {
        VkDescriptorImageInfo imageInfo{};
        imageInfo.imageLayout = VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL;
        imageInfo.imageView = dstImageView;
        imageInfo.sampler = this->textureSampler;

        VkWriteDescriptorSet descriptorWrite{};
        descriptorWrite.sType = VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET;
        descriptorWrite.dstSet = this->descriptorSets[i];
        descriptorWrite.dstBinding = dstBinding;
        descriptorWrite.descriptorCount = 1;
        descriptorWrite.descriptorType = VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER;
        descriptorWrite.pImageInfo = &imageInfo;

        vkUpdateDescriptorSets(this->device, 1, &descriptorWrite, 0, nullptr);
    }
}

void VulkanRenderer::uploadPalettes(const uint8_t* palettes_ptr, size_t palette_channels_count) {
    if (palette_channels_count == 0 || palettes_ptr == nullptr) return;

    size_t colorsCount = palette_channels_count >> 2;

    createDataTexture(
        palettes_ptr, 
        256,
        colorsCount / 256,
        sizeof(float),
        VK_FORMAT_R8G8B8A8_UNORM, 
        this->paletteImage, this->paletteImageMemory, this->paletteImageView, 
        1
    );
}

void VulkanRenderer::uploadColormap(const uint8_t* colormap_ptr, size_t colormap_bytes_count) {
    if (colormap_bytes_count == 0 || colormap_ptr == nullptr) return;

    createDataTexture(
        colormap_ptr, 
        256,
        colormap_bytes_count / 256,
        sizeof(uint8_t),
        VK_FORMAT_R8_UINT, 
        this->colormapImage, this->colormapImageMemory, this->colormapImageView, 
        2
    );
}

void VulkanRenderer::uploadAnimLevelInfo(const AnimLevelInfo* info_ptr, size_t info_count) {
    if (info_count == 0 || info_ptr == nullptr) return;

    VkDeviceSize bufferSize = (MAX_TEXTURES >> 2) * sizeof(AnimLevelInfo);

    createBufferBinding(info_ptr, bufferSize, this->animLevelBuffer, 
        this->animLevelBufferMemory, 3);
}
