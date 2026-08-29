#version 450
#extension GL_EXT_nonuniform_qualifier : enable

layout(binding = 1) uniform sampler2D palTex;
layout(binding = 2) uniform usampler2D colormapTex;

layout(binding = 4) uniform sampler2D texSamplers[];

layout(push_constant) uniform LevelConstants {
    vec2 resolution;
    uint paletteIndex;
    uint skyIndex;
    float widthFactor;
    float globalTimer;
    float cameraYaw;
    uint flags; 
} lc;

layout(location = 0) flat in uint fragLightLevel;      
layout(location = 1) in vec2 fragTexCoord;
layout(location = 2) flat in uint fragTexId;
layout(location = 3) flat in uint fragFloorTexId;
layout(location = 4) in float fragViewZ;
layout(location = 5) in float fragScrollDir;
layout(location = 6) in vec3 fragBarycentric;
layout(location = 7) in vec3 fragTriangleColor;

layout(location = 0) out vec4 outColor;

const float TAU = 6.2831853071; 

const uint WIREMAP = 1;
const uint BYTE_SHADOWS = 2;

void main() {
    if (bool(lc.flags & WIREMAP)) {
        vec3 d = fwidth(fragBarycentric);
        vec3 thickness = d * 1.5;
        vec3 edge = smoothstep(vec3(0.0), thickness, fragBarycentric);

        float minEdge = min(min(edge.x, edge.y), edge.z);
        float isEdge = 1.0 - minEdge;

        outColor = vec4(mix(vec3(1.0), fragTriangleColor, isEdge), 1.0);

        return;
    } 

    vec2 screenUV = gl_FragCoord.xy / lc.resolution; 
    uint targetTexId;
    vec2 targetUV;

    // untextured walls
    if (fragTexId == 65535) {
        float scale = fragViewZ * 0.04;

        targetUV = screenUV * scale; 
        targetTexId = fragFloorTexId;
        
    // sky walls or sky ceilings
    } else if (fragTexId == 65534 || fragTexId == 65533) {
        float skyU = lc.cameraYaw / TAU * lc.widthFactor;
        skyU += screenUV.x * (0.4 * lc.widthFactor);
        skyU = fract(skyU);

        float skyV = screenUV.y * 1.6;

        targetUV = vec2(skyU, skyV);
        targetTexId = lc.skyIndex;
    } else {
        float scrolledX = fract(fragTexCoord.x + lc.globalTimer * fragScrollDir);
        targetUV = vec2(scrolledX, fragTexCoord.y);
        targetTexId = fragTexId;
    }

    float rawColor = textureLod(texSamplers[nonuniformEXT(targetTexId)], targetUV, 0.0).r;
    uint colorIndex = uint(rawColor * 255.0 + 0.5);
    if (colorIndex == 255) {
        discard;
    }
    
    if (bool(lc.flags & BYTE_SHADOWS)) {
        outColor = texelFetch(palTex, ivec2(colorIndex, lc.paletteIndex), 0) * (float(fragLightLevel) / 255.0);  

        return;
    }

    uint colormapIdx = 31 - (fragLightLevel >> 3);
    uint finalColormapIdx = (fragTexId == 65534 || fragTexId == 65533) ? 0 : colormapIdx;
    uint shadedIndex = texelFetch(colormapTex, ivec2(colorIndex, finalColormapIdx), 0).r;

    outColor = texelFetch(palTex, ivec2(shadedIndex, lc.paletteIndex), 0);
}
