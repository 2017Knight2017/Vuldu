#version 450
#extension GL_EXT_nonuniform_qualifier : enable

layout(binding = 1) uniform usampler2D texSamplers[512];

layout(binding = 2) uniform PaletteBuffer {
    vec4 colors[3584]; 
} pal;

layout(push_constant) uniform SpriteConstants {
    int paletteIndex;      // offset = 0
    int textureId;         // offset = 4
    float spriteWidth;     // offset = 8
    float spriteHeight;    // offset = 12
    float leftOffset;      // offset = 16
    float topOffset;       // offset = 20
    float padding[2];      // offset = 24
    vec4 spriteWorldPos;   // offset = 32
} sc;

layout(location = 0) in vec2 fragTexCoord;
layout(location = 1) flat in int fragTexId;

layout(location = 0) out vec4 outColor;

void main() {
    uint colorIndex = texture(texSamplers[nonuniformEXT(fragTexId)], fragTexCoord).r;
    
    if (colorIndex == 255) {
        discard;
    }
    
    int colorPosition = (sc.paletteIndex * 256) + int(colorIndex);

    vec4 finalColor = pal.colors[colorPosition];

    outColor = vec4(finalColor.rgb, 1.0);
}
