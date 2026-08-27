#version 450
#extension GL_EXT_nonuniform_qualifier : enable
#extension GL_EXT_shader_8bit_storage : require

layout(binding = 1) readonly buffer PaletteBuffer {
    vec4 colors[3584]; 
} pal;

layout(binding = 2) readonly buffer ColormapBuffer {
    uint8_t colors[8448]; 
} colormap;

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

void main() {
    float rawColor = textureLod(texSamplers[nonuniformEXT(fragTexId)], fragTexCoord, 0.0).r;
    uint colorIndex = uint(rawColor * 255.0 + 0.5);
    
    if (colorIndex == 255) {
        discard;
    }

    if (bool(sc.flags & BYTE_SHADOWS)) {
        vec3 modernColor = pal.colors[(sc.paletteIndex << 8) | colorIndex].rgb * float(fragLightLevel);

        outColor = vec4(modernColor.rgb, 1.0);

        return;
    } 

    uint colormapIdx = 31 - (fragLightLevel >> 3);
    uint colormapOffset = (colormapIdx << 8) | colorIndex;
    uint shadedIndex = uint(colormap.colors[colormapOffset]);

    uint colorPosition = (sc.paletteIndex << 8) | shadedIndex;
    vec4 finalColor = pal.colors[colorPosition];

    outColor = vec4(finalColor.rgb, 1.0);
}
