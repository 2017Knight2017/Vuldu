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

layout(location = 0) in vec3 fragLightLevel;      
layout(location = 1) in vec2 fragTexCoord;
layout(location = 2) flat in uint fragTexId;
layout(location = 3) flat in uint fragSectorId;
layout(location = 4) flat in uint fragColormapIdx;

layout(location = 0) out vec4 outColor;

void main() {
    float rawColor = texture(texSamplers[nonuniformEXT(fragTexId)], fragTexCoord).r;
    uint colorIndex = uint(rawColor * 255.0);
    
    if (colorIndex == 255) {
        discard;
    }

    uint colormapOffset = (fragColormapIdx * 256) | colorIndex;
    uint shadedIndex = uint(colormap.colors[colormapOffset]);
    
    uint colorPosition = (sc.paletteIndex * 256) | shadedIndex;
    vec4 finalColor = pal.colors[colorPosition];

    vec3 modernColor = pal.colors[(sc.paletteIndex * 256) | colorIndex].rgb * fragLightLevel;

    outColor = vec4(finalColor.rgb, 1.0);
    //outColor = vec4(modernColor.rgb, 1.0);
}
