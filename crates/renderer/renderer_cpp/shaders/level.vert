#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
} ubo;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inLightLevel;
layout(location = 2) in vec2 inTexCoord;
layout(location = 3) in int inTexId;
layout(location = 4) in int inSectorId;
layout(location = 5) in int inColormapIdx;

layout(location = 0) out vec3 fragLightLevel;      
layout(location = 1) out vec2 fragTexCoord;
layout(location = 2) flat out uint fragTexId;
layout(location = 3) flat out uint fragSectorId;
layout(location = 4) flat out uint fragColormapIdx;

void main() {
    fragLightLevel = inLightLevel;
    fragTexCoord = inTexCoord;
    fragTexId = inTexId;
    fragSectorId = inSectorId;
    fragColormapIdx = inColormapIdx;
    
    gl_Position = ubo.proj * ubo.view * ubo.model * vec4(inPosition, 1.0);
}