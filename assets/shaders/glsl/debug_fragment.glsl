#version 330
in vec2 fragCoord;
in vec2 fragTexCoord;
// TODO: this is the proof of how the triangles are oriented in the quad:
// flat in vec4 fragColor;
in vec4 fragColor;

in vec3 vertexDebugPayload0;
in vec4 vertexDebugPayload1;

out vec4 finalColor;

uniform vec2 iResolution;

const float depthScalar     = 6.0;
const int   tileSize        = 32;
const float zigzagAmplitude = 1.3;

const vec4 BLACK = vec4(0.0, 0.0, 0.0, 1.0);
const vec4 WHITE = vec4(1.0, 1.0, 1.0, 1.0);

// #define PASS
#define GRID_UV
// #define BLACK_LODGE_FLOOR

void main() {
    vec4 fColor = fragColor;
#ifdef GRID_UV
    ivec2 tileCoord = ivec2(floor(fragTexCoord * iResolution / tileSize));
    int   sum       = tileCoord.x + tileCoord.y;
    fColor          = (sum % 2) == 0 ? WHITE : BLACK;
#endif
#ifdef BLACK_LODGE_FLOOR
    vec2  tileCoord       = fragTexCoord * iResolution / float(tileSize);
    float localTileCoord  = fract(tileCoord.y);
    float zig             = abs(localTileCoord * 2.0 - 1.0);
    float zag             = zig * zigzagAmplitude;
    ivec2 zigzagTileCoord = ivec2(floor(vec2(tileCoord.x + zag, tileCoord.y)));
    zigzagTileCoord.y *= 2;
    int sum = zigzagTileCoord.x + zigzagTileCoord.y;
    fColor  = (sum % 2) == 0 ? WHITE : BLACK;
#endif
#ifdef PASS
    finalColor = fragColor;
    return;
#endif
    finalColor = fColor;
}