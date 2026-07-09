#version 450
#extension GL_EXT_nonuniform_qualifier : enable
#extension GL_EXT_shader_8bit_storage : require

layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
} ubo;

layout(binding = 1) readonly buffer PaletteBuffer {
    vec4 colors[3584]; 
} pal;

layout(binding = 2) readonly buffer ColormapBuffer {
    uint8_t colors[8448];
} colormap;

layout(binding = 3) uniform sampler2D skySamplers[];

layout(binding = 4) uniform sampler2D texSamplers[];

layout(push_constant) uniform LevelConstants {
    uint paletteIndex;
    float resolution[2];
    uint skyIndex;
    float skyWidth;
} lc;

layout(location = 0) in float fragLightLevel;      
layout(location = 1) in vec2 fragTexCoord;
layout(location = 2) flat in uint fragTexId;
layout(location = 3) flat in uint fragColormapIdx;
layout(location = 4) flat in uint fragFloorTexId;
layout(location = 5) in float fragViewZ;
//layout(location = 6) in vec3 fragBarycentric;
//layout(location = 7) in vec3 fragTriangleColor;

layout(location = 0) out vec4 outColor;

const float PI = 3.14159265359;

void main() {
    uint colorIndex = 0;

    vec2 res = vec2(lc.resolution[0], lc.resolution[1]);
    vec2 screenUV = gl_FragCoord.xy / res; 

    // untextured walls
    if (fragTexId == 65535) {
        float scale = fragViewZ * 0.04;

        vec2 floorUV = screenUV * scale; 

        float rawColor = textureLod(texSamplers[nonuniformEXT(fragFloorTexId)], floorUV, 0.0).r;
        colorIndex = uint(rawColor * 255.0 + 0.5);
    
    // sky
    } else if (fragTexId == 65534) {
        vec3 forward = normalize(vec3(ubo.proj[0][2], ubo.proj[1][2], ubo.proj[2][2]));
        float cameraYaw = atan(forward.z, forward.x);

        float widthFactor = 1024.0 / lc.skyWidth;

        float skyU = (cameraYaw + PI) / (2.0 * PI) * widthFactor;
        skyU += screenUV.x * (0.4 * widthFactor);
        skyU = fract(skyU);

        float skyV = screenUV.y * 1.6;

        float rawColor = textureLod(skySamplers[nonuniformEXT(lc.skyIndex)], vec2(skyU, skyV), 0.0).r;
        colorIndex = uint(rawColor * 255.0 + 0.5); 
    } else {
        float rawColor = textureLod(texSamplers[nonuniformEXT(fragTexId)], fragTexCoord, 0.0).r;
        colorIndex = uint(rawColor * 255.0 + 0.5);
    }

    if (colorIndex == 255) {
        discard;
    }

    /// COLORMAP shadows
    uint finalColormapIdx = (fragTexId == 65534) ? 0 : fragColormapIdx;
    uint colormapOffset = (finalColormapIdx * 256) | colorIndex;
    uint shadedIndex = uint(colormap.colors[colormapOffset]);
    
    uint colorPosition = (lc.paletteIndex * 256) | shadedIndex;
    vec4 colormapColor = pal.colors[colorPosition];
    
    outColor = vec4(colormapColor.rgb, 1.0);

    /// 256-unit shadows
    //vec3 modernColor = pal.colors[(lc.paletteIndex * 256) | colorIndex].rgb * fragLightLevel;
    //outColor = vec4(modernColor.rgb, 1.0);  

    /// WIREMAP
    //vec3 d = fwidth(fragBarycentric);
    //vec3 thickness = d * 1.5;
    //vec3 edge = smoothstep(vec3(0.0), thickness, fragBarycentric);
    //float minEdge = min(min(edge.x, edge.y), edge.z);
    //float isEdge = 1.0 - minEdge;
    //outColor = vec4(mix(vec3(1.0), fragTriangleColor, isEdge), 1.0);
}
