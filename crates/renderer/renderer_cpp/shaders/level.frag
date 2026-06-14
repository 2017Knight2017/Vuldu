#version 450
#extension GL_EXT_nonuniform_qualifier : enable
#extension GL_EXT_shader_8bit_storage : require

layout(binding = 1) uniform usampler2D texSamplers[512];

layout(binding = 2) readonly buffer PaletteBuffer {
    vec4 colors[3584]; 
} pal;

layout(binding = 3) readonly buffer ColormapBuffer {
    uint8_t colors[8448];
} colormap;

layout(push_constant) uniform LevelConstants {
    uint paletteIndex;
    uint lightLevel;
} lc;

layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec2 fragTexCoord;
layout(location = 2) flat in uint fragTexId;

layout(location = 0) out vec4 outColor;

void main() {
    uint colorIndex = texture(texSamplers[nonuniformEXT(fragTexId)], fragTexCoord).r;
    
    if (colorIndex == 255) {
        discard;
    }

    uint colormapOffset = (lc.lightLevel * 256) | colorIndex;
    uint shadedIndex = uint(colormap.colors[colormapOffset]);
    
    uint colorPosition = (lc.paletteIndex * 256) | shadedIndex;
    vec4 finalColor = pal.colors[colorPosition];

    outColor = vec4(finalColor.rgb, 1.0);
}