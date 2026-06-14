#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
} ubo;

layout(location = 0) out vec2 fragTexCoord;
layout(location = 1) flat out uint fragTexId;

layout(push_constant) uniform SpriteConstants {
    uint paletteIndex;      // offset = 0
    uint lightLevel;        // offset = 4
    uint textureId;         // offset = 8
    float spriteWidth;     // offset = 12
    float spriteHeight;    // offset = 16
    float leftOffset;      // offset = 20
    float topOffset;       // offset = 24
    float padding;         // offset = 28
    vec4 spriteWorldPos;   // offset = 32
} sc;

const vec2 positions[6] = vec2[](
    vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(1.0, 1.0),
    vec2(1.0, 1.0), vec2(0.0, 1.0), vec2(0.0, 0.0)
);

void main() {
    vec2 inPosition = positions[gl_VertexIndex];
    fragTexCoord = inPosition;
    fragTexId = sc.textureId;
    
    float scale = 1.0 / 64.0; 
    
    float xOffset = (inPosition.x * sc.spriteWidth) - sc.leftOffset;
    float yOffset = ((1.0 - inPosition.y) * sc.spriteHeight) - sc.topOffset;

    xOffset *= scale;
    yOffset *= scale;

    vec4 cameraSpacePos = ubo.view * vec4(sc.spriteWorldPos.xyz, 1.0);

    cameraSpacePos.x += xOffset;
    cameraSpacePos.y += yOffset; 

    gl_Position = ubo.proj * cameraSpacePos;
}