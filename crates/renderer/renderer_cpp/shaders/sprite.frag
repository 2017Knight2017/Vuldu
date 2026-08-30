#version 450
#extension GL_EXT_nonuniform_qualifier : enable

layout(binding = 1) uniform sampler2D palTex;
layout(binding = 2) uniform usampler2D colormapTex;

layout(binding = 4) uniform sampler2D texSamplers[];

layout(push_constant) uniform SpriteConstants {
    uint paletteIndex;
    uint flags;
} sc;

layout(location = 0) in vec2 fragTexCoord;
layout(location = 1) flat in uint fragTexId;
layout(location = 2) flat in uint fragLightLevel;

layout(location = 0) out vec4 outColor;

const uint BYTE_SHADOWS = 2;
const uint FULL_BRIGHT = 4;

void main() {
    float rawColor = textureLod(texSamplers[nonuniformEXT(fragTexId)], fragTexCoord, 0.0).r;
    uint colorIndex = uint(rawColor * 255.0 + 0.5);
    
    if (colorIndex == 255) {
        discard;
    }

    uint finalLight = fragLightLevel;
    if (bool(sc.flags & FULL_BRIGHT))
        finalLight = 255;

    if (bool(sc.flags & BYTE_SHADOWS)) {
        outColor = texelFetch(palTex, ivec2(colorIndex, sc.paletteIndex), 0) * (float(finalLight) / 255.0);  

        return;
    } 

    uint colormapIdx = 31 - (finalLight >> 3);
    uint shadedIndex = texelFetch(colormapTex, ivec2(colorIndex, colormapIdx), 0).r;

    outColor = texelFetch(palTex, ivec2(shadedIndex, sc.paletteIndex), 0);
}
