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

layout(push_constant) uniform UiConstants {
    uint paletteIndex;
} uc;

layout(location = 0) in vec2 fragTexCoord;
layout(location = 1) flat in uint fragTexId;

layout(location = 0) out vec4 outColor;

void main() {
    float rawColor = textureLod(texSamplers[nonuniformEXT(fragTexId)], fragTexCoord, 0.0).r;
	uint colorIndex = uint(rawColor * 255.0 + 0.5);

    if (colorIndex == 255) {
        discard;
    }

    uint shadedIndex = uint(colormap.colors[(7 << 8) | colorIndex]);

    uint colorPosition = (uc.paletteIndex * 256) | shadedIndex;
    vec4 finalColor = pal.colors[colorPosition];

    outColor = vec4(finalColor.rgb, 1.0);
}