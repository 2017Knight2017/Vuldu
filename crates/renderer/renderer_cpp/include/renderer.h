#pragma once
#include <vulkan/vulkan.h>
#include <vulkan/vulkan_beta.h>
#include <memory>
#include <vector>
#include <optional>

#ifdef DEBUG_MODE
const bool enableValidationLayers = true;
#else
const bool enableValidationLayers = false;
#endif

inline const uint32_t MAX_FRAMES_IN_FLIGHT = 2;
inline const uint32_t MAX_TEXTURES = 8192;
inline const uint32_t MAX_SKY = 16;  
inline const uint32_t MAX_PAL = 14;
inline const uint32_t MAX_OBJECTS = 50000;
inline const uint32_t MAX_UI = 256;
inline const size_t ANIM_INFO_NUM = 22;
inline const float PIXELS_IN_PANORAMA = 1024.0;

struct WindowHandles;
struct Vertex;
struct UniformBufferObject;
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

struct PushConstants {
    uint32_t paletteIndex;
    float resolution[2];  
    uint32_t skyIndex;
    float widthFactor;
    float globalTimer;
    float cameraYaw;
    bool wireframe;
    bool _padding[3];
};

class VulkanRenderer {
public:
    VulkanRenderer();
    ~VulkanRenderer();
    void initVulkan(const WindowHandles& handles, size_t window_raw_ptr);
    void cleanup();
    void recreateSwapChain();
    void updateLevelGeometry(const Vertex* vertices_ptr, size_t vertex_count, const uint32_t* indices_ptr, size_t index_count);
    void updateObjectGeometry(const Vertex* vertices_ptr, size_t vertex_count, const uint32_t* indices_ptr, size_t index_count);
    void updateUiGeometry(const Vertex* vertices_ptr, size_t vertex_count, const uint32_t* indices_ptr, size_t index_count);
    void updateObjectInstances(const ObjectInstance* instances_ptr, size_t instances_count);
    void updateUiInstances(const UiInstance* instances_ptr, size_t instances_count);
    void uploadPalettes(const float* palettes_ptr, size_t colormap_bytes_count);
    void uploadColormap(const uint8_t* colormap_ptr, size_t colormap_bytes_count);
    void uploadTextureArray(
        const TextureDescriptor* descriptors_ptr, 
        size_t descriptor_count, 
        const uint8_t* all_pixels_ptr, 
        size_t all_pixels_count, 
        const float* sky_widths_ptr, 
        size_t sky_widths_count
    );
    void uploadAnimLevelInfo(const AnimLevelInfo* info_ptr, size_t info_count);
    void setPaletteIndex(uint32_t idx);
    void setSkyIndex(uint32_t idx);
    void setGlobalTimer(uint32_t global_timer);
    void setCameraYaw(float camera_yaw);
    void setWireframe(bool flag);
    void startFrame(const UniformBufferObject* ubo_ptr);
    void endFrame();
    void drawLevel();
    void drawObjects();
    void drawUi();
    
private:
    size_t window_raw_ptr = 0;
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
    std::vector<VkBuffer> uniformBuffers;
    std::vector<VkDeviceMemory> uniformBuffersMemory;
    std::vector<void*> uniformBuffersMapped;

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

    VkBuffer paletteBuffer = VK_NULL_HANDLE;
    VkDeviceMemory paletteBufferMemory = VK_NULL_HANDLE;
    VkBuffer colormapBuffer = VK_NULL_HANDLE;
    VkDeviceMemory colormapBufferMemory = VK_NULL_HANDLE;
    VkBuffer animLevelBuffer = VK_NULL_HANDLE;
    VkDeviceMemory animLevelBufferMemory = VK_NULL_HANDLE;

    float globalTimer = 0.0;
    uint32_t currentPaletteIndex = 0;
    uint32_t currentSkyIndex = 0;
    std::vector<float> skyWidths;
    float cameraYaw = 0.0;
    bool wireframe = false;
    
    
    void createInstance(const WindowHandles& handles);
    void setupDebugMessenger();
    void createSurface(const WindowHandles& handles);
    void pickPhysicalDevice();
    void createLogicalDevice();
    void createSwapChain();
    void createImageViews();
    void createDescriptorSetLayout();
    void createPipelines();
    void createDepthResources();
    void createUniformBuffers();
    void createObjectInstanceBuffers();
    void createUiInstanceBuffers();
    void createDescriptorPool();
    void createDescriptorSets();
    void createTextureSamplers();
    void createCommandPool();
    void createCommandBuffers();
    void createSyncObjects();

    void updateUniformBuffer(const UniformBufferObject* ubo_ptr);

    void cleanupSwapChain();

    void createBuffer(VkDeviceSize bufferSize, VkBufferUsageFlags usage, 
        VkMemoryPropertyFlags properties, VkBuffer& buffer, VkDeviceMemory& bufferMemory);
    void copyBuffer(VkBuffer srcBuffer, VkBuffer dstBuffer, VkDeviceSize size);
    void createBinding(const void* data_ptr, VkDeviceSize bufferSize, VkBuffer& dstBuffer, 
    	VkDeviceMemory& dstBufferMemory, uint32_t dstBinding, bool isStorage
    );
    void createImage(uint32_t width, uint32_t height, VkFormat format, 
        VkImageUsageFlags usage, VkMemoryPropertyFlags properties, VkImage& image, 
        VkDeviceMemory& imageMemory);
    void beginRendering(VkCommandBuffer currentCommandBuffer);
    
    VkCommandBuffer beginSingleTimeCommands();
    void endSingleTimeCommands(VkCommandBuffer commandBuffer);
    VkFormat findDepthFormat();

    void createSpritePipeline(
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

    bool isDeviceSuitable(VkPhysicalDevice device);
    QueueFamilyIndices findQueueFamilies(VkPhysicalDevice device);
    SwapChainSupportDetails querySwapChainSupport(VkPhysicalDevice device);
    bool checkDeviceExtensionSupport(VkPhysicalDevice device);
    bool checkValidationLayerSupport();

    void populateDebugMessengerCreateInfo(VkDebugUtilsMessengerCreateInfoEXT& createInfo);
    void DestroyDebugUtilsMessengerEXT(VkInstance instance, VkDebugUtilsMessengerEXT debugMessenger, const VkAllocationCallbacks* pAllocator);
};

std::unique_ptr<VulkanRenderer> createRenderer();
size_t getMaxSky();
