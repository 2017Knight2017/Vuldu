#include "renderer.h"
#include "renderer/src/bridge.rs.h"

using Bind = vk::DescriptorBindingFlagBits;

void VulkanRenderer::createDescriptorSetLayout() {
	vk::DescriptorSetLayoutBinding mvpLayoutBinding{
		0,
		vk::DescriptorType::eUniformBuffer,
		1,
		vk::ShaderStageFlagBits::eVertex
	};

	vk::DescriptorSetLayoutBinding paletteLayoutBinding{
		1,
		vk::DescriptorType::eCombinedImageSampler,
		1,
		vk::ShaderStageFlagBits::eFragment
	};

	vk::DescriptorSetLayoutBinding colormapLayoutBinding{
		2,
		vk::DescriptorType::eCombinedImageSampler,
		1,
		vk::ShaderStageFlagBits::eFragment
	};

	vk::DescriptorSetLayoutBinding animLevelLayoutBinding{
		3,
		vk::DescriptorType::eUniformBuffer,
		1,
		vk::ShaderStageFlagBits::eVertex
	};
	
	vk::DescriptorSetLayoutBinding sectorHeightsLayoutBinding{
		4,
		vk::DescriptorType::eStorageBuffer,
		1,
		vk::ShaderStageFlagBits::eVertex
	};

	vk::DescriptorSetLayoutBinding switchesLayoutBinding{
		5,
		vk::DescriptorType::eStorageBuffer,
		1,
		vk::ShaderStageFlagBits::eVertex
	};

	vk::DescriptorSetLayoutBinding textureSamplerLayoutBinding{
		6,
		vk::DescriptorType::eCombinedImageSampler,
		MAX_TEXTURES,
		vk::ShaderStageFlagBits::eFragment
	};

	std::array<vk::DescriptorSetLayoutBinding, DESCRIPTORS_COUNT> bindings{
		mvpLayoutBinding,
		paletteLayoutBinding, 
		colormapLayoutBinding,
		animLevelLayoutBinding,
		sectorHeightsLayoutBinding,
		switchesLayoutBinding,
		textureSamplerLayoutBinding,
	};

	std::array<vk::DescriptorBindingFlags, DESCRIPTORS_COUNT> bindingFlags{};
	bindingFlags.back() = Bind::ePartiallyBound | Bind::eUpdateAfterBind | Bind::eVariableDescriptorCount;

	vk::DescriptorSetLayoutBindingFlagsCreateInfo bindingFlagsInfo{bindingFlags}; 

	vk::DescriptorSetLayoutCreateInfo layoutInfo{
		vk::DescriptorSetLayoutCreateFlagBits::eUpdateAfterBindPool,
		bindings,
		bindingFlagsInfo
	};
	
	this->descriptorSetLayout = this->device.createDescriptorSetLayout(layoutInfo);
}

void VulkanRenderer::createMVPBuffer() {
	vk::DeviceSize bufferSize = sizeof(MVP);

	for (size_t i = 0; i < MAX_FRAMES_IN_FLIGHT; ++i) {
		auto [buffer, memory] = createBuffer(
			bufferSize, 
			vk::BufferUsageFlagBits::eUniformBuffer, 
			vk::MemoryPropertyFlagBits::eHostVisible | vk::MemoryPropertyFlagBits::eHostCoherent 
		);

		void* mapped = memory.mapMemory(0, bufferSize, {});

		this->MVPBuffers[i] = std::move(buffer);
		this->MVPBuffersMemory[i] = std::move(memory); 
		this->MVPBuffersMapped[i] = mapped;
	}
}

void VulkanRenderer::createInstanceBuffersInternal(
	vk::DeviceSize bufferSize, 
	std::array<vk::raii::Buffer, MAX_FRAMES_IN_FLIGHT>& pBuffers,
	std::array<vk::raii::DeviceMemory, MAX_FRAMES_IN_FLIGHT>& pBuffersMemory,
	std::array<void*, MAX_FRAMES_IN_FLIGHT>& pBuffersMapped
) {
	for (size_t i = 0; i < MAX_FRAMES_IN_FLIGHT; ++i) {
		auto [buffer, memory] = createBuffer(
			bufferSize, 
			vk::BufferUsageFlagBits::eVertexBuffer, 
			vk::MemoryPropertyFlagBits::eHostVisible | vk::MemoryPropertyFlagBits::eHostCoherent
		);

		void* mapped = memory.mapMemory(0, bufferSize, {});

		pBuffers[i] = std::move(buffer);
		pBuffersMemory[i] = std::move(memory);
		pBuffersMapped[i] = mapped;
	}
}

void VulkanRenderer::createInstanceBuffers() {
	createInstanceBuffersInternal(
		sizeof(ObjectInstance) * MAX_OBJECTS, 
		this->objectInst.buffers, 
		this->objectInst.memories, 
		this->objectInst.mapped
	);
	createInstanceBuffersInternal(
		sizeof(UiInstance) * MAX_UI, 
		this->uiInst.buffers, 
		this->uiInst.memories, 
		this->uiInst.mapped
	);
	createInstanceBuffersInternal(
		sizeof(UiInstance), 
		this->weaponInst.buffers, 
		this->weaponInst.memories, 
		this->weaponInst.mapped
	);
}

using PoolSize = vk::DescriptorPoolSize;

void VulkanRenderer::createDescriptorPool() {
	std::array<vk::DescriptorPoolSize, DESCRIPTORS_COUNT> poolSizes{
		PoolSize{vk::DescriptorType::eUniformBuffer, MAX_FRAMES_IN_FLIGHT},
		PoolSize{vk::DescriptorType::eCombinedImageSampler, MAX_FRAMES_IN_FLIGHT},
		PoolSize{vk::DescriptorType::eCombinedImageSampler, MAX_FRAMES_IN_FLIGHT},
		PoolSize{vk::DescriptorType::eUniformBuffer, MAX_FRAMES_IN_FLIGHT},
		PoolSize{vk::DescriptorType::eStorageBuffer, MAX_FRAMES_IN_FLIGHT},
		PoolSize{vk::DescriptorType::eStorageBuffer, MAX_FRAMES_IN_FLIGHT},
		PoolSize{vk::DescriptorType::eCombinedImageSampler, MAX_FRAMES_IN_FLIGHT * MAX_TEXTURES},
	};
	
	vk::DescriptorPoolCreateInfo poolInfo{
		vk::DescriptorPoolCreateFlagBits::eUpdateAfterBind | vk::DescriptorPoolCreateFlagBits::eFreeDescriptorSet,
		MAX_FRAMES_IN_FLIGHT,
		poolSizes
	};
	
	this->descriptorPool = this->device.createDescriptorPool(poolInfo);
}

void VulkanRenderer::createDescriptorSets() {
	std::array<vk::DescriptorSetLayout, MAX_FRAMES_IN_FLIGHT> layouts{this->descriptorSetLayout, this->descriptorSetLayout};
	std::array<uint32_t, MAX_FRAMES_IN_FLIGHT> variableCounts{MAX_TEXTURES, MAX_TEXTURES};

	vk::DescriptorSetVariableDescriptorCountAllocateInfo variableCountsInfo{variableCounts};

	vk::DescriptorSetAllocateInfo allocInfo{
		this->descriptorPool, 
		layouts,
		variableCountsInfo
	};
	
	auto descriptorSets = this->device.allocateDescriptorSets(allocInfo);

	for (size_t i = 0; i < MAX_FRAMES_IN_FLIGHT; ++i) {
		this->descriptorSets[i] = std::move(descriptorSets[i]);

		vk::DescriptorBufferInfo bufferInfo;
		bufferInfo
			.setBuffer(this->MVPBuffers[i])
			.setRange(sizeof(MVP));

		vk::WriteDescriptorSet descriptorWrite;
		descriptorWrite
			.setDstSet(this->descriptorSets[i])
			.setDstBinding(0)
			.setDescriptorType(vk::DescriptorType::eUniformBuffer)
			.setBufferInfo(bufferInfo);

		this->device.updateDescriptorSets(descriptorWrite, {});
	}
}
