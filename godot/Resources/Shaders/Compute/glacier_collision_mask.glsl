#[compute]
#version 450
#include "res://Resources/Shaders/Glacier/noise.gdshaderinc"
#include "res://Resources/Shaders/Glacier/projections.gdshaderinc"

layout(local_size_x = 2, local_size_y = 2, local_size_z = 1) in;

layout(std430, push_constant) uniform PushConstants {
    vec2 iResolution;
    float iTime;     
    uint _pad;        
} push_constants;

#define COLLISION_MASK_SSBO_UNIFORM_BINDING 0
// Godot doesn’t allow for r8ui the integer formats so we must use unorm float r8
//layout(r8ui, set = 0, binding = COLLISION_MASK_SSBO_UNIFORM_BINDING) writeonly uniform uimage2D collision_mask_ssbo;
layout(r8, set = 0, binding = COLLISION_MASK_SSBO_UNIFORM_BINDING) writeonly uniform image2D collision_mask_ssbo;

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
    // TODO: again this needs to be float R8_UNORM format for the image, 0.0→0 and 1.0→255 on the GPU.
    // uint solid = solidTop ? 1u : 0u;
    float solid = solidTop ? 1.0 : 0.0;
    //TODO: godot y-down flip 
    ivec2 flipped = ivec2(gid.x, int(push_constants.iResolution.y) - gid.y - 1);
    //unorm rewrite
    //imageStore(collision_mask_ssbo, flipped, uvec4(solid, 0u, 0u, 0u));
    imageStore(collision_mask_ssbo, flipped, vec4(solid, 0.0, 0.0, 0.0));
}
