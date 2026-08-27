#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
} ubo;

struct animLevelInfo {
    uint texId;
    uint frames;
};

const uint ANIM_INFO_NUM = 22;

layout(binding = 3) readonly buffer AnimLevelBuffer {
    animLevelInfo info[ANIM_INFO_NUM];
} anim;

layout(push_constant) uniform LevelConstants {
    uint paletteIndex;
    float resolution[2];
    uint skyIndex;
    float widthFactor;
    float globalTimer;
    float cameraYaw;
    uint flags;
} lc;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec2 inTexCoord;
layout(location = 2) in uint inLightLevel;
layout(location = 3) in uint inTexId;
layout(location = 4) in uint inFloorTexId;
layout(location = 5) in float inScrollDir;

layout(location = 0) flat out uint fragLightLevel;      
layout(location = 1) out vec2 fragTexCoord;
layout(location = 2) flat out uint fragTexId;
layout(location = 3) flat out uint fragFloorTexId;
layout(location = 4) out float fragViewZ;
layout(location = 5) out float fragScrollDir;
layout(location = 6) out vec3 fragBarycentric;
layout(location = 7) out vec3 fragTriangleColor;

const uint ANIM_SPEED = 3;

uint getAnimId() {
    for (uint i = 0; i < ANIM_INFO_NUM; i++) {
        uint animStartId = anim.info[i].texId;
        uint frames = anim.info[i].frames;

        if (inTexId >= animStartId && inTexId < animStartId + frames) {
            uint srcFrame = inTexId - animStartId;
            uint dividedTimer = uint(lc.globalTimer + 0.5) >> ANIM_SPEED;

            uint animFrameNum = frames == 2 || frames == 4 
                ? (srcFrame + dividedTimer) & (frames - 1)
                : (srcFrame + dividedTimer) % 3;

            return animStartId + animFrameNum;
        } 
    }

    return inTexId;
}

vec3 hashColor(int id) {
    float r = fract(sin(float(id) * 12.9898) * 43758.5453);
    float g = fract(sin(float(id) * 78.233) * 43758.5453);
    float b = fract(sin(float(id) * 45.164) * 43758.5453);
    return vec3(r, g, b);
}

const uint WIREMAP = 1;

void main() {
    fragLightLevel = inLightLevel;
    fragScrollDir = inScrollDir;
    fragTexCoord = inTexCoord;
    fragTexId = getAnimId();
    fragFloorTexId = inFloorTexId;

    vec4 viewPos = ubo.view * ubo.model * vec4(inPosition, 1.0);

    fragViewZ = abs(viewPos.z);

    gl_Position = ubo.proj * viewPos;

    // sky ceilings
    //if (inTexId == 65533) {
    //    gl_Position.z = gl_Position.w;
    //}

    if (bool(lc.flags & WIREMAP)) {
        int localIndex = gl_VertexIndex % 3;
    
        if (localIndex == 0)      fragBarycentric = vec3(1.0, 0.0, 0.0);
        else if (localIndex == 1) fragBarycentric = vec3(0.0, 1.0, 0.0);
        else                      fragBarycentric = vec3(0.0, 0.0, 1.0);

        int triangleID = gl_VertexIndex / 3;
        fragTriangleColor = hashColor(triangleID);
    }
}
