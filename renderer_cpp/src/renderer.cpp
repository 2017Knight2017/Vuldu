#include <iostream>
#include "renderer.h"

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
    createFramebuffers();
    createUniformBuffers();
    createDescriptorPool();
    createDescriptorSets(); 
    createCommandPool();
    createCommandBuffers();
    createIndexBuffer();
    createSyncObjects();
}

void VulkanRenderer::recreateSwapChain() {
    vkDeviceWaitIdle(this->device);

    cleanupSwapChain();

    createSwapChain();
    createImageViews();
    createFramebuffers();

    createSyncObjects(); 

    this->currentFrame = 0;
}

void VulkanRenderer::cleanupSwapChain() {
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

    for (size_t i = 0; i < MAX_FRAMES_IN_FLIGHT; i++) {
        vkDestroyBuffer(this->device, this->uniformBuffers[i], nullptr);
        vkFreeMemory(this->device, this->uniformBuffersMemory[i], nullptr);
    }
    this->uniformBuffers.clear();
    this->uniformBuffersMemory.clear();

    if (this->descriptorPool != VK_NULL_HANDLE) {
        vkDestroyDescriptorPool(this->device, this->descriptorPool, nullptr);
        this->descriptorPool = VK_NULL_HANDLE;
    }

    if (this->descriptorSetLayout != VK_NULL_HANDLE) {
        vkDestroyDescriptorSetLayout(this->device, this->descriptorSetLayout, nullptr);
        this->descriptorSetLayout = VK_NULL_HANDLE;
    }

    if (this->indexBuffer != VK_NULL_HANDLE) {
        vkDestroyBuffer(this->device, this->indexBuffer, nullptr);
        this->indexBuffer = VK_NULL_HANDLE;
    }

    if (this->indexBufferMemory != VK_NULL_HANDLE) {
        vkFreeMemory(this->device, this->indexBufferMemory, nullptr);
        this->indexBufferMemory = VK_NULL_HANDLE;
    }

    if (this->vertexBuffer != VK_NULL_HANDLE) {
        vkDestroyBuffer(this->device, this->vertexBuffer, nullptr);
        this->vertexBuffer = VK_NULL_HANDLE;
    }

    if (this->vertexBufferMemory != VK_NULL_HANDLE) {
        vkFreeMemory(this->device, this->vertexBufferMemory, nullptr);
        this->vertexBufferMemory = VK_NULL_HANDLE;
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

    if (this->commandPool != VK_NULL_HANDLE) {
        vkDestroyCommandPool(this->device, this->commandPool, nullptr);
        this->commandPool = VK_NULL_HANDLE;
    }

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
