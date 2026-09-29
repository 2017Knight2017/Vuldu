#version 450

layout(location = 0) in vec3 inVertexPos; 
layout(location = 1) in vec2 inTexCoord; 
layout(location = 2) in vec2 inInstancePos; 
layout(location = 3) in vec2 inInstanceSize; 
layout(location = 4) in uint inInstanceTexId; 

layout(location = 0) out vec2 fragTexCoord;
layout(location = 1) flat out uint fragTexId;

const vec2 orgResolution = vec2(320.0, 200.0);

void main() {
    fragTexCoord = inTexCoord;
    fragTexId = inInstanceTexId;

    // { 320.0, 32.0 } = { 1.0, 1.0 } * { 320.0, 32.0 } + { 0.0, 0.0 }
    vec2 pixelPos = (inVertexPos.xy * inInstanceSize) + inInstancePos;

    // { 1.0, -0.68 } = ({ 320.0, 32.0 } / { 320.0, 200.0 }) * 2.0 - 1.0
    vec2 ndcPos = (pixelPos / orgResolution) * 2.0 - 1.0;

    gl_Position = vec4(ndcPos, 0.0, 1.0);
}
