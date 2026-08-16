#version 450

layout(push_constant) uniform UiConstants {
    uint paletteIndex;
    float resolution[2];
} uc;

layout(location = 0) in vec3 inVertexPos; 
layout(location = 1) in vec2 inTexCoord; 
layout(location = 2) in vec2 inInstancePos; 
layout(location = 3) in vec2 inInstanceSize; 
layout(location = 4) in uint inInstanceTexId; 

layout(location = 0) out vec2 fragTexCoord;
layout(location = 1) flat out uint fragTexId;

void main() {
    fragTexCoord = inTexCoord;
    fragTexId = inInstanceTexId;

    vec2 pixelPos = (inVertexPos.xy * inInstanceSize) + inInstancePos;

    vec2 res = vec2(uc.resolution[0], uc.resolution[1]);
    vec2 ndcPos = (pixelPos / res) * 2.0 - 1.0;

    gl_Position = vec4(ndcPos, 0.0, 1.0);
}
