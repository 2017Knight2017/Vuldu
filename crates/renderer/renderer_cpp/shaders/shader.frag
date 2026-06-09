#version 450
#extension GL_EXT_nonuniform_qualifier : enable

layout(binding = 2) uniform PaletteBuffer {
    vec4 colors[3584]; 
} pal;

layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec2 fragTexCoord;
layout(location = 2) flat in int fragTexId;

layout(binding = 1) uniform sampler2D texSamplers[512];

layout(push_constant) uniform PushConstants {
    layout(offset = 64) int paletteIndex;
} pcs;

layout(location = 0) out vec4 outColor;

void main() {
    float rawIndex = texture(texSamplers[nonuniformEXT(fragTexId)], fragTexCoord).r;
    int colorIndex = int(rawIndex * 255.0);

    int flatIndex = (pcs.paletteIndex * 256) + colorIndex;

    vec3 finalColor = pal.colors[flatIndex].rgb;
    outColor = vec4(finalColor, 1.0);
}
