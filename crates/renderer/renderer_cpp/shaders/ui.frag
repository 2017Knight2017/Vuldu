#version 450
#extension GL_EXT_nonuniform_qualifier : enable

layout(binding = 1) uniform sampler2D palTex;
layout(binding = 2) uniform usampler2D colormapTex;

layout(binding = 5) uniform sampler2D texSamplers[];

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

    uint shadedIndex = texelFetch(colormapTex, ivec2(colorIndex, 7), 0).r;

    outColor = texelFetch(palTex, ivec2(shadedIndex, uc.paletteIndex), 0);
}