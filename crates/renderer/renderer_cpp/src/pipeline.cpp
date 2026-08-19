#include <cstring>
#include <stdexcept>
#include "renderer.h"
#include "renderer/src/bridge.rs.h"
#include "utils.h"

static const uint32_t levelVertShader[] = 
#include "level_vert.h"
;

static const uint32_t levelFragShader[] = 
#include "level_frag.h"
;

static const uint32_t spriteVertShader[] = 
#include "sprite_vert.h"
;

static const uint32_t spriteFragShader[] = 
#include "sprite_frag.h"
;

static const uint32_t uiVertShader[] = 
#include "ui_vert.h"
;

static const uint32_t uiFragShader[] = 
#include "ui_frag.h"
;

VkFormat findSupportedFormat(VkPhysicalDevice physicalDevice, const std::vector<VkFormat>& candidates, VkImageTiling tiling, VkFormatFeatureFlags features) {
    for (VkFormat format : candidates) {
        VkFormatProperties props;
        vkGetPhysicalDeviceFormatProperties(physicalDevice, format, &props);

        if (tiling == VK_IMAGE_TILING_LINEAR && (props.linearTilingFeatures & features) == features) {
            return format;
        } else if (tiling == VK_IMAGE_TILING_OPTIMAL && (props.optimalTilingFeatures & features) == features) {
            return format;
        }
    }

    throw std::runtime_error("failed to find supported format!");
}

VkFormat VulkanRenderer::findDepthFormat() {
    return findSupportedFormat(
		this->physicalDevice,
        {VK_FORMAT_D32_SFLOAT, VK_FORMAT_D32_SFLOAT_S8_UINT, VK_FORMAT_D24_UNORM_S8_UINT},
        VK_IMAGE_TILING_OPTIMAL,
        VK_FORMAT_FEATURE_DEPTH_STENCIL_ATTACHMENT_BIT
    );
}

std::vector<VkVertexInputBindingDescription> getLevelBindings() {
    return { { 0, sizeof(Vertex), VK_VERTEX_INPUT_RATE_VERTEX } };
}

std::vector<VkVertexInputAttributeDescription> getLevelAttributes() {
    return {
        { 0, 0, VK_FORMAT_R32G32B32_SFLOAT, offsetof(Vertex, pos) },
        { 1, 0, VK_FORMAT_R32G32_SFLOAT,    offsetof(Vertex, texture_pos) },
        { 2, 0, VK_FORMAT_R32_SFLOAT,       offsetof(Vertex, light_level) },
        { 3, 0, VK_FORMAT_R32_UINT,         offsetof(Vertex, texture_id) },
        { 4, 0, VK_FORMAT_R32_UINT,         offsetof(Vertex, colormap_idx) },
		{ 5, 0, VK_FORMAT_R32_UINT,         offsetof(Vertex, floor_tex_id) },
		{ 6, 0, VK_FORMAT_R32_SFLOAT,       offsetof(Vertex, scroll_dir) }
    };
}

std::vector<VkVertexInputBindingDescription> getSpriteBindings() {
    return {
        { 0, sizeof(Vertex), VK_VERTEX_INPUT_RATE_VERTEX },
        { 1, sizeof(ObjectInstance), VK_VERTEX_INPUT_RATE_INSTANCE }
    };
}

std::vector<VkVertexInputAttributeDescription> getSpriteAttributes() {
    return {
        { 0, 0, VK_FORMAT_R32G32B32_SFLOAT, offsetof(Vertex, pos) },
        { 1, 0, VK_FORMAT_R32G32_SFLOAT,    offsetof(Vertex, texture_pos) },
        { 2, 1, VK_FORMAT_R32G32B32_SFLOAT, offsetof(ObjectInstance, pos) },
        { 3, 1, VK_FORMAT_R32G32_SFLOAT,    offsetof(ObjectInstance, sprite_offset) },
		{ 4, 1, VK_FORMAT_R32G32_SFLOAT,    offsetof(ObjectInstance, sprite_size) },
        { 5, 1, VK_FORMAT_R32_SFLOAT,       offsetof(ObjectInstance, light_level) },
        { 6, 1, VK_FORMAT_R32_UINT,         offsetof(ObjectInstance, texture_id) },
        { 7, 1, VK_FORMAT_R32_UINT,         offsetof(ObjectInstance, colormap_idx) }
    };
}

std::vector<VkVertexInputBindingDescription> getUiBindings() {
    return {
        { 0, sizeof(Vertex), VK_VERTEX_INPUT_RATE_VERTEX },
        { 1, sizeof(UiInstance), VK_VERTEX_INPUT_RATE_INSTANCE }
    };
}

std::vector<VkVertexInputAttributeDescription> getUiAttributes() {
    return {
        { 0, 0, VK_FORMAT_R32G32B32_SFLOAT, offsetof(Vertex, pos) },
        { 1, 0, VK_FORMAT_R32G32_SFLOAT,    offsetof(Vertex, texture_pos) },
		{ 2, 1, VK_FORMAT_R32G32_SFLOAT,    offsetof(UiInstance, pos) },
		{ 3, 1, VK_FORMAT_R32G32_SFLOAT,    offsetof(UiInstance, sprite_size) },
        { 4, 1, VK_FORMAT_R32_UINT,         offsetof(UiInstance, texture_id) }
    };
}

void VulkanRenderer::createDepthResources() {
	VkFormat depthFormat = findDepthFormat();

	createImage(
		this->swapChainExtent.width, 
		this->swapChainExtent.height, 
		depthFormat, 
		VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT, 
		VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT, 
		this->depthImage, 
		this->depthImageMemory
	);
	createImageView(
		this->device, 
		this->depthImage, 
		depthFormat, 
		VK_IMAGE_ASPECT_DEPTH_BIT, 
		&this->depthImageView
	);
}

void VulkanRenderer::updateLevelGeometry(const Vertex* vertices_ptr, size_t vertex_count, const uint32_t* indices_ptr, size_t index_count) {
    if (vertex_count == 0 || vertices_ptr == nullptr || index_count == 0 || indices_ptr == nullptr) return;

    vkDeviceWaitIdle(this->device);

    destroyResource(this->device, this->levelVertexBuffer, vkDestroyBuffer);
    destroyResource(this->device, this->levelVertexBufferMemory, vkFreeMemory);
    destroyResource(this->device, this->levelIndexBuffer, vkDestroyBuffer);
    destroyResource(this->device, this->levelIndexBufferMemory, vkFreeMemory);

    this->levelVertexCount = static_cast<uint32_t>(vertex_count);
    this->levelIndexCount = static_cast<uint32_t>(index_count);

    VkDeviceSize vertexBufferSize = sizeof(Vertex) * vertex_count;
    VkDeviceSize indexBufferSize = sizeof(uint32_t) * index_count;

    VkBuffer vertexStagingBuffer;
    VkDeviceMemory vertexStagingBufferMemory;
    createBuffer(vertexBufferSize, VK_BUFFER_USAGE_TRANSFER_SRC_BIT, VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT, vertexStagingBuffer, vertexStagingBufferMemory);

    void* vertexData;
    if (vkMapMemory(this->device, vertexStagingBufferMemory, 0, vertexBufferSize, 0, &vertexData) != VK_SUCCESS) {
        throw std::runtime_error("failed to map vertex staging buffer memory!");
    }
    memcpy(vertexData, vertices_ptr, static_cast<size_t>(vertexBufferSize));
    vkUnmapMemory(this->device, vertexStagingBufferMemory);

    createBuffer(vertexBufferSize, VK_BUFFER_USAGE_TRANSFER_DST_BIT | VK_BUFFER_USAGE_VERTEX_BUFFER_BIT, VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT, this->levelVertexBuffer, this->levelVertexBufferMemory);
    copyBuffer(vertexStagingBuffer, this->levelVertexBuffer, vertexBufferSize);

    vkDestroyBuffer(this->device, vertexStagingBuffer, nullptr);
    vkFreeMemory(this->device, vertexStagingBufferMemory, nullptr);

    
    VkBuffer indexStagingBuffer;
    VkDeviceMemory indexStagingBufferMemory;
    createBuffer(indexBufferSize, VK_BUFFER_USAGE_TRANSFER_SRC_BIT, VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT, indexStagingBuffer, indexStagingBufferMemory);
	
    void* indexData;
    if (vkMapMemory(this->device, indexStagingBufferMemory, 0, indexBufferSize, 0, &indexData) != VK_SUCCESS) {
        throw std::runtime_error("failed to map index staging buffer memory!");
    }
    memcpy(indexData, indices_ptr, static_cast<size_t>(indexBufferSize));
    vkUnmapMemory(this->device, indexStagingBufferMemory);

    createBuffer(indexBufferSize, VK_BUFFER_USAGE_TRANSFER_DST_BIT | VK_BUFFER_USAGE_INDEX_BUFFER_BIT, VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT, this->levelIndexBuffer, this->levelIndexBufferMemory);
    copyBuffer(indexStagingBuffer, this->levelIndexBuffer, indexBufferSize);

    vkDestroyBuffer(this->device, indexStagingBuffer, nullptr);
    vkFreeMemory(this->device, indexStagingBufferMemory, nullptr);
}

void VulkanRenderer::updateObjectGeometry(const Vertex* vertices_ptr, size_t vertex_count, const uint32_t* indices_ptr, size_t index_count) {
    if (vertex_count == 0 || vertices_ptr == nullptr || index_count == 0 || indices_ptr == nullptr) return;

    vkDeviceWaitIdle(this->device);

    destroyResource(this->device, this->objectVertexBuffer, vkDestroyBuffer);
    destroyResource(this->device, this->objectVertexBufferMemory, vkFreeMemory);
    destroyResource(this->device, this->objectIndexBuffer, vkDestroyBuffer);
    destroyResource(this->device, this->objectIndexBufferMemory, vkFreeMemory);

    this->objectVertexCount = static_cast<uint32_t>(vertex_count);
    this->objectIndexCount = static_cast<uint32_t>(index_count);

    VkDeviceSize vertexBufferSize = sizeof(Vertex) * vertex_count;
    VkDeviceSize indexBufferSize = sizeof(uint32_t) * index_count;

    VkBuffer vertexStagingBuffer;
    VkDeviceMemory vertexStagingBufferMemory;
    createBuffer(vertexBufferSize, VK_BUFFER_USAGE_TRANSFER_SRC_BIT, VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT, vertexStagingBuffer, vertexStagingBufferMemory);

    void* vertexData;
    vkMapMemory(this->device, vertexStagingBufferMemory, 0, vertexBufferSize, 0, &vertexData);
    memcpy(vertexData, vertices_ptr, static_cast<size_t>(vertexBufferSize));
    vkUnmapMemory(this->device, vertexStagingBufferMemory);

    createBuffer(vertexBufferSize, VK_BUFFER_USAGE_TRANSFER_DST_BIT | VK_BUFFER_USAGE_VERTEX_BUFFER_BIT, VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT, this->objectVertexBuffer, this->objectVertexBufferMemory);
    copyBuffer(vertexStagingBuffer, this->objectVertexBuffer, vertexBufferSize);

    vkDestroyBuffer(this->device, vertexStagingBuffer, nullptr);
    vkFreeMemory(this->device, vertexStagingBufferMemory, nullptr);

    
    VkBuffer indexStagingBuffer;
    VkDeviceMemory indexStagingBufferMemory;
    createBuffer(indexBufferSize, VK_BUFFER_USAGE_TRANSFER_SRC_BIT, VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT, indexStagingBuffer, indexStagingBufferMemory);
	
    void* indexData;
   	vkMapMemory(this->device, indexStagingBufferMemory, 0, indexBufferSize, 0, &indexData);
    memcpy(indexData, indices_ptr, static_cast<size_t>(indexBufferSize));
    vkUnmapMemory(this->device, indexStagingBufferMemory);

    createBuffer(indexBufferSize, VK_BUFFER_USAGE_TRANSFER_DST_BIT | VK_BUFFER_USAGE_INDEX_BUFFER_BIT, VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT, this->objectIndexBuffer, this->objectIndexBufferMemory);
    copyBuffer(indexStagingBuffer, this->objectIndexBuffer, indexBufferSize);

    vkDestroyBuffer(this->device, indexStagingBuffer, nullptr);
    vkFreeMemory(this->device, indexStagingBufferMemory, nullptr);
}

void VulkanRenderer::updateUiGeometry(const Vertex* vertices_ptr, size_t vertex_count, const uint32_t* indices_ptr, size_t index_count) {
    if (vertex_count == 0 || vertices_ptr == nullptr || index_count == 0 || indices_ptr == nullptr) return;

    vkDeviceWaitIdle(this->device);

    destroyResource(this->device, this->uiVertexBuffer, vkDestroyBuffer);
    destroyResource(this->device, this->uiVertexBufferMemory, vkFreeMemory);
    destroyResource(this->device, this->uiIndexBuffer, vkDestroyBuffer);
    destroyResource(this->device, this->uiIndexBufferMemory, vkFreeMemory);

    this->uiVertexCount = static_cast<uint32_t>(vertex_count);
    this->uiIndexCount = static_cast<uint32_t>(index_count);

    VkDeviceSize vertexBufferSize = sizeof(Vertex) * vertex_count;
    VkDeviceSize indexBufferSize = sizeof(uint32_t) * index_count;

    VkBuffer vertexStagingBuffer;
    VkDeviceMemory vertexStagingBufferMemory;
    createBuffer(vertexBufferSize, VK_BUFFER_USAGE_TRANSFER_SRC_BIT, VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT, vertexStagingBuffer, vertexStagingBufferMemory);

    void* vertexData;
    vkMapMemory(this->device, vertexStagingBufferMemory, 0, vertexBufferSize, 0, &vertexData);
    memcpy(vertexData, vertices_ptr, static_cast<size_t>(vertexBufferSize));
    vkUnmapMemory(this->device, vertexStagingBufferMemory);

    createBuffer(vertexBufferSize, VK_BUFFER_USAGE_TRANSFER_DST_BIT | VK_BUFFER_USAGE_VERTEX_BUFFER_BIT, VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT, this->uiVertexBuffer, this->uiVertexBufferMemory);
    copyBuffer(vertexStagingBuffer, this->uiVertexBuffer, vertexBufferSize);

    vkDestroyBuffer(this->device, vertexStagingBuffer, nullptr);
    vkFreeMemory(this->device, vertexStagingBufferMemory, nullptr);

    
    VkBuffer indexStagingBuffer;
    VkDeviceMemory indexStagingBufferMemory;
    createBuffer(indexBufferSize, VK_BUFFER_USAGE_TRANSFER_SRC_BIT, VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT, indexStagingBuffer, indexStagingBufferMemory);
	
    void* indexData;
   	vkMapMemory(this->device, indexStagingBufferMemory, 0, indexBufferSize, 0, &indexData);
    memcpy(indexData, indices_ptr, static_cast<size_t>(indexBufferSize));
    vkUnmapMemory(this->device, indexStagingBufferMemory);

    createBuffer(indexBufferSize, VK_BUFFER_USAGE_TRANSFER_DST_BIT | VK_BUFFER_USAGE_INDEX_BUFFER_BIT, VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT, this->uiIndexBuffer, this->uiIndexBufferMemory);
    copyBuffer(indexStagingBuffer, this->uiIndexBuffer, indexBufferSize);

    vkDestroyBuffer(this->device, indexStagingBuffer, nullptr);
    vkFreeMemory(this->device, indexStagingBufferMemory, nullptr);
}

void VulkanRenderer::createPipelines() {
	std::vector<VkDynamicState> dynamicStates = {
	    VK_DYNAMIC_STATE_VIEWPORT,
	    VK_DYNAMIC_STATE_SCISSOR
	};

	VkPipelineDynamicStateCreateInfo dynamicState{};
	dynamicState.sType = VK_STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO;
	dynamicState.dynamicStateCount = static_cast<uint32_t>(dynamicStates.size());
	dynamicState.pDynamicStates = dynamicStates.data();

	VkPipelineInputAssemblyStateCreateInfo inputAssembly{};
	inputAssembly.sType = VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO;
	inputAssembly.topology = VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST;
	inputAssembly.primitiveRestartEnable = VK_FALSE;

	VkPipelineViewportStateCreateInfo viewportState{};
	viewportState.sType = VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO;
	viewportState.viewportCount = 1;
	viewportState.scissorCount = 1;

	VkPipelineRasterizationStateCreateInfo rasterizer{};
	rasterizer.sType = VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO;
	rasterizer.depthClampEnable = VK_FALSE;
	rasterizer.rasterizerDiscardEnable = VK_FALSE;
	rasterizer.polygonMode = VK_POLYGON_MODE_FILL;
	rasterizer.lineWidth = 1.0f;
	rasterizer.cullMode = VK_CULL_MODE_BACK_BIT;
	//rasterizer.cullMode = VK_CULL_MODE_NONE;
	//rasterizer.frontFace = VK_FRONT_FACE_CLOCKWISE;
	rasterizer.frontFace = VK_FRONT_FACE_COUNTER_CLOCKWISE;
	rasterizer.depthBiasEnable = VK_FALSE;

	VkPipelineMultisampleStateCreateInfo multisampling{};
	multisampling.sType = VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO;
	multisampling.sampleShadingEnable = VK_FALSE;
	multisampling.rasterizationSamples = VK_SAMPLE_COUNT_1_BIT;

	VkPipelineDepthStencilStateCreateInfo depthStencil{};
	depthStencil.sType = VK_STRUCTURE_TYPE_PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO;
	depthStencil.depthTestEnable = VK_TRUE;
	depthStencil.depthWriteEnable = VK_TRUE;
	depthStencil.depthCompareOp = VK_COMPARE_OP_LESS_OR_EQUAL;
	depthStencil.depthBoundsTestEnable = VK_FALSE;
	depthStencil.stencilTestEnable = VK_FALSE;

	VkPipelineColorBlendAttachmentState colorBlendAttachment{};
	colorBlendAttachment.colorWriteMask = VK_COLOR_COMPONENT_R_BIT | VK_COLOR_COMPONENT_G_BIT | VK_COLOR_COMPONENT_B_BIT | VK_COLOR_COMPONENT_A_BIT;
	colorBlendAttachment.blendEnable = VK_FALSE;

	VkPipelineColorBlendStateCreateInfo colorBlending{};
	colorBlending.sType = VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO;
	colorBlending.logicOpEnable = VK_FALSE;
	colorBlending.attachmentCount = 1;
	colorBlending.pAttachments = &colorBlendAttachment;

	VkPipelineRenderingCreateInfo renderingInfo{};
	renderingInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_RENDERING_CREATE_INFO_KHR;
	renderingInfo.colorAttachmentCount = 1;
	renderingInfo.pColorAttachmentFormats = &this->swapChainImageFormat; 
	renderingInfo.depthAttachmentFormat = findDepthFormat();
	renderingInfo.stencilAttachmentFormat = VK_FORMAT_UNDEFINED;	

	createSpritePipeline(
		&inputAssembly,
		&viewportState,
		&rasterizer,
		&multisampling,
		&depthStencil,
		&colorBlending,
		&dynamicState,
		&renderingInfo
	);
	createLevelPipeline(
		&inputAssembly,
		&viewportState,
		&rasterizer,
		&multisampling,
		&depthStencil,
		&colorBlending,
		&dynamicState,
		&renderingInfo
	);
	createUiPipeline(
		&inputAssembly,
		&viewportState,
		&rasterizer,
		&multisampling,
		&colorBlending,
		&dynamicState,
		&renderingInfo
	);
}

void VulkanRenderer::createSpritePipeline(
	VkPipelineInputAssemblyStateCreateInfo* inputAssembly,
	VkPipelineViewportStateCreateInfo* viewportState,
	VkPipelineRasterizationStateCreateInfo* rasterizer,
	VkPipelineMultisampleStateCreateInfo* multisampling,
	VkPipelineDepthStencilStateCreateInfo* depthStencil,
	VkPipelineColorBlendStateCreateInfo* colorBlending,
	VkPipelineDynamicStateCreateInfo* dynamicState,
	VkPipelineRenderingCreateInfo* renderingInfo
) {
	VkShaderModule vertShaderModule = createShaderModule(this->device, spriteVertShader);
    VkShaderModule fragShaderModule = createShaderModule(this->device, spriteFragShader);

	VkPipelineShaderStageCreateInfo vertShaderInfo{};
	vertShaderInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO;
	vertShaderInfo.stage = VK_SHADER_STAGE_VERTEX_BIT;
	vertShaderInfo.module = vertShaderModule;
	vertShaderInfo.pName = "main";

	VkPipelineShaderStageCreateInfo fragShaderInfo{};
	fragShaderInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO;
	fragShaderInfo.stage = VK_SHADER_STAGE_FRAGMENT_BIT;
	fragShaderInfo.module = fragShaderModule;
	fragShaderInfo.pName = "main";

	VkPipelineShaderStageCreateInfo shaderStages[] = {vertShaderInfo, fragShaderInfo};

	auto bindingDescriptions = getSpriteBindings();
	auto attributeDescriptions = getSpriteAttributes();

	VkPipelineVertexInputStateCreateInfo vertexInputInfo{};
	vertexInputInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO;
	vertexInputInfo.vertexBindingDescriptionCount = static_cast<uint32_t>(bindingDescriptions.size());
	vertexInputInfo.vertexAttributeDescriptionCount = static_cast<uint32_t>(attributeDescriptions.size());
	vertexInputInfo.pVertexBindingDescriptions = bindingDescriptions.data();
	vertexInputInfo.pVertexAttributeDescriptions = attributeDescriptions.data();

	VkPushConstantRange pushConstantRange{};
	pushConstantRange.stageFlags = VK_SHADER_STAGE_FRAGMENT_BIT; 
	pushConstantRange.offset = 0;
	pushConstantRange.size = sizeof(uint32_t);

	VkPipelineLayoutCreateInfo pipelineLayoutInfo{};
	pipelineLayoutInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO;
	pipelineLayoutInfo.setLayoutCount = 1;
	pipelineLayoutInfo.pSetLayouts = &this->descriptorSetLayout;
	pipelineLayoutInfo.pushConstantRangeCount = 1;
	pipelineLayoutInfo.pPushConstantRanges = &pushConstantRange;

	VkResult pipelineLayoutResult = vkCreatePipelineLayout(this->device, &pipelineLayoutInfo, nullptr, &this->spritePipelineLayout);
	if (pipelineLayoutResult != VK_SUCCESS) {
	    throw std::runtime_error("failed to create pipeline layout!");
	}

	VkGraphicsPipelineCreateInfo pipelineInfo{};
	pipelineInfo.sType = VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO;
	pipelineInfo.pNext = renderingInfo;
	pipelineInfo.stageCount = 2;
	pipelineInfo.pStages = shaderStages;
	pipelineInfo.pVertexInputState = &vertexInputInfo;
	pipelineInfo.pInputAssemblyState = inputAssembly;
	pipelineInfo.pViewportState = viewportState;
	pipelineInfo.pRasterizationState = rasterizer;
	pipelineInfo.pMultisampleState = multisampling;
	pipelineInfo.pDepthStencilState = depthStencil;
	pipelineInfo.pColorBlendState = colorBlending;
	pipelineInfo.pDynamicState = dynamicState;
	pipelineInfo.layout = this->spritePipelineLayout;
	pipelineInfo.subpass = 0;

	VkResult PipelineResult = vkCreateGraphicsPipelines(this->device, VK_NULL_HANDLE, 1, &pipelineInfo, nullptr, &this->spritePipeline);
	if (PipelineResult != VK_SUCCESS) {
    	throw std::runtime_error("failed to create graphics pipeline!");
	}
	
	vkDestroyShaderModule(this->device, fragShaderModule, nullptr);
    vkDestroyShaderModule(this->device, vertShaderModule, nullptr);
}

void VulkanRenderer::createLevelPipeline(
	VkPipelineInputAssemblyStateCreateInfo* inputAssembly,
	VkPipelineViewportStateCreateInfo* viewportState,
	VkPipelineRasterizationStateCreateInfo* rasterizer,
	VkPipelineMultisampleStateCreateInfo* multisampling,
	VkPipelineDepthStencilStateCreateInfo* depthStencil,
	VkPipelineColorBlendStateCreateInfo* colorBlending,
	VkPipelineDynamicStateCreateInfo* dynamicState,
	VkPipelineRenderingCreateInfo* renderingInfo
) {
	VkShaderModule vertShaderModule = createShaderModule(this->device, levelVertShader);
    VkShaderModule fragShaderModule = createShaderModule(this->device, levelFragShader);

	VkPipelineShaderStageCreateInfo vertShaderInfo{};
	vertShaderInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO;
	vertShaderInfo.stage = VK_SHADER_STAGE_VERTEX_BIT;
	vertShaderInfo.module = vertShaderModule;
	vertShaderInfo.pName = "main";

	VkPipelineShaderStageCreateInfo fragShaderInfo{};
	fragShaderInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO;
	fragShaderInfo.stage = VK_SHADER_STAGE_FRAGMENT_BIT;
	fragShaderInfo.module = fragShaderModule;
	fragShaderInfo.pName = "main";

	VkPipelineShaderStageCreateInfo shaderStages[] = {vertShaderInfo, fragShaderInfo};

	auto bindingDescriptions = getLevelBindings();
	auto attributeDescriptions = getLevelAttributes();

	VkPipelineVertexInputStateCreateInfo vertexInputInfo{};
	vertexInputInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO;
	vertexInputInfo.vertexBindingDescriptionCount = static_cast<uint32_t>(bindingDescriptions.size());
	vertexInputInfo.vertexAttributeDescriptionCount = static_cast<uint32_t>(attributeDescriptions.size());
	vertexInputInfo.pVertexBindingDescriptions = bindingDescriptions.data();
	vertexInputInfo.pVertexAttributeDescriptions = attributeDescriptions.data();

	VkPushConstantRange pushConstantRange{};
	pushConstantRange.stageFlags = VK_SHADER_STAGE_FRAGMENT_BIT; 
	pushConstantRange.offset = 0;
	pushConstantRange.size = sizeof(PushConstants);

	VkPipelineLayoutCreateInfo pipelineLayoutInfo{};
	pipelineLayoutInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO;
	pipelineLayoutInfo.setLayoutCount = 1;
	pipelineLayoutInfo.pSetLayouts = &this->descriptorSetLayout;
	pipelineLayoutInfo.pushConstantRangeCount = 1;
	pipelineLayoutInfo.pPushConstantRanges = &pushConstantRange;

	VkResult pipelineLayoutResult = vkCreatePipelineLayout(this->device, &pipelineLayoutInfo, nullptr, &this->levelPipelineLayout);
	if (pipelineLayoutResult != VK_SUCCESS) {
	    throw std::runtime_error("failed to create pipeline layout!");
	}

	VkGraphicsPipelineCreateInfo pipelineInfo{};
	pipelineInfo.sType = VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO;
	pipelineInfo.pNext = renderingInfo;
	pipelineInfo.stageCount = 2;
	pipelineInfo.pStages = shaderStages;
	pipelineInfo.pVertexInputState = &vertexInputInfo;
	pipelineInfo.pInputAssemblyState = inputAssembly;
	pipelineInfo.pViewportState = viewportState;
	pipelineInfo.pRasterizationState = rasterizer;
	pipelineInfo.pMultisampleState = multisampling;
	pipelineInfo.pDepthStencilState = depthStencil;
	pipelineInfo.pColorBlendState = colorBlending;
	pipelineInfo.pDynamicState = dynamicState;
	pipelineInfo.layout = this->levelPipelineLayout;
	pipelineInfo.subpass = 0;

	VkResult PipelineResult = vkCreateGraphicsPipelines(this->device, VK_NULL_HANDLE, 1, &pipelineInfo, nullptr, &this->levelPipeline);
	if (PipelineResult != VK_SUCCESS) {
    	throw std::runtime_error("failed to create graphics pipeline!");
	}
	
	vkDestroyShaderModule(this->device, fragShaderModule, nullptr);
    vkDestroyShaderModule(this->device, vertShaderModule, nullptr);
}

void VulkanRenderer::createUiPipeline(
	VkPipelineInputAssemblyStateCreateInfo* inputAssembly,
	VkPipelineViewportStateCreateInfo* viewportState,
	VkPipelineRasterizationStateCreateInfo* rasterizer,
	VkPipelineMultisampleStateCreateInfo* multisampling,
	VkPipelineColorBlendStateCreateInfo* colorBlending,
	VkPipelineDynamicStateCreateInfo* dynamicState,
	VkPipelineRenderingCreateInfo* renderingInfo
) {
	VkShaderModule vertShaderModule = createShaderModule(this->device, uiVertShader);
    VkShaderModule fragShaderModule = createShaderModule(this->device, uiFragShader);

	VkPipelineShaderStageCreateInfo vertShaderInfo{};
	vertShaderInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO;
	vertShaderInfo.stage = VK_SHADER_STAGE_VERTEX_BIT;
	vertShaderInfo.module = vertShaderModule;
	vertShaderInfo.pName = "main";

	VkPipelineShaderStageCreateInfo fragShaderInfo{};
	fragShaderInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO;
	fragShaderInfo.stage = VK_SHADER_STAGE_FRAGMENT_BIT;
	fragShaderInfo.module = fragShaderModule;
	fragShaderInfo.pName = "main";

	VkPipelineShaderStageCreateInfo shaderStages[] = {vertShaderInfo, fragShaderInfo};

	VkPipelineDepthStencilStateCreateInfo depthStencil{};
	depthStencil.sType = VK_STRUCTURE_TYPE_PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO;
	depthStencil.depthTestEnable = VK_FALSE;
	depthStencil.depthWriteEnable = VK_FALSE;
	depthStencil.depthCompareOp = VK_COMPARE_OP_LESS_OR_EQUAL;
	depthStencil.depthBoundsTestEnable = VK_FALSE;
	depthStencil.stencilTestEnable = VK_FALSE;

	auto bindingDescriptions = getUiBindings();
	auto attributeDescriptions = getUiAttributes();

	VkPipelineVertexInputStateCreateInfo vertexInputInfo{};
	vertexInputInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO;
	vertexInputInfo.vertexBindingDescriptionCount = static_cast<uint32_t>(bindingDescriptions.size());
	vertexInputInfo.vertexAttributeDescriptionCount = static_cast<uint32_t>(attributeDescriptions.size());
	vertexInputInfo.pVertexBindingDescriptions = bindingDescriptions.data();
	vertexInputInfo.pVertexAttributeDescriptions = attributeDescriptions.data();

	VkPushConstantRange pushConstantRange{};
	pushConstantRange.stageFlags = VK_SHADER_STAGE_FRAGMENT_BIT; 
	pushConstantRange.offset = 0;
	pushConstantRange.size = sizeof(uint32_t);

	VkPipelineLayoutCreateInfo pipelineLayoutInfo{};
	pipelineLayoutInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO;
	pipelineLayoutInfo.setLayoutCount = 1;
	pipelineLayoutInfo.pSetLayouts = &this->descriptorSetLayout;
	pipelineLayoutInfo.pushConstantRangeCount = 1;
	pipelineLayoutInfo.pPushConstantRanges = &pushConstantRange;

	VkResult pipelineLayoutResult = vkCreatePipelineLayout(this->device, &pipelineLayoutInfo, nullptr, &this->uiPipelineLayout);
	if (pipelineLayoutResult != VK_SUCCESS) {
	    throw std::runtime_error("failed to create pipeline layout!");
	}

	VkGraphicsPipelineCreateInfo pipelineInfo{};
	pipelineInfo.sType = VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO;
	pipelineInfo.pNext = renderingInfo;
	pipelineInfo.stageCount = 2;
	pipelineInfo.pStages = shaderStages;
	pipelineInfo.pVertexInputState = &vertexInputInfo;
	pipelineInfo.pInputAssemblyState = inputAssembly;
	pipelineInfo.pViewportState = viewportState;
	pipelineInfo.pRasterizationState = rasterizer;
	pipelineInfo.pMultisampleState = multisampling;
	pipelineInfo.pDepthStencilState = &depthStencil;
	pipelineInfo.pColorBlendState = colorBlending;
	pipelineInfo.pDynamicState = dynamicState;
	pipelineInfo.layout = this->uiPipelineLayout;
	pipelineInfo.subpass = 0;

	VkResult PipelineResult = vkCreateGraphicsPipelines(this->device, VK_NULL_HANDLE, 1, &pipelineInfo, nullptr, &this->uiPipeline);
	if (PipelineResult != VK_SUCCESS) {
    	throw std::runtime_error("failed to create graphics pipeline!");
	}
	
	vkDestroyShaderModule(this->device, fragShaderModule, nullptr);
    vkDestroyShaderModule(this->device, vertShaderModule, nullptr);
}
