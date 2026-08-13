#include <cstring>
#include <stdexcept>
#include "renderer.h"
#include "renderer/src/bridge.rs.h"
#include "utils.h"

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
		{ 5, 0, VK_FORMAT_R32_UINT,         offsetof(Vertex, floor_tex_id) }
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

void VulkanRenderer::createGraphicsPipeline() {
    auto spriteVertShader = readFile("crates/renderer/renderer_cpp/shaders/sprite_vert.spv");
    auto spriteFragShader = readFile("crates/renderer/renderer_cpp/shaders/sprite_frag.spv");

	VkShaderModule spriteVertShaderModule = createShaderModule(this->device, spriteVertShader);
    VkShaderModule spriteFragShaderModule = createShaderModule(this->device, spriteFragShader);

	VkPipelineShaderStageCreateInfo spriteVertShaderInfo{};
	spriteVertShaderInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO;
	spriteVertShaderInfo.stage = VK_SHADER_STAGE_VERTEX_BIT;
	spriteVertShaderInfo.module = spriteVertShaderModule;
	spriteVertShaderInfo.pName = "main";

	VkPipelineShaderStageCreateInfo spriteFragShaderInfo{};
	spriteFragShaderInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO;
	spriteFragShaderInfo.stage = VK_SHADER_STAGE_FRAGMENT_BIT;
	spriteFragShaderInfo.module = spriteFragShaderModule;
	spriteFragShaderInfo.pName = "main";

	VkPipelineShaderStageCreateInfo spriteShaderStages[] = {spriteVertShaderInfo, spriteFragShaderInfo};


	auto levelVertShader = readFile("crates/renderer/renderer_cpp/shaders/level_vert.spv");
    auto levelFragShader = readFile("crates/renderer/renderer_cpp/shaders/level_frag.spv");

	VkShaderModule levelVertShaderModule = createShaderModule(this->device, levelVertShader);
    VkShaderModule levelFragShaderModule = createShaderModule(this->device, levelFragShader);

	VkPipelineShaderStageCreateInfo levelVertShaderInfo{};
	levelVertShaderInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO;
	levelVertShaderInfo.stage = VK_SHADER_STAGE_VERTEX_BIT;
	levelVertShaderInfo.module = levelVertShaderModule;
	levelVertShaderInfo.pName = "main";

	VkPipelineShaderStageCreateInfo levelFragShaderInfo{};
	levelFragShaderInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO;
	levelFragShaderInfo.stage = VK_SHADER_STAGE_FRAGMENT_BIT;
	levelFragShaderInfo.module = levelFragShaderModule;
	levelFragShaderInfo.pName = "main";

	VkPipelineShaderStageCreateInfo levelShaderStages[] = {levelVertShaderInfo, levelFragShaderInfo};

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
	rasterizer.depthBiasConstantFactor = 0.0f; // Optional
	rasterizer.depthBiasClamp = 0.0f; // Optional
	rasterizer.depthBiasSlopeFactor = 0.0f; // Optional

	VkPipelineMultisampleStateCreateInfo multisampling{};
	multisampling.sType = VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO;
	multisampling.sampleShadingEnable = VK_FALSE;
	multisampling.rasterizationSamples = VK_SAMPLE_COUNT_1_BIT;
	multisampling.minSampleShading = 1.0f; // Optional
	multisampling.pSampleMask = nullptr; // Optional
	multisampling.alphaToCoverageEnable = VK_FALSE; // Optional
	multisampling.alphaToOneEnable = VK_FALSE; // Optional

	VkPipelineDepthStencilStateCreateInfo depthStencil{};
	depthStencil.sType = VK_STRUCTURE_TYPE_PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO;
	depthStencil.depthTestEnable = VK_TRUE;
	depthStencil.depthWriteEnable = VK_TRUE;
	depthStencil.depthCompareOp = VK_COMPARE_OP_LESS_OR_EQUAL;
	depthStencil.depthBoundsTestEnable = VK_FALSE;
	depthStencil.minDepthBounds = 0.0f; // Optional
	depthStencil.maxDepthBounds = 1.0f; // Optional
	depthStencil.stencilTestEnable = VK_FALSE;
	depthStencil.front = {}; // Optional
	depthStencil.back = {}; // Optional

	VkPipelineColorBlendAttachmentState colorBlendAttachment{};
	colorBlendAttachment.colorWriteMask = VK_COLOR_COMPONENT_R_BIT | VK_COLOR_COMPONENT_G_BIT | VK_COLOR_COMPONENT_B_BIT | VK_COLOR_COMPONENT_A_BIT;
	colorBlendAttachment.blendEnable = VK_FALSE;
	colorBlendAttachment.srcColorBlendFactor = VK_BLEND_FACTOR_ONE; // Optional
	colorBlendAttachment.dstColorBlendFactor = VK_BLEND_FACTOR_ZERO; // Optional
	colorBlendAttachment.colorBlendOp = VK_BLEND_OP_ADD; // Optional
	colorBlendAttachment.srcAlphaBlendFactor = VK_BLEND_FACTOR_ONE; // Optional
	colorBlendAttachment.dstAlphaBlendFactor = VK_BLEND_FACTOR_ZERO; // Optional
	colorBlendAttachment.alphaBlendOp = VK_BLEND_OP_ADD; // Optional

	VkPipelineColorBlendStateCreateInfo colorBlending{};
	colorBlending.sType = VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO;
	colorBlending.logicOpEnable = VK_FALSE;
	colorBlending.logicOp = VK_LOGIC_OP_COPY; // Optional
	colorBlending.attachmentCount = 1;
	colorBlending.pAttachments = &colorBlendAttachment;
	colorBlending.blendConstants[0] = 0.0f; // Optional
	colorBlending.blendConstants[1] = 0.0f; // Optional
	colorBlending.blendConstants[2] = 0.0f; // Optional
	colorBlending.blendConstants[3] = 0.0f; // Optional

	VkPipelineRenderingCreateInfo renderingInfo{};
	renderingInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_RENDERING_CREATE_INFO_KHR;
	renderingInfo.colorAttachmentCount = 1;
	renderingInfo.pColorAttachmentFormats = &this->swapChainImageFormat; 
	renderingInfo.depthAttachmentFormat = findDepthFormat();
	renderingInfo.stencilAttachmentFormat = VK_FORMAT_UNDEFINED;

	auto spriteBindingDescriptions = getSpriteBindings();
	auto spriteAttributeDescriptions = getSpriteAttributes();

	VkPipelineVertexInputStateCreateInfo spriteVertexInputInfo{};
	spriteVertexInputInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO;
	spriteVertexInputInfo.vertexBindingDescriptionCount = static_cast<uint32_t>(spriteBindingDescriptions.size());
	spriteVertexInputInfo.vertexAttributeDescriptionCount = static_cast<uint32_t>(spriteAttributeDescriptions.size());
	spriteVertexInputInfo.pVertexBindingDescriptions = spriteBindingDescriptions.data();
	spriteVertexInputInfo.pVertexAttributeDescriptions = spriteAttributeDescriptions.data();

	VkPushConstantRange spritePushConstantRange{};
	spritePushConstantRange.stageFlags = VK_SHADER_STAGE_FRAGMENT_BIT; 
	spritePushConstantRange.offset = 0;
	spritePushConstantRange.size = sizeof(uint32_t);

	VkPipelineLayoutCreateInfo spritePipelineLayoutInfo{};
	spritePipelineLayoutInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO;
	spritePipelineLayoutInfo.setLayoutCount = 1;
	spritePipelineLayoutInfo.pSetLayouts = &this->descriptorSetLayout;
	spritePipelineLayoutInfo.pushConstantRangeCount = 1;
	spritePipelineLayoutInfo.pPushConstantRanges = &spritePushConstantRange;

	VkResult spritePipelineLayoutResult = vkCreatePipelineLayout(this->device, &spritePipelineLayoutInfo, nullptr, &spritePipelineLayout);
	if (spritePipelineLayoutResult != VK_SUCCESS) {
	    throw std::runtime_error("failed to create pipeline layout!");
	}

	VkGraphicsPipelineCreateInfo spritePipelineInfo{};
	spritePipelineInfo.sType = VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO;
	spritePipelineInfo.pNext = &renderingInfo;
	spritePipelineInfo.stageCount = 2;
	spritePipelineInfo.pStages = spriteShaderStages;
	spritePipelineInfo.pVertexInputState = &spriteVertexInputInfo;
	spritePipelineInfo.pInputAssemblyState = &inputAssembly;
	spritePipelineInfo.pViewportState = &viewportState;
	spritePipelineInfo.pRasterizationState = &rasterizer;
	spritePipelineInfo.pMultisampleState = &multisampling;
	spritePipelineInfo.pDepthStencilState = &depthStencil;
	spritePipelineInfo.pColorBlendState = &colorBlending;
	spritePipelineInfo.pDynamicState = &dynamicState;
	spritePipelineInfo.layout = this->spritePipelineLayout;
	spritePipelineInfo.subpass = 0;
	spritePipelineInfo.basePipelineHandle = VK_NULL_HANDLE; // Optional
	spritePipelineInfo.basePipelineIndex = -1; // Optional

	VkResult spritePipelineResult = vkCreateGraphicsPipelines(this->device, VK_NULL_HANDLE, 1, &spritePipelineInfo, nullptr, &spritePipeline);
	if (spritePipelineResult != VK_SUCCESS) {
    	throw std::runtime_error("failed to create graphics pipeline!");
	}
	
	vkDestroyShaderModule(this->device, spriteFragShaderModule, nullptr);
    vkDestroyShaderModule(this->device, spriteVertShaderModule, nullptr);

	auto levelBindingDescriptions = getLevelBindings();
	auto levelAttributeDescriptions = getLevelAttributes();

	VkPipelineVertexInputStateCreateInfo levelVertexInputInfo{};
	levelVertexInputInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO;
	levelVertexInputInfo.vertexBindingDescriptionCount = static_cast<uint32_t>(levelBindingDescriptions.size());
	levelVertexInputInfo.vertexAttributeDescriptionCount = static_cast<uint32_t>(levelAttributeDescriptions.size());
	levelVertexInputInfo.pVertexBindingDescriptions = levelBindingDescriptions.data();
	levelVertexInputInfo.pVertexAttributeDescriptions = levelAttributeDescriptions.data();

	VkPushConstantRange levelPushConstantRange{};
	levelPushConstantRange.stageFlags = VK_SHADER_STAGE_FRAGMENT_BIT; 
	levelPushConstantRange.offset = 0;
	levelPushConstantRange.size = sizeof(PushConstants);

	VkPipelineLayoutCreateInfo levelPipelineLayoutInfo{};
	levelPipelineLayoutInfo.sType = VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO;
	levelPipelineLayoutInfo.setLayoutCount = 1;
	levelPipelineLayoutInfo.pSetLayouts = &this->descriptorSetLayout;
	levelPipelineLayoutInfo.pushConstantRangeCount = 1;
	levelPipelineLayoutInfo.pPushConstantRanges = &levelPushConstantRange;

	VkResult levelPipelineLayoutResult = vkCreatePipelineLayout(this->device, &levelPipelineLayoutInfo, nullptr, &levelPipelineLayout);
	if (levelPipelineLayoutResult != VK_SUCCESS) {
	    throw std::runtime_error("failed to create pipeline layout!");
	}

	VkGraphicsPipelineCreateInfo levelPipelineInfo{};
	levelPipelineInfo.sType = VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO;
	levelPipelineInfo.pNext = &renderingInfo;
	levelPipelineInfo.stageCount = 2;
	levelPipelineInfo.pStages = levelShaderStages;
	levelPipelineInfo.pVertexInputState = &levelVertexInputInfo;
	levelPipelineInfo.pInputAssemblyState = &inputAssembly;
	levelPipelineInfo.pViewportState = &viewportState;
	levelPipelineInfo.pRasterizationState = &rasterizer;
	levelPipelineInfo.pMultisampleState = &multisampling;
	levelPipelineInfo.pDepthStencilState = &depthStencil;
	levelPipelineInfo.pColorBlendState = &colorBlending;
	levelPipelineInfo.pDynamicState = &dynamicState;
	levelPipelineInfo.layout = this->levelPipelineLayout;
	levelPipelineInfo.subpass = 0;
	levelPipelineInfo.basePipelineHandle = VK_NULL_HANDLE; // Optional
	levelPipelineInfo.basePipelineIndex = -1; // Optional

	VkResult levelPipelineResult = vkCreateGraphicsPipelines(this->device, VK_NULL_HANDLE, 1, &levelPipelineInfo, nullptr, &levelPipeline);
	if (levelPipelineResult != VK_SUCCESS) {
    	throw std::runtime_error("failed to create graphics pipeline!");
	}
	
	vkDestroyShaderModule(this->device, levelFragShaderModule, nullptr);
    vkDestroyShaderModule(this->device, levelVertShaderModule, nullptr);
}
