#include <cstring>
#include <stdexcept>
#include <array>
#include "renderer.h"
#include "renderer/src/bridge.rs.h"
#include "utils.h"

static const uint32_t levelVertShader[] = 
#include "level_vert.h"
;

static const uint32_t levelFragShader[] = 
#include "level_frag.h"
;

static const uint32_t objectVertShader[] = 
#include "object_vert.h"
;

static const uint32_t objectFragShader[] = 
#include "object_frag.h"
;

static const uint32_t uiVertShader[] = 
#include "ui_vert.h"
;

static const uint32_t uiFragShader[] = 
#include "ui_frag.h"
;

static const uint32_t weaponVertShader[] = 
#include "weapon_vert.h"
;

static const uint32_t weaponFragShader[] = 
#include "weapon_frag.h"
;

constexpr vk::Format findSupportedFormat(
	vk::PhysicalDevice physicalDevice, 
	const std::vector<vk::Format>& candidates, 
	vk::ImageTiling tiling, 
	vk::FormatFeatureFlags features
) {
	for (vk::Format format : candidates) {
		vk::FormatProperties props = physicalDevice.getFormatProperties(format);

		if (tiling == vk::ImageTiling::eLinear && (props.linearTilingFeatures & features) == features) {
			return format;
		} else if (tiling == vk::ImageTiling::eOptimal && (props.optimalTilingFeatures & features) == features) {
			return format;
		}
	}

	throw std::runtime_error("failed to find supported format!");
}

vk::Format VulkanRenderer::findDepthFormat() {
	return findSupportedFormat(
		this->physicalDevice,
		{vk::Format::eD32Sfloat, vk::Format::eD32SfloatS8Uint, vk::Format::eD24UnormS8Uint},
		vk::ImageTiling::eOptimal,
		vk::FormatFeatureFlagBits::eDepthStencilAttachment
	);
}

using BindDesc = vk::VertexInputBindingDescription;
using AttrDesc = vk::VertexInputAttributeDescription;

consteval std::array<BindDesc, 1> getLevelBindings() {
	return std::array{ BindDesc{ 0, sizeof(LevelVertex), vk::VertexInputRate::eVertex } };
}

consteval std::array<AttrDesc, 9> getLevelAttributes() {
	return std::array{
		AttrDesc{ 0, 0, vk::Format::eR32G32B32Sfloat, offsetof(LevelVertex, pos) },
		AttrDesc{ 1, 0, vk::Format::eR32G32Sfloat,    offsetof(LevelVertex, texture_pos) },
		AttrDesc{ 2, 0, vk::Format::eR32G32Uint,      offsetof(LevelVertex, light_level) },
		AttrDesc{ 3, 0, vk::Format::eR32G32Uint,      offsetof(LevelVertex, texture_id) },
		AttrDesc{ 4, 0, vk::Format::eR32G32Uint,      offsetof(LevelVertex, floor_tex_id) },
		AttrDesc{ 5, 0, vk::Format::eR32G32Sfloat,    offsetof(LevelVertex, scroll_dir) },
		AttrDesc{ 6, 0, vk::Format::eR32G32Uint,      offsetof(LevelVertex, plane_a) },
		AttrDesc{ 7, 0, vk::Format::eR32G32Uint,      offsetof(LevelVertex, plane_b) },
		AttrDesc{ 8, 0, vk::Format::eR32G32Sfloat,    offsetof(LevelVertex, inv_tex_h) },
	};
}

consteval std::array<BindDesc, 2> getObjectBindings() {
	return std::array{
		BindDesc{ 0, sizeof(SpriteVertex), vk::VertexInputRate::eVertex },
		BindDesc{ 1, sizeof(ObjectInstance), vk::VertexInputRate::eInstance }
	};
}

consteval std::array<AttrDesc, 7> getObjectAttributes() {
	return std::array{
		AttrDesc{ 0, 0, vk::Format::eR32G32B32Sfloat, offsetof(SpriteVertex, pos) },
		AttrDesc{ 1, 0, vk::Format::eR32G32Sfloat,    offsetof(SpriteVertex, texture_pos) },
		AttrDesc{ 2, 1, vk::Format::eR32G32B32Sfloat, offsetof(ObjectInstance, pos) },
		AttrDesc{ 3, 1, vk::Format::eR32G32Sfloat,    offsetof(ObjectInstance, sprite_offset) },
		AttrDesc{ 4, 1, vk::Format::eR32G32Sfloat,    offsetof(ObjectInstance, sprite_size) },
		AttrDesc{ 5, 1, vk::Format::eR32G32Uint,      offsetof(ObjectInstance, light_level) },
		AttrDesc{ 6, 1, vk::Format::eR32G32Uint,      offsetof(ObjectInstance, texture_id) },
	};
}

consteval std::array<BindDesc, 2> getUiBindings() {
	return std::array{
		BindDesc{ 0, sizeof(SpriteVertex), vk::VertexInputRate::eVertex },
		BindDesc{ 1, sizeof(UiInstance), vk::VertexInputRate::eInstance }
	};
}

consteval std::array<AttrDesc, 5> getUiAttributes() {
	return std::array{
		AttrDesc{ 0, 0, vk::Format::eR32G32B32Sfloat, offsetof(SpriteVertex, pos) },
		AttrDesc{ 1, 0, vk::Format::eR32G32Sfloat,    offsetof(SpriteVertex, texture_pos) },
		AttrDesc{ 2, 1, vk::Format::eR32G32Sfloat,    offsetof(UiInstance, pos) },
		AttrDesc{ 3, 1, vk::Format::eR32G32Sfloat,    offsetof(UiInstance, sprite_size) },
		AttrDesc{ 4, 1, vk::Format::eR32G32Uint,      offsetof(UiInstance, texture_id) }
	};
}

consteval std::array<BindDesc, 2> getWeaponBindings() {
	return std::array{
		BindDesc{ 0, sizeof(SpriteVertex), vk::VertexInputRate::eVertex },
		BindDesc{ 1, sizeof(UiInstance), vk::VertexInputRate::eInstance }
	};
}

consteval std::array<AttrDesc, 5> getWeaponAttributes() {
	return std::array{
		AttrDesc{ 0, 0, vk::Format::eR32G32B32Sfloat, offsetof(SpriteVertex, pos) },
		AttrDesc{ 1, 0, vk::Format::eR32G32Sfloat,    offsetof(SpriteVertex, texture_pos) },
		AttrDesc{ 2, 1, vk::Format::eR32G32Sfloat,    offsetof(UiInstance, pos) },
		AttrDesc{ 3, 1, vk::Format::eR32G32Sfloat,    offsetof(UiInstance, sprite_size) },
		AttrDesc{ 4, 1, vk::Format::eR32G32Uint,      offsetof(UiInstance, texture_id) }
	};
}

void VulkanRenderer::createDepthResources() {
	vk::Format depthFormat = findDepthFormat();

	std::tie(
		this->swapchainRes->depthImage, 
		this->swapchainRes->depthImageMemory
	) = createImage(
		this->swapchainRes->extent.width, 
		this->swapchainRes->extent.height, 
		depthFormat, 
		vk::ImageUsageFlagBits::eDepthStencilAttachment,
		vk::MemoryPropertyFlagBits::eDeviceLocal
	);

	this->swapchainRes->depthImageView = createImageView(
		this->device, 
		this->swapchainRes->depthImage, 
		depthFormat, 
		vk::ImageAspectFlagBits::eDepth
	);
}

void VulkanRenderer::updateLevelGeometry(
	rust::Slice<const LevelVertex> vertices, 
	rust::Slice<const uint32_t> indices
) {
	if (vertices.empty() || indices.empty()) return;

	this->levelMesh.vertexCount = static_cast<uint32_t>(vertices.size());
	this->levelMesh.indexCount = static_cast<uint32_t>(indices.size());

	vk::DeviceSize vertexBufferSize = sizeof(LevelVertex) * vertices.size();
	vk::DeviceSize indexBufferSize = sizeof(uint32_t) * indices.size();

	updateGeometry(
		vertices.data(), 
		indices, 
		this->levelMesh,
		vertexBufferSize, 
		indexBufferSize
	);
}

void VulkanRenderer::updateObjectGeometry(
	rust::Slice<const SpriteVertex> vertices, 
	rust::Slice<const uint32_t> indices
) {
	if (vertices.empty() || indices.empty()) return;

	this->objectMesh.vertexCount = static_cast<uint32_t>(vertices.size());
	this->objectMesh.indexCount = static_cast<uint32_t>(indices.size());

	vk::DeviceSize vertexBufferSize = sizeof(SpriteVertex) * vertices.size();
	vk::DeviceSize indexBufferSize = sizeof(uint32_t) * indices.size();

	updateGeometry(
		vertices.data(), 
		indices, 
		this->objectMesh, 
		vertexBufferSize, 
		indexBufferSize
	);
}

void VulkanRenderer::updateUiGeometry(
	rust::Slice<const SpriteVertex> vertices, 
	rust::Slice<const uint32_t> indices
) {
	if (vertices.empty() || indices.empty()) return;

	this->uiMesh.vertexCount = static_cast<uint32_t>(vertices.size());
	this->uiMesh.indexCount = static_cast<uint32_t>(indices.size());

	vk::DeviceSize vertexBufferSize = sizeof(SpriteVertex) * vertices.size();
	vk::DeviceSize indexBufferSize = sizeof(uint32_t) * indices.size();

	updateGeometry(
		vertices.data(), 
		indices, 
		this->uiMesh, 
		vertexBufferSize, 
		indexBufferSize
	);
}

void VulkanRenderer::updateWeaponGeometry(
	rust::Slice<const SpriteVertex> vertices, 
	rust::Slice<const uint32_t> indices
) {
	if (vertices.empty() || indices.empty()) return;

	this->weaponMesh.vertexCount = static_cast<uint32_t>(vertices.size());
	this->weaponMesh.indexCount = static_cast<uint32_t>(indices.size());

	vk::DeviceSize vertexBufferSize = sizeof(SpriteVertex) * vertices.size();
	vk::DeviceSize indexBufferSize = sizeof(uint32_t) * indices.size();

	updateGeometry(
		vertices.data(), 
		indices, 
		this->weaponMesh, 
		vertexBufferSize,  
		indexBufferSize
	);
}

void VulkanRenderer::updateGeometry(
	const void* vertices, 
	rust::Slice<const uint32_t> indices,
	MeshBuffers& mesh,
	vk::DeviceSize vertexBufferSize,
	vk::DeviceSize indexBufferSize
) {
	this->device.waitIdle();

	auto [vertexStagingBuffer, vertexStagingBufferMemory] = createBuffer(
		vertexBufferSize, 
		vk::BufferUsageFlagBits::eTransferSrc, 
		vk::MemoryPropertyFlagBits::eHostVisible | vk::MemoryPropertyFlagBits::eHostCoherent
	);

	void* vertexData = vertexStagingBufferMemory.mapMemory(0, vertexBufferSize, {});
	memcpy(vertexData, vertices, vertexBufferSize);
	vertexStagingBufferMemory.unmapMemory();

	std::tie(mesh.vertexBuffer, mesh.vertexBufferMemory) = createBuffer(
		vertexBufferSize, 
		vk::BufferUsageFlagBits::eTransferDst | vk::BufferUsageFlagBits::eVertexBuffer, 
		vk::MemoryPropertyFlagBits::eDeviceLocal
	);
	copyBuffer(vertexStagingBuffer, mesh.vertexBuffer, vertexBufferSize);
	
	auto [indexStagingBuffer, indexStagingBufferMemory] = createBuffer(
		indexBufferSize, 
		vk::BufferUsageFlagBits::eTransferSrc, 
		vk::MemoryPropertyFlagBits::eHostVisible | vk::MemoryPropertyFlagBits::eHostCoherent
	);
	
	void* indexData = indexStagingBufferMemory.mapMemory(0, indexBufferSize, {});
	memcpy(indexData, indices.data(), indexBufferSize);
	indexStagingBufferMemory.unmapMemory();

	std::tie(mesh.indexBuffer, mesh.indexBufferMemory) = createBuffer(
		indexBufferSize, 
		vk::BufferUsageFlagBits::eTransferDst | vk::BufferUsageFlagBits::eIndexBuffer, 
		vk::MemoryPropertyFlagBits::eDeviceLocal 
	);
	copyBuffer(indexStagingBuffer, mesh.indexBuffer, indexBufferSize);
}

using ShaderStg = vk::PipelineShaderStageCreateInfo;

void VulkanRenderer::createPipelines() {
	std::array<vk::DynamicState, 2> dynamicStates{
		vk::DynamicState::eViewport,
		vk::DynamicState::eScissor
	};

	vk::PipelineDynamicStateCreateInfo dynamicState{{}, dynamicStates};

	vk::PipelineInputAssemblyStateCreateInfo inputAssembly;
	inputAssembly.setTopology(vk::PrimitiveTopology::eTriangleList);

	vk::PipelineViewportStateCreateInfo viewportState;
	viewportState
		.setViewportCount(1)
		.setScissorCount(1);

	vk::PipelineRasterizationStateCreateInfo rasterizer;
	rasterizer
		.setPolygonMode(vk::PolygonMode::eFill)
		.setLineWidth(1.0f)
		.setCullMode(vk::CullModeFlagBits::eBack)  // eNone
		.setFrontFace(vk::FrontFace::eCounterClockwise);  // eClockwise

	vk::PipelineMultisampleStateCreateInfo multisampling;
	multisampling.setRasterizationSamples(vk::SampleCountFlagBits::e1);

	vk::PipelineColorBlendAttachmentState colorBlendAttachment{};

	vk::PipelineColorBlendStateCreateInfo colorBlending;
	colorBlending.setAttachments(colorBlendAttachment);

	vk::PipelineRenderingCreateInfo renderingInfo;
	renderingInfo
		.setColorAttachmentFormats(this->swapchainRes->imageFormat)
		.setDepthAttachmentFormat(findDepthFormat());

	vk::PipelineDepthStencilStateCreateInfo depthStencil{};
	vk::PipelineDepthStencilStateCreateInfo depthTestWriteStencil;
	depthTestWriteStencil
		.setDepthTestEnable(VK_TRUE)
		.setDepthWriteEnable(VK_TRUE)
		.setDepthCompareOp(vk::CompareOp::eLessOrEqual);
	
	vk::GraphicsPipelineCreateInfo pipelineInfoTemplate;
	pipelineInfoTemplate
		.setPNext(renderingInfo)
		.setPInputAssemblyState(&inputAssembly)
		.setPViewportState(&viewportState)
		.setPRasterizationState(&rasterizer)
		.setPMultisampleState(&multisampling)
		.setPColorBlendState(&colorBlending)
		.setPDynamicState(&dynamicState);

	std::array<vk::GraphicsPipelineCreateInfo, PIPELINES_COUNT> pipelineInfos;
	pipelineInfos.fill(pipelineInfoTemplate);

	vk::raii::ShaderModule levelVertShaderModule = this->device.createShaderModule({{}, levelVertShader});
	vk::raii::ShaderModule levelFragShaderModule = this->device.createShaderModule({{}, levelFragShader});
	vk::raii::ShaderModule objectVertShaderModule = this->device.createShaderModule({{}, objectVertShader});
	vk::raii::ShaderModule objectFragShaderModule = this->device.createShaderModule({{}, objectFragShader});
	vk::raii::ShaderModule uiVertShaderModule = this->device.createShaderModule({{}, uiVertShader});
	vk::raii::ShaderModule uiFragShaderModule = this->device.createShaderModule({{}, uiFragShader});
	vk::raii::ShaderModule weaponVertShaderModule = this->device.createShaderModule({{}, weaponVertShader});
	vk::raii::ShaderModule weaponFragShaderModule = this->device.createShaderModule({{}, weaponFragShader});

	std::array<ShaderStg, 2> levelStages{
		ShaderStg{{}, vk::ShaderStageFlagBits::eVertex, levelVertShaderModule, "main"}, 
		ShaderStg{{}, vk::ShaderStageFlagBits::eFragment, levelFragShaderModule, "main"}
	};

	std::array<ShaderStg, 2> objectStages{
		ShaderStg{{}, vk::ShaderStageFlagBits::eVertex, objectVertShaderModule, "main"}, 
		ShaderStg{{}, vk::ShaderStageFlagBits::eFragment, objectFragShaderModule, "main"}
	};

	std::array<ShaderStg, 2> uiStages{
		ShaderStg{{}, vk::ShaderStageFlagBits::eVertex, uiVertShaderModule, "main"}, 
		ShaderStg{{}, vk::ShaderStageFlagBits::eFragment, uiFragShaderModule, "main"}
	};

	std::array<ShaderStg, 2> weaponStages{
		ShaderStg{{}, vk::ShaderStageFlagBits::eVertex, weaponVertShaderModule, "main"}, 
		ShaderStg{{}, vk::ShaderStageFlagBits::eFragment, weaponFragShaderModule, "main"}
	};

	auto levelBindingDescriptions = getLevelBindings();
	auto levelAttributeDescriptions = getLevelAttributes();
	auto objectBindingDescriptions = getObjectBindings();
	auto objectAttributeDescriptions = getObjectAttributes();
	auto uiBindingDescriptions = getUiBindings();
	auto uiAttributeDescriptions = getUiAttributes();
	auto weaponBindingDescriptions = getWeaponBindings();
	auto weaponAttributeDescriptions = getWeaponAttributes();

	vk::PipelineVertexInputStateCreateInfo levelVertexInputInfo{{}, levelBindingDescriptions, levelAttributeDescriptions};
	vk::PipelineVertexInputStateCreateInfo objectVertexInputInfo{{}, objectBindingDescriptions, objectAttributeDescriptions};
	vk::PipelineVertexInputStateCreateInfo uiVertexInputInfo{{}, uiBindingDescriptions, uiAttributeDescriptions};
	vk::PipelineVertexInputStateCreateInfo weaponVertexInputInfo{{}, weaponBindingDescriptions, weaponAttributeDescriptions};

	vk::PushConstantRange levelPushConstantRange{
		SHADER_STAGE_FLAGS[PipelineType::Level],
		0,
		sizeof(LevelPushConstants)
	};
	vk::PushConstantRange objectPushConstantRange{
		SHADER_STAGE_FLAGS[PipelineType::Object], 
		0, 
		sizeof(ObjectPushConstants)
	};
	vk::PushConstantRange uiPushConstantRange{
		SHADER_STAGE_FLAGS[PipelineType::Ui],
		0,
		sizeof(uint32_t)
	};

	this->pipelineLayouts = {
		this->device.createPipelineLayout({
			{}, 
			*this->descriptorSetLayout, 
			levelPushConstantRange
		}),
		this->device.createPipelineLayout({
			{}, 
			*this->descriptorSetLayout, 
			objectPushConstantRange
		}),
		this->device.createPipelineLayout({
			{}, 
			*this->descriptorSetLayout, 
			uiPushConstantRange
		}),
		this->device.createPipelineLayout({
			{}, 
			*this->descriptorSetLayout, 
			uiPushConstantRange  // weapon uses the same constants as ui
		})
	};

	pipelineInfos[PipelineType::Level]
		.setLayout(this->pipelineLayouts[PipelineType::Level])
		.setPVertexInputState(&levelVertexInputInfo)
		.setPDepthStencilState(&depthTestWriteStencil)
		.setStages(levelStages);
	pipelineInfos[PipelineType::Object]
		.setLayout(this->pipelineLayouts[PipelineType::Object])
		.setPVertexInputState(&objectVertexInputInfo)
		.setPDepthStencilState(&depthTestWriteStencil)
		.setStages(objectStages);
	pipelineInfos[PipelineType::Ui]
		.setLayout(this->pipelineLayouts[PipelineType::Ui])
		.setPVertexInputState(&uiVertexInputInfo)
		.setPDepthStencilState(&depthStencil)
		.setStages(uiStages);
	pipelineInfos[PipelineType::Weapon]
		.setLayout(this->pipelineLayouts[PipelineType::Weapon])
		.setPVertexInputState(&weaponVertexInputInfo)
		.setPDepthStencilState(&depthStencil)
		.setStages(weaponStages);

	auto pipelines = this->device.createGraphicsPipelines(VK_NULL_HANDLE, pipelineInfos);
	for (size_t i = 0; i < PIPELINES_COUNT; ++i) {
		this->pipelines[i] = std::move(pipelines[i]);
	}
}
