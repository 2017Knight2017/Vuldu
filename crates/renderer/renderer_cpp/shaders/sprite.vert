#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
} ubo;

layout(location = 0) in vec3 inVertexPos;
layout(location = 1) in vec2 inTexCoord;
layout(location = 2) in vec3 inInstancePos;
layout(location = 3) in vec2 inInstanceOffset;
layout(location = 4) in vec2 inInstanceSize;
layout(location = 5) in float inInstanceLight;
layout(location = 6) in uint inInstanceTexId;
layout(location = 7) in uint inInstanceColormapIdx;

layout(location = 0) out vec2 fragTexCoord;
layout(location = 1) flat out uint fragTexId;
layout(location = 2) out float fragLightLevel;
layout(location = 3) flat out uint fragColormapIdx;

void main() {
    if (inInstanceSize.x < 0.0) {
        fragTexCoord = vec2(1.0 - inTexCoord.x, inTexCoord.y);
    } else {
        fragTexCoord = inTexCoord;
    }
    fragTexId = inInstanceTexId;
    fragLightLevel = inInstanceLight;
    fragColormapIdx = inInstanceColormapIdx;

    float vertexX = inVertexPos.x * abs(inInstanceSize.x);
    float vertexY = inVertexPos.y * inInstanceSize.y;

    vec4 cameraSpacePos = ubo.view * vec4(inInstancePos, 1.0);

    cameraSpacePos.x += vertexX - inInstanceOffset.x;
    cameraSpacePos.y += vertexY + (inInstanceOffset.y - inInstanceSize.y);

    gl_Position = ubo.proj * cameraSpacePos;
}
