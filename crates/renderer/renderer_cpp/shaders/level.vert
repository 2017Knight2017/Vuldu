#version 450

layout(binding = 0) uniform MVP {
    mat4 model;
    mat4 view;
    mat4 proj;
} mvp;

struct animLevelInfo {
    uint texId;
    uint frames;
};

const uint ANIM_INFO_SIZE = 4096;

layout(binding = 3) uniform AnimLevelBuffer {
    animLevelInfo info[ANIM_INFO_SIZE];
} anim;

layout(binding = 4) readonly buffer SectorHeightsBuffer {
    float heights[]; 
} sec;

layout(binding = 5) readonly buffer SwitchesBuffer {
    uint ids[];
} sw;

layout(push_constant) uniform LevelConstants {
    vec2 resolution;
    uint paletteIndex;
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
layout(location = 6) in uint inPlaneA;
layout(location = 7) in uint inPlaneB;
layout(location = 8) in float inInvTexH;

layout(location = 0) flat out uint fragLightLevel;      
layout(location = 1) out vec2 fragTexCoord;
layout(location = 2) flat out uint fragTexId;
layout(location = 3) flat out uint fragFloorTexId;
layout(location = 4) out float fragViewZ;
layout(location = 5) out float fragScrollDir;
layout(location = 6) out vec3 fragBarycentric;
layout(location = 7) out vec3 fragTriangleColor;
layout(location = 8) out flat uint fragIsMid;

const uint SW_BASE = 0xF000;
const uint SW_MAX = 0xFFFF - 3;
const uint ANIM_SPEED = 3;

uint getAnimId() {
    uint texId = inTexId >= SW_BASE && inTexId <= SW_MAX 
        ? sw.ids[inTexId - SW_BASE] 
        : inTexId;

    if (texId >= ANIM_INFO_SIZE) return texId;
    animLevelInfo info = anim.info[texId];

    uint frames = info.frames;
    if (frames == 0) return texId;

    uint animStartId = info.texId;

    uint srcFrame = texId - animStartId;
    uint dividedTimer = uint(lc.globalTimer + 0.5) >> ANIM_SPEED;

    uint animFrameNum = frames == 2 || frames == 4 
        ? (srcFrame + dividedTimer) & (frames - 1)
        : (srcFrame + dividedTimer) % 3;

    return animStartId + animFrameNum;
}

vec3 hashColor(uint id) {
    uint state = id * 747796405u + 2891336453u;
    uint word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    uint hashedColor = (word >> 22u) ^ word;

    float r = float(hashedColor & 0xFFu) / 255.0;
    float g = float((hashedColor >> 8) & 0xFFu) / 255.0;
    float b = float((hashedColor >> 16) & 0xFFu) / 255.0;

    return vec3(r, g, b);
}

const uint WIREMAP = 1;

const vec3 BARY[3] = vec3[3](
    vec3(1.0, 0.0, 0.0),
    vec3(0.0, 1.0, 0.0),
    vec3(0.0, 0.0, 1.0)
);

const uint SKY_CEIL = 65533;

const uint TYPE_FLOOR = 0;
const uint TYPE_CEIL = 1;
const uint TYPE_STATIC = 2;
const uint OP_COPY_A = 0;
const uint OP_MIN = 1;

const uint SECTOR_MASK = 0x7FFFFFF;
const uint TYPE_SHIFT = 27;
const uint OP_SHIFT = 29;
const uint ANCHOR_SHIFT = 31;
const uint MID_SHIFT = 29;
const uint PEG_SHIFT = 30;

void main() {
    fragLightLevel = inLightLevel;
    fragScrollDir = inScrollDir;
    fragTexId = getAnimId();
    fragFloorTexId = inFloorTexId;

    uint sectorA = inPlaneA & SECTOR_MASK;
    uint pTypeA = (inPlaneA >> TYPE_SHIFT) & 3;
    uint op = (inPlaneA >> OP_SHIFT) & 3;
    bool an = bool(inPlaneA >> ANCHOR_SHIFT);

    uint sectorB = inPlaneB & SECTOR_MASK;
    uint pTypeB = (inPlaneB >> TYPE_SHIFT) & 3;

    vec3 pos = inPosition;
    float v = inTexCoord.y;

    if (pTypeA != TYPE_STATIC) pos.y = sec.heights[sectorA*2 + pTypeA];

    if (op != OP_COPY_A) {
    	float yb = sec.heights[sectorB*2 + pTypeB];
    	pos.y = (op == OP_MIN) ? min(pos.y, yb) : max(pos.y, yb);
    }

    fragIsMid = inPlaneB >> MID_SHIFT;
    if (fragIsMid != 0) {
        float openLow  = max(sec.heights[sectorA*2 + TYPE_FLOOR],
                             sec.heights[sectorB*2 + TYPE_FLOOR]);
        float openHigh = min(sec.heights[sectorA*2 + TYPE_CEIL],
                             sec.heights[sectorB*2 + TYPE_CEIL]);

        float texH = 1.0 / inInvTexH;
        float anchor = bool(fragIsMid >> 1) ? openLow + texH : openHigh;
        
        v += (anchor - pos.y) * inInvTexH;
    } else if (an) {
    	float anchor = sec.heights[sectorB*2 + pTypeB];
    	v += (anchor - pos.y) * inInvTexH;
    }

    fragTexCoord = vec2(inTexCoord.x, v);

    vec4 viewPos = mvp.view * mvp.model * vec4(pos, 1.0);
    fragViewZ = viewPos.z;

    gl_Position = mvp.proj * viewPos;

    //if (inTexId == SKY_CEIL) {
    //    gl_Position.z = gl_Position.w;
    //}

    if (bool(lc.flags & WIREMAP)) {
        fragBarycentric = BARY[gl_VertexIndex % 3];

        uint triangleID = uint(gl_VertexIndex / 3);
        fragTriangleColor = hashColor(triangleID);
    }
}
