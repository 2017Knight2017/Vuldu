#pragma once
#include <vulkan/vulkan.h>
#include <vulkan/vulkan_beta.h>
#include <memory>
#include <vector>
#include <array>
#include <optional>
#include "rust/cxx.h"

#ifdef DEBUG_MODE
const bool enableValidationLayers = true;
#else
const bool enableValidationLayers = false;
#endif

inline const uint32_t MAX_FRAMES_IN_FLIGHT = 2;
inline const uint32_t MAX_TEXTURES = 8192;
inline const uint32_t ANIM_INFO_SIZE = 4096;
inline const uint32_t MAX_SKY = 16;  
inline const uint32_t MAX_PAL = 14;
inline const uint32_t MAX_OBJECTS = 50000;
inline const uint32_t MAX_UI = 512;
inline const float PIXELS_IN_PANORAMA = 1024.0;

struct WindowHandles;
struct LevelVertex;
struct SpriteVertex;
struct MVP;
struct TextureDescriptor;
struct ObjectInstance;
struct UiInstance;
struct AnimLevelInfo;

struct QueueFamilyIndices {
    std::optional<uint32_t> graphicsFamily;
    std::optional<uint32_t> presentFamily;

    bool isComplete() {
        return graphicsFamily.has_value() && presentFamily.has_value();
    }
};

struct SwapChainSupportDetails {
    VkSurfaceCapabilitiesKHR capabilities;
    std::vector<VkSurfaceFormatKHR> formats;
    std::vector<VkPresentModeKHR> presentModes;
};

const std::vector<const char*> deviceExtensions = {
    VK_KHR_SWAPCHAIN_EXTENSION_NAME,
#if defined(__APPLE__)
    VK_KHR_PORTABILITY_SUBSET_EXTENSION_NAME
#endif
};

const std::vector<const char*> validationLayers = {
    "VK_LAYER_KHRONOS_validation"
};

struct ObjectPushConstants {
    uint32_t paletteIndex;
    uint32_t flags;
};

struct LevelPushConstants {
    std::array<float, 2> resolution;
    uint32_t paletteIndex;
    uint32_t skyIndex;
    float widthFactor;
    float globalTimer;
    float cameraYaw;
    uint32_t flags;
};

class VulkanRenderer {
public:
    VulkanRenderer();
    ~VulkanRenderer();
    void initVulkan(const WindowHandles& handles, uint32_t width, uint32_t height);
    void cleanup();
    void recreateSwapChain(uint32_t width, uint32_t height);
    void updateLevelGeometry(rust::Slice<const LevelVertex> vertices, rust::Slice<const uint32_t> indices);
    void updateObjectGeometry(rust::Slice<const SpriteVertex> vertices, rust::Slice<const uint32_t> indices);
    void updateUiGeometry(rust::Slice<const SpriteVertex> vertices, rust::Slice<const uint32_t> indices);
    void updateObjectInstances(rust::Slice<const ObjectInstance> instances);
    void updateUiInstances(rust::Slice<const UiInstance> instances);
    void uploadPalettes(rust::Slice<const uint8_t> palettes);
    void uploadColormap(rust::Slice<const uint8_t> colormap);
    void uploadTextureArray(
        rust::Slice<const TextureDescriptor> descriptors, 
        rust::Slice<const uint8_t> pixels, 
        rust::Slice<const float> sky_widths
    );
    void uploadAnimLevelInfo(rust::Slice<const AnimLevelInfo> info);
    void initSectorHeights(rust::Slice<const float> heights);
    void updateSectorHeights(rust::Slice<const float> heights);
    void initSwitches(rust::Slice<const uint32_t> switches);
    void updateSwitches(rust::Slice<const uint32_t> switches);
    void setPaletteIndex(uint32_t idx);
    uint32_t getPaletteIndex();
    void setSkyIndex(uint32_t idx);
    void setGlobalTimer(uint32_t global_timer);
    void setCameraYaw(float camera_yaw);
    void setFlags(uint32_t flags_to_invert);
    void startFrame(const MVP& mvp);
    void endFrame();
    void drawLevel();
    void drawObjects();
    void drawUi();
    
private:
    VkInstance instance = VK_NULL_HANDLE;
    VkDebugUtilsMessengerEXT debugMessenger = VK_NULL_HANDLE;
    VkSurfaceKHR surface = VK_NULL_HANDLE;

    VkPhysicalDevice physicalDevice = VK_NULL_HANDLE;
    VkDevice device = VK_NULL_HANDLE;
    VkQueue graphicsQueue = VK_NULL_HANDLE;
    VkQueue presentQueue = VK_NULL_HANDLE;

    VkSwapchainKHR swapChain = VK_NULL_HANDLE;
    VkFormat swapChainImageFormat = VK_FORMAT_UNDEFINED;
    VkExtent2D swapChainExtent{};
    std::vector<VkImage> swapChainImages;
    std::vector<VkImageView> swapChainImageViews;

    VkDescriptorSetLayout descriptorSetLayout = VK_NULL_HANDLE;
    VkPipelineLayout spritePipelineLayout = VK_NULL_HANDLE;
    VkPipeline spritePipeline = VK_NULL_HANDLE;
    VkPipelineLayout levelPipelineLayout = VK_NULL_HANDLE;
    VkPipeline levelPipeline = VK_NULL_HANDLE;
    VkPipelineLayout uiPipelineLayout = VK_NULL_HANDLE;
    VkPipeline uiPipeline = VK_NULL_HANDLE;

    VkCommandPool commandPool = VK_NULL_HANDLE;
    std::vector<VkCommandBuffer> commandBuffers;

    VkDescriptorPool descriptorPool = VK_NULL_HANDLE;
    std::vector<VkDescriptorSet> descriptorSets;
    std::vector<VkBuffer> MVPBuffers;
    std::vector<VkDeviceMemory> MVPBuffersMemory;
    std::vector<void*> MVPBuffersMapped;

    std::vector<VkBuffer> objectInstanceBuffers;
    std::vector<VkDeviceMemory> objectInstanceBuffersMemory;
    std::vector<void*> objectInstanceBuffersMapped;
    uint32_t activeObjectsCount = 0;

    std::vector<VkBuffer> uiInstanceBuffers;
    std::vector<VkDeviceMemory> uiInstanceBuffersMemory;
    std::vector<void*> uiInstanceBuffersMapped;
    uint32_t activeUiCount = 0;

    VkBuffer levelVertexBuffer = VK_NULL_HANDLE;
    VkDeviceMemory levelVertexBufferMemory = VK_NULL_HANDLE;
    VkBuffer levelIndexBuffer = VK_NULL_HANDLE;
    VkDeviceMemory levelIndexBufferMemory = VK_NULL_HANDLE;
    uint32_t levelVertexCount = 0;
    uint32_t levelIndexCount = 0;

    VkBuffer objectVertexBuffer = VK_NULL_HANDLE;
    VkDeviceMemory objectVertexBufferMemory = VK_NULL_HANDLE;
    VkBuffer objectIndexBuffer = VK_NULL_HANDLE;
    VkDeviceMemory objectIndexBufferMemory = VK_NULL_HANDLE;
    uint32_t objectVertexCount = 0;
    uint32_t objectIndexCount = 0;

    VkBuffer uiVertexBuffer = VK_NULL_HANDLE;
    VkDeviceMemory uiVertexBufferMemory = VK_NULL_HANDLE;
    VkBuffer uiIndexBuffer = VK_NULL_HANDLE;
    VkDeviceMemory uiIndexBufferMemory = VK_NULL_HANDLE;
    uint32_t uiVertexCount = 0;
    uint32_t uiIndexCount = 0;

    std::vector<VkImage> textureImages;
    std::vector<VkDeviceMemory> textureImageMemories;
    std::vector<VkImageView> textureImageViews;
    VkSampler textureSampler = VK_NULL_HANDLE;

    VkImage depthImage = VK_NULL_HANDLE;
    VkDeviceMemory depthImageMemory = VK_NULL_HANDLE;
    VkImageView depthImageView = VK_NULL_HANDLE;

    uint32_t currentFrame = 0;
    uint32_t currentImageIndex = 0;
    std::vector<VkSemaphore> imageAvailableSemaphores;
    std::vector<VkSemaphore> renderFinishedSemaphores;
    std::vector<VkFence> inFlightFences;

    VkImage paletteImage = VK_NULL_HANDLE;
    VkDeviceMemory paletteImageMemory = VK_NULL_HANDLE;
    VkImageView paletteImageView = VK_NULL_HANDLE;
    VkImage colormapImage = VK_NULL_HANDLE;
    VkDeviceMemory colormapImageMemory = VK_NULL_HANDLE;
    VkImageView colormapImageView = VK_NULL_HANDLE;

    VkBuffer animLevelBuffer = VK_NULL_HANDLE;
    VkDeviceMemory animLevelBufferMemory = VK_NULL_HANDLE;

    VkBuffer sectorHeightsBuffer = VK_NULL_HANDLE;
    VkDeviceMemory sectorHeightsBufferMemory = VK_NULL_HANDLE;
    void* sectorHeightsBufferMapped = VK_NULL_HANDLE;
    VkBuffer switchesBuffer = VK_NULL_HANDLE;
    VkDeviceMemory switchesBufferMemory = VK_NULL_HANDLE;
    void* switchesBufferMapped = VK_NULL_HANDLE;

    float globalTimer = 0.0;
    uint32_t currentPaletteIndex = 0;
    uint32_t currentSkyIndex = 0;
    std::vector<float> skyWidths;
    float cameraYaw = 0.0;
    uint32_t flags = 0;
    
    
    void createInstance(const WindowHandles& handles);
    void setupDebugMessenger();
    void createSurface(const WindowHandles& handles);
    void pickPhysicalDevice();
    void createLogicalDevice();
    void createSwapChain(uint32_t width, uint32_t height);
    void createImageViews();
    void createDescriptorSetLayout();
    void createPipelines();
    void createDepthResources();
    void createMVPBuffer();
    void createObjectInstanceBuffers();
    void createUiInstanceBuffers();
    void createDescriptorPool();
    void createDescriptorSets();
    void createTextureSamplers();
    void createCommandPool();
    void createCommandBuffers();
    void createSyncObjects();

    void updateMVPBuffer(const MVP& mvp);

    void cleanupSwapChain();

    void createBuffer(VkDeviceSize bufferSize, VkBufferUsageFlags usage, 
        VkMemoryPropertyFlags properties, VkBuffer& buffer, VkDeviceMemory& bufferMemory);
    void copyBuffer(VkBuffer srcBuffer, VkBuffer dstBuffer, VkDeviceSize size);
    void createBufferBinding(const void* data_ptr, VkDeviceSize bufferSize, VkBuffer& dstBuffer, 
    	VkDeviceMemory& dstBufferMemory, uint32_t dstBinding);
    void createImage(uint32_t width, uint32_t height, VkFormat format, 
        VkImageUsageFlags usage, VkMemoryPropertyFlags properties, VkImage& image, 
        VkDeviceMemory& imageMemory);
    void beginRendering(VkCommandBuffer currentCommandBuffer);

    void updateGeometry(
    	const void* vertices, 
	    rust::Slice<const uint32_t> indices,
	    VkBuffer& vertexBuffer,
	    VkDeviceMemory& vertexBufferMemory,
	    VkDeviceSize vertexBufferSize,
	    VkBuffer& indexBuffer,
	    VkDeviceMemory& indexBufferMemory,
	    VkDeviceSize indexBufferSize
    );

    void createDataTexture(
        const void* data_ptr, 
        size_t width,
        size_t height,
        VkFormat format,
        VkImage& dstImage, 
        VkDeviceMemory& dstImageMemory,
        VkImageView& dstImageView,
        uint32_t dstBinding
    );
    
    VkCommandBuffer beginSingleTimeCommands();
    void endSingleTimeCommands(VkCommandBuffer commandBuffer);
    VkFormat findDepthFormat();

    void createObjectPipeline(
    	VkPipelineInputAssemblyStateCreateInfo* inputAssembly,
    	VkPipelineViewportStateCreateInfo* viewportState,
    	VkPipelineRasterizationStateCreateInfo* rasterizer,
    	VkPipelineMultisampleStateCreateInfo* multisampling,
    	VkPipelineDepthStencilStateCreateInfo* depthStencil,
    	VkPipelineColorBlendStateCreateInfo* colorBlending,
    	VkPipelineDynamicStateCreateInfo* dynamicState,
    	VkPipelineRenderingCreateInfo* renderingInfo
    );
    void createLevelPipeline(
    	VkPipelineInputAssemblyStateCreateInfo* inputAssembly,
    	VkPipelineViewportStateCreateInfo* viewportState,
    	VkPipelineRasterizationStateCreateInfo* rasterizer,
    	VkPipelineMultisampleStateCreateInfo* multisampling,
    	VkPipelineDepthStencilStateCreateInfo* depthStencil,
    	VkPipelineColorBlendStateCreateInfo* colorBlending,
    	VkPipelineDynamicStateCreateInfo* dynamicState,
    	VkPipelineRenderingCreateInfo* renderingInfo
    );
    void createUiPipeline(
    	VkPipelineInputAssemblyStateCreateInfo* inputAssembly,
    	VkPipelineViewportStateCreateInfo* viewportState,
    	VkPipelineRasterizationStateCreateInfo* rasterizer,
    	VkPipelineMultisampleStateCreateInfo* multisampling,
        VkPipelineColorBlendStateCreateInfo* colorBlending,
    	VkPipelineDynamicStateCreateInfo* dynamicState,
    	VkPipelineRenderingCreateInfo* renderingInfo
    );

    void populateDebugMessengerCreateInfo(VkDebugUtilsMessengerCreateInfoEXT& createInfo);
    void DestroyDebugUtilsMessengerEXT(VkInstance instance, VkDebugUtilsMessengerEXT debugMessenger, const VkAllocationCallbacks* pAllocator);
};

bool isDeviceSuitable(VkPhysicalDevice physicalDevice, VkSurfaceKHR surface);
QueueFamilyIndices findQueueFamilies(VkPhysicalDevice physicalDevice, VkSurfaceKHR surface);
SwapChainSupportDetails querySwapChainSupport(VkPhysicalDevice physicalDevice, VkSurfaceKHR surface);
bool checkDeviceExtensionSupport(VkPhysicalDevice physicalDevice);
bool checkValidationLayerSupport();

std::unique_ptr<VulkanRenderer> createRenderer();
size_t getMaxSky();
size_t getAnimInfoSize();
