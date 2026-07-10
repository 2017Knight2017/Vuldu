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
    createInstanceBuffers();
    createDescriptorPool();
    createDescriptorSets(); 
    createCommandPool();
    createDepthResources();
    createFramebuffers();
    createTextureSamplers();
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


    for (size_t i = 0; i < this->textureImageViews.size(); i++) {
        vkDestroyImageView(this->device, this->textureImageViews[i], nullptr);
    }
    this->textureImageViews.clear();

    for (size_t i = 0; i < this->textureImages.size(); i++) {
        vkDestroyImage(this->device, this->textureImages[i], nullptr);
    }
    this->textureImages.clear();

    for (size_t i = 0; i < this->textureImageMemories.size(); i++) {
        vkFreeMemory(this->device, this->textureImageMemories[i], nullptr);
    }
    this->textureImageMemories.clear();

    for (size_t i = 0; i < this->skyWidths.size(); i++) {
        this->skyWidths[i] = 0.0;
    }
    this->skyWidths.clear();

    for (size_t i = 0; i < MAX_FRAMES_IN_FLIGHT; i++) {
        if (this->uniformBuffersMapped[i] != nullptr) {
            vkUnmapMemory(this->device, this->uniformBuffersMemory[i]);
        }
        if (this->instanceBuffersMapped[i] != nullptr) {
            vkUnmapMemory(this->device, this->instanceBuffersMemory[i]);
        }
        vkDestroyBuffer(this->device, this->uniformBuffers[i], nullptr);
        vkFreeMemory(this->device, this->uniformBuffersMemory[i], nullptr);
        vkDestroyBuffer(this->device, this->instanceBuffers[i], nullptr);
        vkFreeMemory(this->device, this->instanceBuffersMemory[i], nullptr);
    }
    this->uniformBuffers.clear();
    this->uniformBuffersMemory.clear();
    this->uniformBuffersMapped.clear();
    this->instanceBuffers.clear();
    this->instanceBuffersMemory.clear();
    this->instanceBuffersMapped.clear();

    destroyResource(this->device, this->descriptorPool, vkDestroyDescriptorPool);
    destroyResource(this->device, this->descriptorSetLayout, vkDestroyDescriptorSetLayout);
    destroyResource(this->device, this->paletteBuffer, vkDestroyBuffer);
    destroyResource(this->device, this->paletteBufferMemory, vkFreeMemory);
    destroyResource(this->device, this->colormapBuffer, vkDestroyBuffer);
    destroyResource(this->device, this->colormapBufferMemory, vkFreeMemory);
    destroyResource(this->device, this->levelIndexBuffer, vkDestroyBuffer);
    destroyResource(this->device, this->levelIndexBufferMemory, vkFreeMemory);
    destroyResource(this->device, this->levelVertexBuffer, vkDestroyBuffer);
    destroyResource(this->device, this->levelVertexBufferMemory, vkFreeMemory);
    destroyResource(this->device, this->objectIndexBuffer, vkDestroyBuffer);
    destroyResource(this->device, this->objectIndexBufferMemory, vkFreeMemory);
    destroyResource(this->device, this->objectVertexBuffer, vkDestroyBuffer);
    destroyResource(this->device, this->objectVertexBufferMemory, vkFreeMemory);
    destroyResource(this->device, this->levelPipeline, vkDestroyPipeline);
    destroyResource(this->device, this->levelPipelineLayout, vkDestroyPipelineLayout);
    destroyResource(this->device, this->spritePipeline, vkDestroyPipeline);
    destroyResource(this->device, this->spritePipelineLayout, vkDestroyPipelineLayout);
    destroyResource(this->device, this->renderPass, vkDestroyRenderPass);

    for (size_t i = 0; i < MAX_FRAMES_IN_FLIGHT; i++) {
        vkDestroyFence(this->device, this->inFlightFences[i], nullptr);
    }
    this->inFlightFences.clear();
    
    for (size_t i = 0; i < this->renderFinishedSemaphores.size(); i++) {
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
