#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
} ubo;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec2 inTexCoord;
layout(location = 2) in float inLightLevel;
layout(location = 3) in uint inTexId;
layout(location = 4) in uint inColormapIdx;
layout(location = 5) in uint inFloorTexId;

layout(location = 0) out float fragLightLevel;      
layout(location = 1) out vec2 fragTexCoord;
layout(location = 2) flat out uint fragTexId;
layout(location = 3) flat out uint fragColormapIdx;
layout(location = 4) flat out uint fragFloorTexId;
layout(location = 5) out float fragViewZ;
//layout(location = 6) out vec3 fragBarycentric;
//layout(location = 7) out vec3 fragTriangleColor;

vec3 hashColor(int id) {
    float r = fract(sin(float(id) * 12.9898) * 43758.5453);
    float g = fract(sin(float(id) * 78.233) * 43758.5453);
    float b = fract(sin(float(id) * 45.164) * 43758.5453);
    return vec3(r, g, b);
}

void main() {
    fragLightLevel = inLightLevel;
    fragTexCoord = inTexCoord;
    fragTexId = inTexId;
    fragColormapIdx = inColormapIdx;
    fragFloorTexId = inFloorTexId;

    vec4 viewPos = ubo.view * ubo.model * vec4(inPosition, 1.0);

    fragViewZ = abs(viewPos.z);

    gl_Position = ubo.proj * viewPos;

    //if (inTexId == 65534) {
    //    gl_Position.z = gl_Position.w;
    //}

    /// WIREMAP
    //int localIndex = gl_VertexIndex % 3;
    //
    //if (localIndex == 0)      fragBarycentric = vec3(1.0, 0.0, 0.0);
    //else if (localIndex == 1) fragBarycentric = vec3(0.0, 1.0, 0.0);
    //else                      fragBarycentric = vec3(0.0, 0.0, 1.0);
    //
    //int triangleID = gl_VertexIndex / 3;
    //fragTriangleColor = hashColor(triangleID);
}
