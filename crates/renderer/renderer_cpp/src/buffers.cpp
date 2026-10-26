#include <cstring>
#include <stdexcept>
#include <iostream>
#include "renderer.h"
#include "renderer/src/bridge.rs.h"
#include "utils.h"

std::pair<vk::raii::Buffer, vk::raii::DeviceMemory> VulkanRenderer::createBuffer(
	vk::DeviceSize bufferSize, 
	vk::BufferUsageFlags usage, 
	vk::MemoryPropertyFlags properties 
) {
	vk::BufferCreateInfo bufferInfo;
	bufferInfo
		.setSize(bufferSize)
		.setUsage(usage);
	
	vk::raii::Buffer buffer{this->device, bufferInfo};

	vk::MemoryRequirements memRequirements = buffer.getMemoryRequirements();

	vk::raii::DeviceMemory bufferMemory = this->device.allocateMemory({
		memRequirements.size,
 		findMemoryType(this->physicalDevice, memRequirements.memoryTypeBits, properties)
	});

	buffer.bindMemory(bufferMemory, 0);

	return std::make_pair(std::move(buffer), std::move(bufferMemory));
}

void VulkanRenderer::copyBuffer(const vk::raii::Buffer& srcBuffer, const vk::raii::Buffer& dstBuffer, vk::DeviceSize size) {
	vk::raii::CommandBuffer commandBuffer = beginSingleTimeCommands();

	vk::BufferCopy copyRegion{};
	copyRegion.setSize(size);

	commandBuffer.copyBuffer(srcBuffer, dstBuffer, copyRegion);

	endSingleTimeCommands(commandBuffer);
}

vk::raii::CommandBuffer VulkanRenderer::beginSingleTimeCommands() {
	vk::CommandBufferAllocateInfo allocInfo;
	allocInfo
		.setLevel(vk::CommandBufferLevel::ePrimary)
		.setCommandPool(*this->commandPool)
		.setCommandBufferCount(1);

	vk::raii::CommandBuffer commandBuffer = std::move(
		this->device.allocateCommandBuffers(allocInfo)[0]
	);

	commandBuffer.begin({vk::CommandBufferUsageFlags::BitsType::eOneTimeSubmit});

	return commandBuffer;
}

void VulkanRenderer::endSingleTimeCommands(vk::raii::CommandBuffer& commandBuffer) {
	commandBuffer.end();

	vk::SubmitInfo submitInfo;
	submitInfo.setCommandBuffers(*commandBuffer);

	this->graphicsQueue.submit(submitInfo);
	this->graphicsQueue.waitIdle();
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

	auto [pixelStagingBuffer, pixelStagingBufferMemory] = createBuffer(
		pixels.size(), 
		vk::BufferUsageFlagBits::eTransferSrc, 
		vk::MemoryPropertyFlagBits::eHostVisible | vk::MemoryPropertyFlagBits::eHostCoherent
	);

	void* pixelsData = pixelStagingBufferMemory.mapMemory(0, pixels.size(), {});
	memcpy(pixelsData, pixels.data(), pixels.size());
	pixelStagingBufferMemory.unmapMemory();

	this->textureImages.reserve(descriptors.size());
	this->textureImageMemories.reserve(descriptors.size());
	for (size_t i = 0; i < descriptors.size(); ++i) {
		auto [image, memory] = createImage(
			descriptors[i].width, 
			descriptors[i].height, 
			vk::Format::eR8Unorm,
			vk::ImageUsageFlagBits::eTransferDst | vk::ImageUsageFlagBits::eSampled,
			vk::MemoryPropertyFlags::BitsType::eDeviceLocal
		);

		this->textureImages.push_back(std::move(image));
		this->textureImageMemories.push_back(std::move(memory));
	}

	vk::raii::CommandBuffer commandBuffer = beginSingleTimeCommands();

	changeMultipleImageLayout(
		commandBuffer, 
		vk::ImageLayout::eUndefined, 
		vk::ImageLayout::eTransferDstOptimal, 
		this->textureImages
	);

	for (size_t i = 0; i < descriptors.size(); ++i) {
		const auto& desc = descriptors[i];
		vk::BufferImageCopy region;
		region
			.setBufferOffset(desc.pixel_offset)
			.setImageSubresource({
				vk::ImageAspectFlagBits::eColor,
				0,
				0,
				1
			})
			.setImageExtent({desc.width, desc.height, 1});
		
		commandBuffer.copyBufferToImage(
			pixelStagingBuffer, 
			this->textureImages[i], 
			vk::ImageLayout::eTransferDstOptimal,
			region
		);
	}

	changeMultipleImageLayout(
		commandBuffer, 
		vk::ImageLayout::eTransferDstOptimal, 
		vk::ImageLayout::eShaderReadOnlyOptimal, 
		this->textureImages
	);
	   
	endSingleTimeCommands(commandBuffer);

	this->textureImageViews.reserve(descriptors.size());
	for (size_t i = 0; i < descriptors.size(); ++i) {
		this->textureImageViews.push_back(createImageView(
			this->device, 
			this->textureImages[i], 
			vk::Format::eR8Unorm, 
			vk::ImageAspectFlagBits::eColor
		));
	}

	std::vector<vk::DescriptorImageInfo> imageInfos(descriptors.size());
	for (size_t i = 0; i < descriptors.size(); ++i) {
		imageInfos[i]
			.setImageLayout(vk::ImageLayout::eShaderReadOnlyOptimal)
			.setImageView(this->textureImageViews[i])
			.setSampler(*this->textureSampler);
	}

	for (size_t i = 0; i < MAX_FRAMES_IN_FLIGHT; ++i) {
		vk::WriteDescriptorSet descriptorWrite;
		descriptorWrite
			.setDstSet(this->descriptorSets[i])
			.setDstBinding(6)
			.setDescriptorType(vk::DescriptorType::eCombinedImageSampler)
			.setDescriptorCount(static_cast<uint32_t>(descriptors.size()))
			.setImageInfo(imageInfos);

		this->device.updateDescriptorSets(descriptorWrite, {});
	}

	std::cout << "Bound " << descriptors.size() << " textures to Bindless Set\n";
}

std::pair<vk::raii::Image, vk::raii::DeviceMemory> VulkanRenderer::createImage(
	uint32_t width, 
	uint32_t height, 
	vk::Format format, 
	vk::ImageUsageFlags usage, 
	vk::MemoryPropertyFlags properties
) {
	vk::ImageCreateInfo imageInfo{
		{},
		vk::ImageType::e2D,
		format,
		{width, height, 1},
		1,
		1,
		vk::SampleCountFlagBits::e1,
		{},
		usage
	};

	vk::raii::Image image{this->device, imageInfo};

	vk::MemoryRequirements memRequirements = image.getMemoryRequirements();

	vk::raii::DeviceMemory imageMemory{this->device, {
		memRequirements.size,
 		findMemoryType(this->physicalDevice, memRequirements.memoryTypeBits, properties)
	}};

	image.bindMemory(imageMemory, 0);

	return std::make_pair(std::move(image), std::move(imageMemory));
}

void VulkanRenderer::createTextureSamplers() {
	vk::PhysicalDeviceProperties properties = this->physicalDevice.getProperties();

	vk::SamplerCreateInfo textureSamplerInfo;
	textureSamplerInfo
		.setAnisotropyEnable(VK_TRUE)
		.setMaxAnisotropy(properties.limits.maxSamplerAnisotropy)
		.setBorderColor(vk::BorderColor::eFloatOpaqueBlack);
	
	this->textureSampler = this->device.createSampler(textureSamplerInfo);
}

DataTexture VulkanRenderer::createDataTexture(
	const void* data_ptr, 
	uint32_t width,
	uint32_t height,
	vk::Format format,
	uint32_t dstBinding
) {
	uint32_t channelsInColor;

	switch (format) {
		case vk::Format::eR8G8B8A8Unorm:
			channelsInColor = 4;
			break;
		
		case vk::Format::eR8Uint:
			channelsInColor = 1;
			break;

		default:
			channelsInColor = 1;
	}

	vk::DeviceSize imageSize = width * height * channelsInColor;

	auto [stagingBuffer, stagingBufferMemory] = createBuffer(
		imageSize, 
		vk::BufferUsageFlagBits::eTransferSrc, 
		vk::MemoryPropertyFlagBits::eHostVisible | vk::MemoryPropertyFlagBits::eHostCoherent 
	);

	void* data = stagingBufferMemory.mapMemory(0, imageSize, {});
	memcpy(data, data_ptr, imageSize);
	stagingBufferMemory.unmapMemory();

	auto [image, imageMemory] = createImage(
		width, 
		height,
		format,
		vk::ImageUsageFlags::BitsType::eTransferDst | vk::ImageUsageFlags::BitsType::eSampled, 
		vk::MemoryPropertyFlags::BitsType::eDeviceLocal
	);

	vk::raii::CommandBuffer commandBuffer = beginSingleTimeCommands();

	changeSingleImageLayout(
		commandBuffer, 
		vk::ImageLayout::eUndefined, 
		vk::ImageLayout::eTransferDstOptimal, 
		image
	);
	
	vk::BufferImageCopy region;
	region
		.setImageSubresource({vk::ImageAspectFlagBits::eColor, 0, 0, 1})
		.setImageExtent({width, height, 1});

	commandBuffer.copyBufferToImage(
		stagingBuffer, 
		image, 
		vk::ImageLayout::eTransferDstOptimal, 
		region
	);

	changeSingleImageLayout(
		commandBuffer, 
		vk::ImageLayout::eTransferDstOptimal, 
		vk::ImageLayout::eReadOnlyOptimal, 
		image
	);

	endSingleTimeCommands(commandBuffer);

	vk::raii::ImageView imageView = createImageView(
		this->device, 
		image, 
		format, 
		vk::ImageAspectFlagBits::eColor
	);

	for (size_t i = 0; i < MAX_FRAMES_IN_FLIGHT; ++i) {
		vk::DescriptorImageInfo imageInfo;
		imageInfo
			.setImageLayout(vk::ImageLayout::eShaderReadOnlyOptimal)
			.setImageView(imageView)
			.setSampler(this->textureSampler);

		vk::WriteDescriptorSet descriptorWrite;
		descriptorWrite
			.setDstSet(this->descriptorSets[i])
			.setDstBinding(dstBinding)
			.setDescriptorType(vk::DescriptorType::eCombinedImageSampler)
			.setImageInfo(imageInfo);

		this->device.updateDescriptorSets(descriptorWrite, {});
	}

	return DataTexture{std::move(image), std::move(imageMemory), std::move(imageView)};
}

void VulkanRenderer::uploadPalettes(rust::Slice<const uint8_t> palettes) {
	if (palettes.empty()) return;

	size_t colorsCount = palettes.size() / 4;

	this->palette = createDataTexture(
		palettes.data(), 
		256,
		colorsCount / 256,
		vk::Format::eR8G8B8A8Unorm, 
		1
	);
}

void VulkanRenderer::uploadColormap(rust::Slice<const uint8_t> colormap) {
	if (colormap.empty()) return;

	this->colormap = createDataTexture(
		colormap.data(), 
		256,
		colormap.size() / 256,
		vk::Format::eR8Uint, 
		2
	);
}

void VulkanRenderer::uploadAnimLevelInfo(rust::Slice<const AnimLevelInfo> info) {
	if (info.size() != ANIM_INFO_SIZE) return;

	vk::DeviceSize bufferSize = info.size() * sizeof(AnimLevelInfo);
	
	auto [stagingBuffer, stagingBufferMemory] = createBuffer(
		bufferSize, 
		vk::BufferUsageFlagBits::eTransferSrc, 
		vk::MemoryPropertyFlagBits::eHostVisible | vk::MemoryPropertyFlagBits::eHostCoherent
	);

	void* data = stagingBufferMemory.mapMemory(0, bufferSize, {});
	memcpy(data, info.data(), bufferSize);
	stagingBufferMemory.unmapMemory();

	std::tie(this->animLevel.buffer, this->animLevel.memory) = createBuffer(
		bufferSize, 
		vk::BufferUsageFlagBits::eTransferDst | vk::BufferUsageFlagBits::eUniformBuffer, 
		vk::MemoryPropertyFlagBits::eDeviceLocal
	);

	copyBuffer(stagingBuffer, this->animLevel.buffer, bufferSize);

	for (size_t i = 0; i < MAX_FRAMES_IN_FLIGHT; ++i) {
		vk::DescriptorBufferInfo bufferInfo;
		bufferInfo
			.setBuffer(this->animLevel.buffer)
			.setRange(bufferSize);

		vk::WriteDescriptorSet descriptorWrite;
		descriptorWrite
			.setDstSet(this->descriptorSets[i])
			.setDstBinding(3)
			.setDescriptorType(vk::DescriptorType::eUniformBuffer)
			.setBufferInfo(bufferInfo);

		this->device.updateDescriptorSets(descriptorWrite, {});
	}
}

void VulkanRenderer::initSectorHeights(rust::Slice<const float> heights) {
	if (heights.empty()) return;

	vk::DeviceSize bufferSize = heights.size() * sizeof(float);

	std::tie(this->sectorHeights.buffer, this->sectorHeights.memory) = createBuffer(
		bufferSize,
		vk::BufferUsageFlagBits::eStorageBuffer,
		vk::MemoryPropertyFlagBits::eHostVisible | vk::MemoryPropertyFlagBits::eHostCoherent
	);

	this->sectorHeights.mapped = this->sectorHeights.memory.mapMemory(0, bufferSize, {});
	memcpy(this->sectorHeights.mapped, heights.data(), bufferSize);

	for (size_t i = 0; i < MAX_FRAMES_IN_FLIGHT; ++i) {
		vk::DescriptorBufferInfo bufferInfo;
		bufferInfo
			.setBuffer(this->sectorHeights.buffer)
			.setRange(bufferSize);

		vk::WriteDescriptorSet descriptorWrite;
		descriptorWrite
			.setDstSet(this->descriptorSets[i])
			.setDstBinding(4)
			.setDescriptorType(vk::DescriptorType::eStorageBuffer)
			.setBufferInfo(bufferInfo);

		this->device.updateDescriptorSets(descriptorWrite, {});
	}
}

void VulkanRenderer::updateSectorHeights(rust::Slice<const float> heights) {
	vk::DeviceSize bufferSize = heights.size() * sizeof(float);

	memcpy(this->sectorHeights.mapped, heights.data(), bufferSize);
}

void VulkanRenderer::initSwitches(rust::Slice<const uint32_t> switches) {
	if (switches.empty()) return;

	vk::DeviceSize bufferSize = switches.size() * sizeof(uint32_t);

	std::tie(this->switches.buffer, this->switches.memory) = createBuffer(
		bufferSize,
		vk::BufferUsageFlagBits::eStorageBuffer,
		vk::MemoryPropertyFlagBits::eHostVisible | vk::MemoryPropertyFlagBits::eHostCoherent
	);

	this->switches.mapped = this->switches.memory.mapMemory(0, bufferSize, {});
	memcpy(this->switches.mapped, switches.data(), bufferSize);

	for (size_t i = 0; i < MAX_FRAMES_IN_FLIGHT; ++i) {
		vk::DescriptorBufferInfo bufferInfo;
		bufferInfo
			.setBuffer(this->switches.buffer)
			.setRange(bufferSize);

		vk::WriteDescriptorSet descriptorWrite;
		descriptorWrite
			.setDstSet(this->descriptorSets[i])
			.setDstBinding(5)
			.setDescriptorType(vk::DescriptorType::eStorageBuffer)
			.setBufferInfo(bufferInfo);

		this->device.updateDescriptorSets(descriptorWrite, {});
	}
}

void VulkanRenderer::updateSwitches(rust::Slice<const uint32_t> switches) {
	vk::DeviceSize bufferSize = switches.size() * sizeof(uint32_t);

	memcpy(this->switches.mapped, switches.data(), bufferSize);
}
