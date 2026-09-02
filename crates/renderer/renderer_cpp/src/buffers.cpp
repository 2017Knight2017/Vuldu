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
    rust::Slice<const TextureDescriptor> descriptors, 
    rust::Slice<const uint8_t> pixels, 
    rust::Slice<const float> sky_widths
) {
    if (descriptors.empty()) throw std::runtime_error("No textures to load!");
    if (descriptors.size() > MAX_TEXTURES) throw std::runtime_error("Too many textures to load! (>8192)");
    if (pixels.empty()) throw std::runtime_error("Pixels vector is empty!");
    
    this->skyWidths.resize(sky_widths.size());
    memcpy(this->skyWidths.data(), sky_widths.data(), sky_widths.size() * sizeof(float));

    VkBuffer pixelStagingBuffer;
    VkDeviceMemory pixelStagingBufferMemory;
    createBuffer(pixels.size(), VK_BUFFER_USAGE_TRANSFER_SRC_BIT, 
                 VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT, 
                 pixelStagingBuffer, pixelStagingBufferMemory);

    void* pixelsData;
    vkMapMemory(this->device, pixelStagingBufferMemory, 0, pixels.size(), 0, &pixelsData);
        memcpy(pixelsData, pixels.data(), pixels.size());
    vkUnmapMemory(this->device, pixelStagingBufferMemory);

    this->textureImages.resize(descriptors.size());
    this->textureImageViews.resize(descriptors.size());
    this->textureImageMemories.resize(descriptors.size());
    for (size_t i = 0; i < descriptors.size(); i++) {
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

    for (size_t i = 0; i < descriptors.size(); i++) {
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

    for (size_t i = 0; i < descriptors.size(); i++) {
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

    std::vector<VkDescriptorImageInfo> imageInfos(descriptors.size());
    for (size_t i = 0; i < descriptors.size(); i++) {
        imageInfos[i].imageLayout = VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL;
        imageInfos[i].imageView = this->textureImageViews[i];
        imageInfos[i].sampler = this->textureSampler;
    }

    for (size_t i = 0; i < MAX_FRAMES_IN_FLIGHT; i++) {
        VkWriteDescriptorSet descriptorWrite{};
        descriptorWrite.sType = VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET;
        descriptorWrite.dstSet = this->descriptorSets[i];
        descriptorWrite.dstBinding = 5;
        descriptorWrite.dstArrayElement = 0;
        descriptorWrite.descriptorType = VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER;
        descriptorWrite.descriptorCount = static_cast<uint32_t>(descriptors.size());
        descriptorWrite.pImageInfo = imageInfos.data();

        vkUpdateDescriptorSets(this->device, 1, &descriptorWrite, 0, nullptr);
    }

    std::cout << "Bound " << descriptors.size() << " textures to Bindless Set" << std::endl;
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

    createBuffer(
        bufferSize, 
        VK_BUFFER_USAGE_TRANSFER_DST_BIT |
        VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT, 
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
    VkFormat format,
    VkImage& dstImage, 
    VkDeviceMemory& dstImageMemory,
    VkImageView& dstImageView,
    uint32_t dstBinding
) {
    size_t channelsInColor;

    switch (format) 
    {
    case VK_FORMAT_R8G8B8A8_UNORM:
        channelsInColor = 4;
        break;
    
    case VK_FORMAT_R8_UINT:
        channelsInColor = 1;
        break;

    default:
        channelsInColor = 1;
    }

    VkDeviceSize imageSize = width * height * channelsInColor;

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

void VulkanRenderer::uploadPalettes(rust::Slice<const uint8_t> palettes) {
    if (palettes.empty()) return;

    size_t colorsCount = palettes.size() >> 2;

    createDataTexture(
        palettes.data(), 
        256,
        colorsCount / 256,
        VK_FORMAT_R8G8B8A8_UNORM, 
        this->paletteImage, this->paletteImageMemory, this->paletteImageView, 
        1
    );
}

void VulkanRenderer::uploadColormap(rust::Slice<const uint8_t> colormap) {
    if (colormap.empty()) return;

    createDataTexture(
        colormap.data(), 
        256,
        colormap.size() / 256,
        VK_FORMAT_R8_UINT, 
        this->colormapImage, this->colormapImageMemory, this->colormapImageView, 
        2
    );
}

void VulkanRenderer::uploadAnimLevelInfo(rust::Slice<const AnimLevelInfo> info) {
    if (info.size() != ANIM_INFO_SIZE) return;

    VkDeviceSize bufferSize = info.size() * sizeof(AnimLevelInfo);

    createBufferBinding(info.data(), bufferSize, this->animLevelBuffer, 
        this->animLevelBufferMemory, 3);
}

void VulkanRenderer::initSectorHeights(rust::Slice<const float> heights) {
    if (heights.empty()) return;

    VkDeviceSize bufferSize = heights.size() * sizeof(float);

    createBuffer(
        bufferSize,
        VK_BUFFER_USAGE_STORAGE_BUFFER_BIT,
        VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT,
        this->sectorHeightsBuffer,
        this->sectorHeightsBufferMemory
    );

    vkMapMemory(device, sectorHeightsBufferMemory, 0, bufferSize, 0, &this->sectorHeightsBufferMapped);
    memcpy(this->sectorHeightsBufferMapped, heights.data(), bufferSize);

    for (size_t i = 0; i < MAX_FRAMES_IN_FLIGHT; i++) {
        VkDescriptorBufferInfo bufferInfo{};
        bufferInfo.buffer = this->sectorHeightsBuffer;
        bufferInfo.offset = 0;
        bufferInfo.range = bufferSize;

        VkWriteDescriptorSet descriptorWrite{};
        descriptorWrite.sType = VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET;
        descriptorWrite.dstSet = this->descriptorSets[i];
        descriptorWrite.dstBinding = 4;
        descriptorWrite.descriptorCount = 1;
        descriptorWrite.descriptorType = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER;
        descriptorWrite.pBufferInfo = &bufferInfo;

        vkUpdateDescriptorSets(this->device, 1, &descriptorWrite, 0, nullptr);
    }
}

void VulkanRenderer::updateSectorHeights(rust::Slice<const float> heights) {
    VkDeviceSize bufferSize = heights.size() * sizeof(float);

    memcpy(this->sectorHeightsBufferMapped, heights.data(), bufferSize);
}
