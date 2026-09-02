#include <iostream>
#include "renderer.h"
#include "utils.h"

std::unique_ptr<VulkanRenderer> createRenderer() {
    return std::make_unique<VulkanRenderer>();
}

VulkanRenderer::VulkanRenderer() {}

VulkanRenderer::~VulkanRenderer() {
    cleanup();
}

void VulkanRenderer::initVulkan(const WindowHandles& handles, uint32_t width, uint32_t height) {
    createInstance(handles);
    setupDebugMessenger();
    createSurface(handles);
    pickPhysicalDevice();
    createLogicalDevice();
    createSwapChain(width, height);
    createImageViews();
    createDescriptorSetLayout();
    createPipelines();
    createMVPBuffer();
    createObjectInstanceBuffers();
    createUiInstanceBuffers();
    createDescriptorPool();
    createDescriptorSets(); 
    createCommandPool();
    createDepthResources();
    createTextureSamplers();
    createCommandBuffers();
    createSyncObjects();
}

void VulkanRenderer::recreateSwapChain(uint32_t width, uint32_t height) {
    vkDeviceWaitIdle(this->device);

    cleanupSwapChain();

    createSwapChain(width, height);
    createImageViews();
    createDepthResources();

    this->currentFrame = 0;
}

void VulkanRenderer::cleanupSwapChain() {
    destroyResource(this->device, this->depthImageView, vkDestroyImageView);
    destroyResource(this->device, this->depthImage, vkDestroyImage);
    destroyResource(this->device, this->depthImageMemory, vkFreeMemory);

    for (auto imageView : this->swapChainImageViews) {
        vkDestroyImageView(this->device, imageView, nullptr);
    }
    this->swapChainImageViews.clear();

    vkDestroySwapchainKHR(this->device, this->swapChain, nullptr);
    this->swapChain = VK_NULL_HANDLE;
}

void VulkanRenderer::cleanup() {
    if (this->instance == VK_NULL_HANDLE) {
        return;
    }

    if (this->device != VK_NULL_HANDLE) {
        vkDeviceWaitIdle(this->device);
    }
    std::cout << "[Cleanup] Starting VulkanRenderer destruction..." << std::endl;

    cleanupSwapChain();

    destroyResource(this->device, this->textureSampler, vkDestroySampler);

    for (size_t i = 0; i < this->textureImageViews.size(); i++) {
        vkDestroyImageView(this->device, this->textureImageViews[i], nullptr);
        vkDestroyImage(this->device, this->textureImages[i], nullptr);
        vkFreeMemory(this->device, this->textureImageMemories[i], nullptr);
    }

    for (size_t i = 0; i < MAX_FRAMES_IN_FLIGHT; i++) {
        vkDestroyBuffer(this->device, this->MVPBuffers[i], nullptr);
        vkFreeMemory(this->device, this->MVPBuffersMemory[i], nullptr);
        vkDestroyBuffer(this->device, this->objectInstanceBuffers[i], nullptr);
        vkFreeMemory(this->device, this->objectInstanceBuffersMemory[i], nullptr);
        vkDestroyBuffer(this->device, this->uiInstanceBuffers[i], nullptr);
        vkFreeMemory(this->device, this->uiInstanceBuffersMemory[i], nullptr);
    }

    destroyResource(this->device, this->descriptorPool, vkDestroyDescriptorPool);
    destroyResource(this->device, this->descriptorSetLayout, vkDestroyDescriptorSetLayout);
    destroyResource(this->device, this->animLevelBuffer, vkDestroyBuffer);
    destroyResource(this->device, this->animLevelBufferMemory, vkFreeMemory);
    destroyResource(this->device, this->sectorHeightsBuffer, vkDestroyBuffer);
    destroyResource(this->device, this->sectorHeightsBufferMemory, vkFreeMemory);
    destroyResource(this->device, this->paletteImageView, vkDestroyImageView);
    destroyResource(this->device, this->paletteImage, vkDestroyImage);
    destroyResource(this->device, this->paletteImageMemory, vkFreeMemory);
    destroyResource(this->device, this->colormapImageView, vkDestroyImageView);
    destroyResource(this->device, this->colormapImage, vkDestroyImage);
    destroyResource(this->device, this->colormapImageMemory, vkFreeMemory);
    destroyResource(this->device, this->levelIndexBuffer, vkDestroyBuffer);
    destroyResource(this->device, this->levelIndexBufferMemory, vkFreeMemory);
    destroyResource(this->device, this->levelVertexBuffer, vkDestroyBuffer);
    destroyResource(this->device, this->levelVertexBufferMemory, vkFreeMemory);
    destroyResource(this->device, this->objectIndexBuffer, vkDestroyBuffer);
    destroyResource(this->device, this->objectIndexBufferMemory, vkFreeMemory);
    destroyResource(this->device, this->objectVertexBuffer, vkDestroyBuffer);
    destroyResource(this->device, this->objectVertexBufferMemory, vkFreeMemory);
    destroyResource(this->device, this->uiIndexBuffer, vkDestroyBuffer);
    destroyResource(this->device, this->uiIndexBufferMemory, vkFreeMemory);
    destroyResource(this->device, this->uiVertexBuffer, vkDestroyBuffer);
    destroyResource(this->device, this->uiVertexBufferMemory, vkFreeMemory);
    destroyResource(this->device, this->levelPipeline, vkDestroyPipeline);
    destroyResource(this->device, this->levelPipelineLayout, vkDestroyPipelineLayout);
    destroyResource(this->device, this->spritePipeline, vkDestroyPipeline);
    destroyResource(this->device, this->spritePipelineLayout, vkDestroyPipelineLayout);
    destroyResource(this->device, this->uiPipeline, vkDestroyPipeline);
    destroyResource(this->device, this->uiPipelineLayout, vkDestroyPipelineLayout);

    for (size_t i = 0; i < MAX_FRAMES_IN_FLIGHT; i++) {
        vkDestroyFence(this->device, this->inFlightFences[i], nullptr);
    }
    
    for (size_t i = 0; i < this->renderFinishedSemaphores.size(); i++) {
        vkDestroySemaphore(this->device, this->renderFinishedSemaphores[i], nullptr);
        vkDestroySemaphore(this->device, this->imageAvailableSemaphores[i], nullptr);
    }

    destroyResource(this->device, this->commandPool, vkDestroyCommandPool);

    if (this->device != VK_NULL_HANDLE) {
        vkDestroyDevice(this->device, nullptr);
        this->device = VK_NULL_HANDLE; 
    }

    if (enableValidationLayers && this->debugMessenger != VK_NULL_HANDLE) {
        DestroyDebugUtilsMessengerEXT(this->instance, this->debugMessenger, nullptr);
        this->debugMessenger = VK_NULL_HANDLE;
    }

    if (this->surface != VK_NULL_HANDLE) {
        vkDestroySurfaceKHR(this->instance, this->surface, nullptr);
        this->surface = VK_NULL_HANDLE;
    }

    if (this->instance != VK_NULL_HANDLE) {
        vkDestroyInstance(this->instance, nullptr);
        this->instance = VK_NULL_HANDLE;
    }

    std::cout << "[Cleanup] VulkanRenderer successfully destroyed!" << std::endl;
}