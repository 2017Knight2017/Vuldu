#include <iostream>
#include "renderer.h"

static VKAPI_ATTR vk::Bool32 VKAPI_CALL debugCallback(
	vk::DebugUtilsMessageSeverityFlagBitsEXT messageSeverity, 
	vk::DebugUtilsMessageTypeFlagsEXT messageType, 
	const vk::DebugUtilsMessengerCallbackDataEXT* pCallbackData, 
	void* pUserData
) {
	(void)messageSeverity;
	(void)messageType;
	(void)pUserData;
	
	std::cerr << "-- validation layer: " << pCallbackData->pMessage << std::endl;

	return VK_FALSE;
}

using MsgBit = vk::DebugUtilsMessageTypeFlagBitsEXT;
using SvrtBit = vk::DebugUtilsMessageSeverityFlagBitsEXT;

void VulkanRenderer::setupDebugMessenger() {
	if (!enableValidationLayers) return;

	vk::DebugUtilsMessengerCreateInfoEXT createInfo{
		{},
		SvrtBit::eVerbose | SvrtBit::eWarning | SvrtBit::eError,
		MsgBit::eGeneral | MsgBit::eValidation | MsgBit::ePerformance,
		debugCallback
	};

	this->debugMessenger = this->instance.createDebugUtilsMessengerEXT(createInfo);
}