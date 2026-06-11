#version 450
#extension GL_EXT_nonuniform_qualifier : enable

layout(binding = 1) uniform usampler2D texSamplers[512];

layout(binding = 2) uniform PaletteBuffer {
    vec4 colors[3584]; 
} pal;

layout(push_constant) uniform LevelConstants {
    layout(offset = 0) int paletteIndex; // Начинается с 0, размер 4 байта
} lcs;

layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec2 fragTexCoord;
layout(location = 2) flat in int fragTexId;

layout(location = 0) out vec4 outColor;

void main() {
    uint colorIndex = texture(texSamplers[nonuniformEXT(fragTexId)], fragTexCoord).r;
    
    if (colorIndex == 255) {
        discard;
    }
    
    int baseOffset = (lcs.paletteIndex * 768) + (int(colorIndex) * 3);

    float r = float(pal.colors[baseOffset + 0]) / 255.0;
    float g = float(pal.colors[baseOffset + 1]) / 255.0;
    float b = float(pal.colors[baseOffset + 2]) / 255.0;

    outColor = vec4(r, g, b, 1.0) * vec4(fragColor, 1.0);
}