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
    createDescriptorSetLayout();
    createGraphicsPipeline();
    createUniformBuffers();
    createDescriptorPool();
    createDescriptorSets(); 
    createCommandPool();
    createDepthResources();
    createFramebuffers();
    createTextureSampler();
    uint8_t dummyPixel[] = { 255, 0, 255, 255 };
    addTexture(dummyPixel, 1, 1);
    createCommandBuffers();
    createSyncObjects();
}

void VulkanRenderer::recreateSwapChain() {
    vkDeviceWaitIdle(this->device);

    cleanupSwapChain();

    createSwapChain();
    createImageViews();
    createDepthResources();
    createFramebuffers();

    createSyncObjects(); 

    this->currentFrame = 0;
}

void VulkanRenderer::cleanupSwapChain() {
    destroyResource(this->device, this->depthImageView, vkDestroyImageView);
    destroyResource(this->device, this->depthImage, vkDestroyImage);
    destroyResource(this->device, this->depthImageMemory, vkFreeMemory);

    for (auto framebuffer : this->swapChainFramebuffers) {
        vkDestroyFramebuffer(this->device, framebuffer, nullptr);
    }
    this->swapChainFramebuffers.clear();

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

    for (size_t i = 0; i < this->textureImages.size(); i++) {
        vkDestroyImageView(this->device, this->textureImageViews[i], nullptr);
        vkDestroyImage(this->device, this->textureImages[i], nullptr);
        vkFreeMemory(this->device, this->textureImageMemories[i], nullptr);
    }
    this->textureImageViews.clear();
    this->textureImages.clear();
    this->textureImageMemories.clear();

    for (size_t i = 0; i < MAX_FRAMES_IN_FLIGHT; i++) {
        vkDestroyBuffer(this->device, this->uniformBuffers[i], nullptr);
        vkFreeMemory(this->device, this->uniformBuffersMemory[i], nullptr);
    }
    this->uniformBuffers.clear();
    this->uniformBuffersMemory.clear();

    destroyResource(this->device, this->descriptorPool, vkDestroyDescriptorPool);
    destroyResource(this->device, this->descriptorSetLayout, vkDestroyDescriptorSetLayout);
    destroyResource(this->device, this->paletteBuffer, vkDestroyBuffer);
    destroyResource(this->device, this->paletteBufferMemory, vkFreeMemory);
    destroyResource(this->device, this->indexBuffer, vkDestroyBuffer);
    destroyResource(this->device, this->indexBufferMemory, vkFreeMemory);
    destroyResource(this->device, this->vertexBuffer, vkDestroyBuffer);
    destroyResource(this->device, this->vertexBufferMemory, vkFreeMemory);
    destroyResource(this->device, this->levelPipeline, vkDestroyPipeline);
    destroyResource(this->device, this->levelPipelineLayout, vkDestroyPipelineLayout);
    destroyResource(this->device, this->spritePipeline, vkDestroyPipeline);
    destroyResource(this->device, this->spritePipelineLayout, vkDestroyPipelineLayout);
    destroyResource(this->device, this->renderPass, vkDestroyRenderPass);

    for (size_t i = 0; i < MAX_FRAMES_IN_FLIGHT; i++) {
        vkDestroyFence(this->device, this->inFlightFences[i], nullptr);
    }
    this->inFlightFences.clear();
    
    for (size_t i = 0; i < this->swapChainImages.size(); i++) {
        vkDestroySemaphore(this->device, this->renderFinishedSemaphores[i], nullptr);
        vkDestroySemaphore(this->device, this->imageAvailableSemaphores[i], nullptr);
    }
    this->renderFinishedSemaphores.clear();
    this->imageAvailableSemaphores.clear();

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
