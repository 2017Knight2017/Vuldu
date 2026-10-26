#if defined(_WIN32)
	#ifndef NOMINMAX
		#define NOMINMAX
	#endif

	#ifndef WIN32_LEAN_AND_MEAN
		#define WIN32_LEAN_AND_MEAN
	#endif

	#include <windows.h>

	#define VK_USE_PLATFORM_WIN32_KHR
#elif defined(__APPLE__)
	#define VK_USE_PLATFORM_METAL_EXT
#elif defined(__linux__)
	#define VK_USE_PLATFORM_WAYLAND_KHR
	#define VK_USE_PLATFORM_XLIB_KHR
#endif

#include <stdexcept>
#include <cstring>
#include "renderer.h"
#include "renderer/src/bridge.rs.h"

bool checkValidationLayerSupport() {
	std::vector<vk::LayerProperties> availableLayers = vk::enumerateInstanceLayerProperties();

	for (const char* layerName : validationLayers) {
		bool layerFound = false;
		for (const auto& layerProperties : availableLayers) {
			if (strcmp(layerName, layerProperties.layerName) == 0) {
				layerFound = true;
				break;
			}
		}
		if (!layerFound) return false;
	}
	return true;
}

std::vector<const char*> getRequiredExtensions([[maybe_unused]] bool is_x11) {
	std::vector<const char*> extensions = { VK_KHR_SURFACE_EXTENSION_NAME };

	#if defined(_WIN32)
		extensions.push_back(VK_KHR_WIN32_SURFACE_EXTENSION_NAME);
	#elif defined(__APPLE__)
		extensions.push_back(VK_EXT_METAL_SURFACE_EXTENSION_NAME);
		extensions.push_back(VK_KHR_PORTABILITY_ENUMERATION_EXTENSION_NAME);
	#elif defined(__linux__)
		if (is_x11) 
			extensions.push_back(VK_KHR_XLIB_SURFACE_EXTENSION_NAME);
		else 
			extensions.push_back(VK_KHR_WAYLAND_SURFACE_EXTENSION_NAME);
	#endif

	return extensions;
}

void VulkanRenderer::createInstance(const WindowHandles& handles) {
	if (enableValidationLayers && !checkValidationLayerSupport()) {
		throw std::runtime_error("validation layers requested, but not available!");
	}

	vk::ApplicationInfo appInfo;
	appInfo.setApiVersion(VK_API_VERSION_1_3);

	std::vector<const char*> requiredExtensions = getRequiredExtensions(handles.is_x11);
	if (enableValidationLayers) {
		requiredExtensions.push_back(VK_EXT_DEBUG_UTILS_EXTENSION_NAME);
	}

	vk::InstanceCreateInfo createInfo;
	createInfo
		.setPApplicationInfo(&appInfo)
		.setPEnabledExtensionNames(requiredExtensions);
	
	#if defined(__APPLE__)
	createInfo.setFlags(vk::InstanceCreateFlagBits::eEnumeratePortabilityKHR);
	#endif
	
	if (enableValidationLayers) {
		createInfo.setPEnabledLayerNames(validationLayers);
	}

	this->instance = vk::raii::Instance(this->context, createInfo);
};

void VulkanRenderer::createSurface([[maybe_unused]] const WindowHandles& handles) {
	#if defined(_WIN32)
		this->surface = this->instance.createWin32SurfaceKHR({
			{}, 
			reinterpret_cast<HINSTANCE>(handles.display_ptr), 
			reinterpret_cast<HWND>(handles.window_ptr)
		});

	#elif defined(__APPLE__)
		this->surface = this->instance.createMetalSurfaceEXT({
			{}, 
			reinterpret_cast<const CAMetalLayer*>(handles.window_ptr)
		});

	#elif defined(__linux__)
		if (handles.is_x11) {
			this->surface = this->instance.createXlibSurfaceKHR({
				{}, 
				reinterpret_cast<Display*>(handles.display_ptr), 
				reinterpret_cast<Window>(handles.window_ptr)
			});
		} else {
			this->surface = this->instance.createWaylandSurfaceKHR({
				{}, 
				reinterpret_cast<struct wl_display*>(handles.display_ptr), 
				reinterpret_cast<struct wl_surface*>(handles.window_ptr)
			});
		}
	#else
		throw std::runtime_error("unsupported platform for window surface creation!");
	#endif
};
