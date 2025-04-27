#[compute]
#version 450
#include "res://Resources/Shaders/Glacier/noise.gdshaderinc"
#include "res://Resources/Shaders/Glacier/projections.gdshaderinc"
#include "res://Resources/Shaders/Glacier/color.gdshaderinc"
#extension GL_KHR_shader_subgroup_basic : enable  

layout(local_size_x = 2, local_size_y = 2, local_size_z = 1) in;

layout(std430, push_constant) uniform PushConstants {
    vec2 iResolution; // at byte‐offset 0, occupies bytes 0..7
    float iTime;      // at byte‐offset 8, occupies bytes 8..11
    uint _pad;        // at byte‐offset 12, occupies bytes 12..15  
} push_constants;

#define COLLISION_MASK_SSBO_UNIFORM_BINDING 0
layout(r32ui, set = 0, binding = COLLISION_MASK_SSBO_UNIFORM_BINDING) writeonly uniform uimage2D collision_mask_ssbo;

void main() {
    ivec2 gid = ivec2(gl_GlobalInvocationID.xy);
    if (gid.x >= int(push_constants.iResolution.x) ||
        gid.y >= int(push_constants.iResolution.y)) {
        return;
    }
    vec2 pixelCenter = vec2(gid) + vec2(0.5, 0.5);
    vec2 normCoord = vec2(
        (pixelCenter.x * 2.0 - push_constants.iResolution.x) / push_constants.iResolution.y,
        (pixelCenter.y * 2.0 - push_constants.iResolution.y) / push_constants.iResolution.y
    );
    float localNoiseScale;
    vec2  projectedTop = projectTopLayerForParallax(normCoord, localNoiseScale);
    bool  solidTop     = isSolidAtCoord(projectedTop, localNoiseScale, push_constants.iTime);
    vec3 terrainColor  = getTerrainColorBinary(solidTop, normCoord);
    float gammaCorrect = sqrt(terrainColor.r);

    // float maskValue = (gammaCorrect > 0.9) ? 1.0 : 0.0;
    // imageStore(collision_mask_ssbo, gid, vec4(maskValue, 0.0, 0.0, 0.0));
    
    uint maskValue = (gammaCorrect > SOLID_REGION_BRIGHTNESS) ? 1u : 0u;
    imageStore(collision_mask_ssbo, gid, uvec4(maskValue, 0u, 0u, 0u));
}
