#include <iostream>
#include "renderer.h"

std::unique_ptr<VulkanRenderer> create_renderer() {
    return std::make_unique<VulkanRenderer>();
}

VulkanRenderer::VulkanRenderer() {}

VulkanRenderer::~VulkanRenderer() {
    cleanup();
}

void VulkanRenderer::initVulkan(const WindowHandles& handles, size_t window_raw_ptr) {
    this->window_raw_ptr = window_raw_ptr;
    createInstance();
    setupDebugMessenger();
    createSurface(handles);
    pickPhysicalDevice();
    createLogicalDevice();
    createSwapChain();
    createImageViews();
    createRenderPass();
    createGraphicsPipeline();
}

void VulkanRenderer::cleanup() {
    if (this->instance == VK_NULL_HANDLE) {
        return;
    }

    std::cout << "[Cleanup] Starting VulkanRenderer destruction..." << std::endl;

    if (this->device != VK_NULL_HANDLE) {
        vkDeviceWaitIdle(this->device);
    }

    if (this->graphicsPipeline != VK_NULL_HANDLE) {
        vkDestroyPipeline(this->device, this->graphicsPipeline, nullptr);
        this->graphicsPipeline = VK_NULL_HANDLE; 
    }

    if (this->pipelineLayout != VK_NULL_HANDLE) {
        vkDestroyPipelineLayout(this->device, this->pipelineLayout, nullptr);
        this->pipelineLayout = VK_NULL_HANDLE; 
    }

    if (this->renderPass != VK_NULL_HANDLE) {
        vkDestroyRenderPass(this->device, this->renderPass, nullptr);
        this->renderPass = VK_NULL_HANDLE; 
    }

    for (auto imageView : this->swapChainImageViews) {
        if (imageView != VK_NULL_HANDLE) {
            vkDestroyImageView(this->device, imageView, nullptr);
        }
    }
    this->swapChainImageViews.clear();

    if (this->swapChain != VK_NULL_HANDLE) {
        vkDestroySwapchainKHR(this->device, this->swapChain, nullptr);
        this->swapChain = VK_NULL_HANDLE;
    }

    if (this->device != VK_NULL_HANDLE) {
        vkDestroyDevice(this->device, nullptr);
        this->device = VK_NULL_HANDLE; 
    }

    if (this->surface != VK_NULL_HANDLE) {
        vkDestroySurfaceKHR(this->instance, this->surface, nullptr);
        this->surface = VK_NULL_HANDLE;
    }

    if (enableValidationLayers && this->debugMessenger != VK_NULL_HANDLE) {
        DestroyDebugUtilsMessengerEXT(this->instance, this->debugMessenger, nullptr);
        this->debugMessenger = VK_NULL_HANDLE;
    }

    if (this->instance != VK_NULL_HANDLE) {
        vkDestroyInstance(this->instance, nullptr);
        this->instance = VK_NULL_HANDLE;
    }

    std::cout << "[Cleanup] VulkanRenderer successfully destroyed!" << std::endl;
}
