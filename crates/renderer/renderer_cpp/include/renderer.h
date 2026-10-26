#pragma once
#define VULKAN_HPP_HAS_SPACESHIP_OPERATOR
#include <vulkan/vulkan_raii.hpp>
#include <vulkan/vulkan_beta.h>
#include <memory>
#include <vector>
#include <array>
#include <optional>
#include "rust/cxx.h"

#ifdef DEBUG_MODE
inline constexpr bool enableValidationLayers = true;
#else
inline constexpr bool enableValidationLayers = false;
#endif

inline constexpr uint32_t MAX_FRAMES_IN_FLIGHT = 2;
inline constexpr uint32_t PIPELINES_COUNT = 4;
inline constexpr uint32_t DESCRIPTORS_COUNT = 7;
inline constexpr uint32_t MAX_TEXTURES = 8192;
inline constexpr uint32_t ANIM_INFO_SIZE = 4096;
inline constexpr uint32_t MAX_SKY = 16;  
inline constexpr uint32_t MAX_PAL = 14;
inline constexpr uint32_t MAX_OBJECTS = 50000;
inline constexpr uint32_t MAX_UI = 512;
inline constexpr float PIXELS_IN_PANORAMA = 1024.0;

inline constexpr std::array<vk::ShaderStageFlags, PIPELINES_COUNT> SHADER_STAGE_FLAGS = {
	vk::ShaderStageFlagBits::eVertex | vk::ShaderStageFlagBits::eFragment,
	vk::ShaderStageFlagBits::eFragment,
	vk::ShaderStageFlagBits::eFragment,
	vk::ShaderStageFlagBits::eFragment
};

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
	vk::SurfaceCapabilitiesKHR capabilities;
	std::vector<vk::SurfaceFormatKHR> formats;
	std::vector<vk::PresentModeKHR> presentModes;
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

enum PipelineType {
	Level,
	Object,
	Ui,
	Weapon
};

struct SwapchainResources {
	vk::raii::SwapchainKHR swapchain{VK_NULL_HANDLE};
	vk::Format imageFormat = vk::Format::eUndefined;
	vk::Extent2D extent{};
	std::vector<vk::Image> images;
	std::vector<vk::raii::ImageView> imageViews;
	vk::raii::Image depthImage{VK_NULL_HANDLE};
	vk::raii::DeviceMemory depthImageMemory{VK_NULL_HANDLE};
	vk::raii::ImageView depthImageView{VK_NULL_HANDLE};
};

struct InstanceBuffers {
    std::array<vk::raii::Buffer, MAX_FRAMES_IN_FLIGHT> buffers{nullptr, nullptr};
    std::array<vk::raii::DeviceMemory, MAX_FRAMES_IN_FLIGHT> memories{nullptr, nullptr};
    std::array<void*, MAX_FRAMES_IN_FLIGHT> mapped{nullptr, nullptr};
    uint32_t activeCount = 0;
};

struct DynamicBuffer {
	vk::raii::Buffer buffer{VK_NULL_HANDLE};
	vk::raii::DeviceMemory memory{VK_NULL_HANDLE};
	void* mapped{VK_NULL_HANDLE};
};

struct DataTexture {
	vk::raii::Image image{VK_NULL_HANDLE};
	vk::raii::DeviceMemory memory{VK_NULL_HANDLE};
	vk::raii::ImageView view{VK_NULL_HANDLE};
};

struct MeshBuffers {
    vk::raii::Buffer vertexBuffer{VK_NULL_HANDLE};
    vk::raii::DeviceMemory vertexBufferMemory{VK_NULL_HANDLE};
    uint32_t vertexCount = 0;

    vk::raii::Buffer indexBuffer{VK_NULL_HANDLE};
    vk::raii::DeviceMemory indexBufferMemory{VK_NULL_HANDLE};
    uint32_t indexCount = 0;
};

class VulkanRenderer {
public:
	VulkanRenderer();
	~VulkanRenderer();
	void initVulkan(const WindowHandles& handles, uint32_t width, uint32_t height);
	void recreateSwapChain(uint32_t width, uint32_t height);
	void updateLevelGeometry(rust::Slice<const LevelVertex> vertices, rust::Slice<const uint32_t> indices);
	void updateObjectGeometry(rust::Slice<const SpriteVertex> vertices, rust::Slice<const uint32_t> indices);
	void updateUiGeometry(rust::Slice<const SpriteVertex> vertices, rust::Slice<const uint32_t> indices);
	void updateWeaponGeometry(rust::Slice<const SpriteVertex> vertices, rust::Slice<const uint32_t> indices);
	void updateObjectInstances(rust::Slice<const ObjectInstance> instances);
	void updateUiInstances(rust::Slice<const UiInstance> instances);
	void updateWeaponInstance(const UiInstance& instance);
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
	void drawLevel() noexcept;
	void drawObjects() noexcept;
	void drawUi() noexcept;
	void drawWeapon() noexcept;
	
private:
    vk::raii::Context context{};
    vk::raii::Instance instance = VK_NULL_HANDLE;
    vk::raii::DebugUtilsMessengerEXT debugMessenger = VK_NULL_HANDLE;
    vk::raii::SurfaceKHR surface = VK_NULL_HANDLE;

    vk::raii::PhysicalDevice physicalDevice = VK_NULL_HANDLE;
    vk::raii::Device device = VK_NULL_HANDLE;
    vk::raii::Queue graphicsQueue = VK_NULL_HANDLE;
    vk::raii::Queue presentQueue = VK_NULL_HANDLE;

    std::vector<vk::raii::Semaphore> renderFinishedSemaphores;
    std::array<vk::raii::Semaphore, MAX_FRAMES_IN_FLIGHT> imageAvailableSemaphores{nullptr, nullptr};
    std::array<vk::raii::Fence, MAX_FRAMES_IN_FLIGHT> inFlightFences{nullptr, nullptr};

    std::optional<SwapchainResources> swapchainRes{};

    vk::raii::DescriptorSetLayout descriptorSetLayout = VK_NULL_HANDLE;
    std::array<vk::raii::PipelineLayout, PIPELINES_COUNT> pipelineLayouts{nullptr, nullptr, nullptr, nullptr};
    std::array<vk::raii::Pipeline, PIPELINES_COUNT> pipelines{nullptr, nullptr, nullptr, nullptr};

    vk::raii::DescriptorPool descriptorPool = VK_NULL_HANDLE;
    std::array<vk::raii::DescriptorSet, MAX_FRAMES_IN_FLIGHT> descriptorSets{nullptr, nullptr};

    vk::raii::CommandPool commandPool = VK_NULL_HANDLE;
    std::array<vk::raii::CommandBuffer, MAX_FRAMES_IN_FLIGHT> commandBuffers{nullptr, nullptr};

    std::array<vk::raii::Buffer, MAX_FRAMES_IN_FLIGHT> MVPBuffers{nullptr, nullptr};
    std::array<vk::raii::DeviceMemory, MAX_FRAMES_IN_FLIGHT> MVPBuffersMemory{nullptr, nullptr};
    std::array<void*, MAX_FRAMES_IN_FLIGHT> MVPBuffersMapped{nullptr, nullptr};

    MeshBuffers levelMesh{}, objectMesh{}, uiMesh{}, weaponMesh{};
    InstanceBuffers objectInst{}, uiInst{}, weaponInst{};

    std::vector<vk::raii::Image> textureImages;
    std::vector<vk::raii::ImageView> textureImageViews;
	std::vector<vk::raii::DeviceMemory> textureImageMemories;
    vk::raii::Sampler textureSampler = VK_NULL_HANDLE;

    DataTexture palette{}, colormap{};
    DynamicBuffer animLevel{}, sectorHeights{}, switches{};

    uint32_t currentFrame = 0;
    uint32_t currentImageIndex = 0;

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
	void createDescriptorSetLayout();
	void createPipelines();
	void createDepthResources();
	void createMVPBuffer();
	void createInstanceBuffers();
	void createDescriptorPool();
	void createDescriptorSets();
	void createTextureSamplers();
	void createCommandPool();
	void createCommandBuffers();
	void createSyncObjects();

	std::pair<vk::raii::Buffer, vk::raii::DeviceMemory> createBuffer(
		vk::DeviceSize bufferSize, 
		vk::BufferUsageFlags usage, 
		vk::MemoryPropertyFlags properties
	);
	void copyBuffer(const vk::raii::Buffer& srcBuffer, const vk::raii::Buffer& dstBuffer, vk::DeviceSize size);
	std::pair<vk::raii::Image, vk::raii::DeviceMemory> createImage(
		uint32_t width, 
		uint32_t height, 
		vk::Format format, 
		vk::ImageUsageFlags usage, 
		vk::MemoryPropertyFlags properties
	);
	void beginRendering(vk::CommandBuffer currentCommandBuffer) noexcept;
	void draw(
		PipelineType pipelineType,
		const MeshBuffers& mesh,
		const InstanceBuffers& inst,
		const void* constants,
		uint32_t constantsSize
	) noexcept;
	void draw(
		PipelineType pipelineType,
		const MeshBuffers& mesh,
		const void* constants,
		uint32_t constantsSize
	) noexcept;

	void createInstanceBuffersInternal(
		vk::DeviceSize bufferSize, 
		std::array<vk::raii::Buffer, MAX_FRAMES_IN_FLIGHT>& pBuffers,
		std::array<vk::raii::DeviceMemory, MAX_FRAMES_IN_FLIGHT>& pBuffersMemory,
		std::array<void*, MAX_FRAMES_IN_FLIGHT>& pBuffersMapped
	);

	void updateGeometry(
		const void* vertices, 
		rust::Slice<const uint32_t> indices,
		MeshBuffers& mesh,
		vk::DeviceSize vertexBufferSize,
		vk::DeviceSize indexBufferSize
	);

	DataTexture createDataTexture(
		const void* data_ptr, 
		uint32_t width,
		uint32_t height,
		vk::Format format,
		uint32_t dstBinding
	);
	
	vk::raii::CommandBuffer beginSingleTimeCommands();
	void endSingleTimeCommands(vk::raii::CommandBuffer& commandBuffer);
	vk::Format findDepthFormat();
};

bool isDeviceSuitable(vk::PhysicalDevice physicalDevice, vk::SurfaceKHR surface);
QueueFamilyIndices findQueueFamilies(vk::PhysicalDevice physicalDevice, vk::SurfaceKHR surface);
SwapChainSupportDetails querySwapChainSupport(vk::PhysicalDevice physicalDevice, vk::SurfaceKHR surface);
bool checkDeviceExtensionSupport(vk::PhysicalDevice physicalDevice);
bool checkValidationLayerSupport();

std::unique_ptr<VulkanRenderer> createRenderer();
size_t getMaxSky();
size_t getAnimInfoSize();
