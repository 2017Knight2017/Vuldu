#include <string>
#include <stdexcept>
#include <set>
#include "renderer.h"

QueueFamilyIndices findQueueFamilies(vk::PhysicalDevice physicalDevice, vk::SurfaceKHR surface) {
	QueueFamilyIndices indices;

	std::vector<vk::QueueFamilyProperties> queueFamilies = physicalDevice.getQueueFamilyProperties();

	for (size_t i = 0; i < queueFamilies.size(); ++i) {
		if (queueFamilies[i].queueFlags & vk::QueueFlagBits::eGraphics) {
			indices.graphicsFamily = i;
		}

		vk::Bool32 presentSupport = physicalDevice.getSurfaceSupportKHR(i, surface);

		if (presentSupport) {
			indices.presentFamily = i;
		}

		if (indices.isComplete()) {
			break;
		}
	}

	return indices;
}

SwapChainSupportDetails querySwapChainSupport(vk::PhysicalDevice physicalDevice, vk::SurfaceKHR surface) {
	SwapChainSupportDetails details;
	details.capabilities = physicalDevice.getSurfaceCapabilitiesKHR(surface);


	std::vector<vk::SurfaceFormatKHR> formats = physicalDevice.getSurfaceFormatsKHR(surface);
	if (formats.size() != 0) {
		details.formats = formats;
	}

	std::vector<vk::PresentModeKHR> presentModes = physicalDevice.getSurfacePresentModesKHR(surface);
	if (presentModes.size() != 0) {
		details.presentModes = presentModes;
	}

	return details;
}

bool checkDeviceExtensionSupport(vk::PhysicalDevice physicalDevice) {
	std::vector<vk::ExtensionProperties> extensions =  physicalDevice.enumerateDeviceExtensionProperties();

	std::set<std::string> requiredExtensions(deviceExtensions.begin(), deviceExtensions.end());

	for (const auto& extension : extensions) {
		requiredExtensions.erase(extension.extensionName);
	}

	return requiredExtensions.empty();
}

bool isDeviceSuitable(vk::PhysicalDevice physicalDevice, vk::SurfaceKHR surface) {
	QueueFamilyIndices indices = findQueueFamilies(physicalDevice, surface);

	bool extensionsSupported = checkDeviceExtensionSupport(physicalDevice);

	bool swapchainAdequate = false;
	if (extensionsSupported) {
		SwapChainSupportDetails swapchainSupport = querySwapChainSupport(physicalDevice, surface);
		swapchainAdequate = !swapchainSupport.formats.empty() && !swapchainSupport.presentModes.empty();
	}

	vk::PhysicalDeviceFeatures supportedFeatures = physicalDevice.getFeatures();

	return indices.isComplete() && extensionsSupported && swapchainAdequate && supportedFeatures.samplerAnisotropy;
}

void VulkanRenderer::pickPhysicalDevice() {
	std::vector<vk::raii::PhysicalDevice> devices = this->instance.enumeratePhysicalDevices();
	if (devices.size() == 0) {
		throw std::runtime_error("failed to find GPUs with Vulkan support!");
	}
	
	for (const auto& device : devices) {
		if (isDeviceSuitable(device, this->surface)) {
			this->physicalDevice = device;
			break;
		}
	}

	if (this->physicalDevice == VK_NULL_HANDLE) {
		throw std::runtime_error("failed to find a suitable GPU!");
	}
};

void VulkanRenderer::createLogicalDevice() {
	QueueFamilyIndices indices = findQueueFamilies(this->physicalDevice, this->surface);

	std::vector<vk::DeviceQueueCreateInfo> queueCreateInfos;
	std::set<uint32_t> uniqueQueueFamilies = {indices.graphicsFamily.value(), indices.presentFamily.value()};

	float queuePriority = 1.0f;
	for (const uint32_t queueFamily : uniqueQueueFamilies) {
		vk::DeviceQueueCreateInfo queueCreateInfo{
			{},
			queueFamily,
			1,
			&queuePriority
		};
		queueCreateInfos.push_back(queueCreateInfo);
	}

	vk::PhysicalDeviceVulkan13Features vulkan13Features;
	vulkan13Features
		.setSynchronization2(VK_TRUE)
		.setDynamicRendering(VK_TRUE);

	vk::PhysicalDeviceVulkan12Features vulkan12Features;
	vulkan12Features
		.setPNext(vulkan13Features)
		.setDescriptorBindingVariableDescriptorCount(VK_TRUE)
		.setDescriptorBindingSampledImageUpdateAfterBind(VK_TRUE)
		.setDescriptorBindingPartiallyBound(VK_TRUE)
		.setRuntimeDescriptorArray(VK_TRUE)
		.setShaderSampledImageArrayNonUniformIndexing(VK_TRUE);

	vk::PhysicalDeviceFeatures2 deviceFeatures2;
	deviceFeatures2.setPNext(vulkan12Features);
	deviceFeatures2.features.setSamplerAnisotropy(VK_TRUE);

	vk::DeviceCreateInfo createInfo;
	createInfo
		.setPNext(deviceFeatures2)
		.setQueueCreateInfos(queueCreateInfos)
		.setPEnabledExtensionNames(deviceExtensions);

	this->device = this->physicalDevice.createDevice(createInfo);

	this->graphicsQueue = this->device.getQueue(indices.graphicsFamily.value(), 0);
	this->presentQueue = this->device.getQueue(indices.presentFamily.value(), 0);
};