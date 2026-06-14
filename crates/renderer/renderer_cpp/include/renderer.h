#pragma once
#define VK_USE_PLATFORM_WAYLAND_KHR
#include <vulkan/vulkan.h>
#include <memory>
#include <vector>
#include <optional>

#ifdef DEBUG_MODE
const bool enableValidationLayers = true;
#else
const bool enableValidationLayers = false;
#endif

inline const uint32_t MAX_FRAMES_IN_FLIGHT = 2;
inline const uint32_t MAX_TEXTURES = 512;  // Maximal amount of textures on a level
inline const uint32_t MAX_PAL = 14;
inline const uint32_t MAX_LIGHTLEVEL = 32;

struct WindowHandles;
struct Vertex;
struct UniformBufferObject;

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

const std::vector<const char*> extensions = {
    VK_KHR_SURFACE_EXTENSION_NAME, 
    VK_KHR_WAYLAND_SURFACE_EXTENSION_NAME
};

const std::vector<const char*> deviceExtensions = {
    VK_KHR_SWAPCHAIN_EXTENSION_NAME
};

const std::vector<const char*> validationLayers = {
    "VK_LAYER_KHRONOS_validation"
};

struct LevelPushConstants {
    uint32_t paletteIndex;
    uint32_t lightLevel;     
};

struct SpritePushConstants {
    uint32_t paletteIndex;
    uint32_t lightLevel;      
    uint32_t textureId;         
    float spriteWidth;         
    float spriteHeight;        
    float leftOffset;          
    float topOffset;           
    float padding;    
    float worldPos[4];         
};

class VulkanRenderer {
public:
    VulkanRenderer();
    ~VulkanRenderer();
    void initVulkan(const WindowHandles& handles, size_t window_raw_ptr);
    void cleanup();
    uint32_t addTexture(const uint8_t* pixels_ptr, uint32_t width, uint32_t height);
    void updateGeometry(const Vertex* vertices_ptr, size_t vertex_count, const uint16_t* indices_ptr, size_t index_count);
    void uploadPalettes(const float* palettes_ptr);
    void uploadColormap(const uint8_t* colormap_ptr);
    void setPaletteIndex(uint32_t idx);
    void startFrame(const UniformBufferObject* ubo_ptr);
    void endFrame();
    void drawSprite(uint32_t textureId, uint32_t width, uint32_t height, 
                    uint32_t lightLevel, int16_t leftOffset, int16_t topOffset, 
                    float x, float y, float z);
    void drawLevel();
    
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
    std::vector<VkFramebuffer> swapChainFramebuffers;

    VkRenderPass renderPass = VK_NULL_HANDLE;
    VkDescriptorSetLayout descriptorSetLayout = VK_NULL_HANDLE;
    VkPipelineLayout spritePipelineLayout = VK_NULL_HANDLE;
    VkPipeline spritePipeline = VK_NULL_HANDLE;
    VkPipelineLayout levelPipelineLayout = VK_NULL_HANDLE;
    VkPipeline levelPipeline = VK_NULL_HANDLE;

    VkCommandPool commandPool = VK_NULL_HANDLE;
    std::vector<VkCommandBuffer> commandBuffers;

    VkDescriptorPool descriptorPool = VK_NULL_HANDLE;
    std::vector<VkDescriptorSet> descriptorSets;
    std::vector<VkBuffer> uniformBuffers;
    std::vector<VkDeviceMemory> uniformBuffersMemory;
    std::vector<void*> uniformBuffersMapped;

    VkBuffer vertexBuffer = VK_NULL_HANDLE;
    VkDeviceMemory vertexBufferMemory = VK_NULL_HANDLE;
    uint32_t vertexCount = 0;

    VkBuffer indexBuffer = VK_NULL_HANDLE;
    VkDeviceMemory indexBufferMemory = VK_NULL_HANDLE;
    uint32_t indexCount = 0;

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

    uint32_t currentPaletteIndex = 0;
    VkBuffer paletteBuffer = VK_NULL_HANDLE;
    VkDeviceMemory paletteBufferMemory = VK_NULL_HANDLE;

    VkBuffer colormapBuffer = VK_NULL_HANDLE;
    VkDeviceMemory colormapBufferMemory = VK_NULL_HANDLE;
    
    void createInstance();
    void setupDebugMessenger();
    void createSurface(const WindowHandles& handles);
    void pickPhysicalDevice();
    void createLogicalDevice();
    void createSwapChain();
    void createImageViews();
    void createRenderPass();
    void createDescriptorSetLayout();
    void createGraphicsPipeline();
    void createDepthResources();
    void createFramebuffers();
    void createUniformBuffers();
    void createDescriptorPool();
    void createDescriptorSets();
    void createTextureSampler();
    void createCommandPool();
    void createCommandBuffers();
    void createSyncObjects();

    void updateUniformBuffer(const UniformBufferObject* ubo_ptr, uint32_t currentImage);
    void updateDescriptorSetWithTexture(uint32_t textureId, VkImageView newView);

    void recreateSwapChain();
    void cleanupSwapChain();

    void createBuffer(VkDeviceSize bufferSize, VkBufferUsageFlags usage, VkMemoryPropertyFlags properties, VkBuffer& buffer, VkDeviceMemory& bufferMemory);
    void copyBuffer(VkBuffer srcBuffer, VkBuffer dstBuffer, VkDeviceSize size);
    void createImage(uint32_t width, uint32_t height, VkFormat format, VkImageTiling tiling, VkImageUsageFlags usage, VkMemoryPropertyFlags properties, VkImage& image, VkDeviceMemory& imageMemory);
    
    VkCommandBuffer beginSingleTimeCommands();
    void endSingleTimeCommands(VkCommandBuffer commandBuffer);
    void transitionImageLayout(VkImage image, VkImageLayout oldLayout, VkImageLayout newLayout);
    void copyBufferToImage(VkBuffer buffer, VkImage image, uint32_t width, uint32_t height);
    VkFormat findDepthFormat();

    bool isDeviceSuitable(VkPhysicalDevice device);
    QueueFamilyIndices findQueueFamilies(VkPhysicalDevice device);
    SwapChainSupportDetails querySwapChainSupport(VkPhysicalDevice device);
    bool checkDeviceExtensionSupport(VkPhysicalDevice device);
    bool checkValidationLayerSupport();

    void populateDebugMessengerCreateInfo(VkDebugUtilsMessengerCreateInfoEXT& createInfo);
    void DestroyDebugUtilsMessengerEXT(VkInstance instance, VkDebugUtilsMessengerEXT debugMessenger, const VkAllocationCallbacks* pAllocator);
};

std::unique_ptr<VulkanRenderer> createRenderer();