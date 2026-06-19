#version 450
#extension GL_EXT_nonuniform_qualifier : enable
#extension GL_EXT_shader_8bit_storage : require

layout(binding = 1) readonly buffer PaletteBuffer {
    vec4 colors[3584]; 
} pal;

layout(binding = 2) readonly buffer ColormapBuffer {
    uint8_t colors[8448]; 
} colormap;

layout(binding = 3) uniform sampler2D texSamplers[];

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

layout(location = 0) in vec2 fragTexCoord;
layout(location = 1) flat in uint fragTexId;

layout(location = 0) out vec4 outColor;

void main() {
    float rawColor = texture(texSamplers[nonuniformEXT(fragTexId)], fragTexCoord).r;
    uint colorIndex = uint(rawColor * 255.0);
    
    if (colorIndex == 255) {
        discard;
    }

    uint colormapOffset = (sc.lightLevel * 256) | colorIndex;
    uint shadedIndex = uint(colormap.colors[colormapOffset]);
    
    uint colorPosition = (sc.paletteIndex * 256) | shadedIndex;
    vec4 finalColor = pal.colors[colorPosition];

    outColor = vec4(finalColor.rgb, 1.0);
}

