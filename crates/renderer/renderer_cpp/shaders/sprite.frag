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
    uint paletteIndex;
} sc;

layout(location = 0) in vec2 fragTexCoord;
layout(location = 1) flat in uint fragTexId;
layout(location = 2) in float fragLightLevel;
layout(location = 3) flat in uint fragColormapIdx;

layout(location = 0) out vec4 outColor;

void main() {
    float rawColor = textureLod(texSamplers[nonuniformEXT(fragTexId)], fragTexCoord, 0.0).r;
    uint colorIndex = uint(rawColor * 255.0);
    
    if (colorIndex == 255) {
        discard;
    }

    // COLORMAP shadows
    uint colormapOffset = (fragColormapIdx * 256) | colorIndex;
    uint shadedIndex = uint(colormap.colors[colormapOffset]);
    
    uint colorPosition = (sc.paletteIndex * 256) | shadedIndex;
    vec4 finalColor = pal.colors[colorPosition];

    outColor = vec4(finalColor.rgb, 1.0);

    // 256-unit shadows
    //vec3 modernColor = pal.colors[(sc.paletteIndex * 256) | colorIndex].rgb * fragLightLevel;
    //outColor = vec4(modernColor.rgb, 1.0);
}
