#[compute]
#version 450
#include "res://Resources/Shaders/MechanicAnimations/perspective_tilt.gdshaderinc"
#extension GL_KHR_shader_subgroup_basic : enable  

layout(local_size_x = 2, local_size_y = 2, local_size_z = 1) in;

layout(std430, push_constant) uniform PushConstants {
    vec2 iResolution;    // at byte-offset 0, occupies bytes 0..7
    uint sprite_count;   // at byte-offset 8, occupies bytes 8..11
    uint _pad;           // at byte-offset 12, occupies bytes 12..15
} push_constants;

struct SpriteDataSSBO {
    vec2  center_px;        //  8 bytes (two floats)
    vec2  half_size_px;     //  8 bytes (two floats)
    float altitude_normal;  //  4 bytes (one float)
    float  ascending;       //  4 bytes (one float)
    vec2  _pad;             //  8 bytes (two floats to align to 32-byte multiples)
};

#define SPRITE_DATA_SSBO_UNIFORM_BINDING 0
layout(std430, set = 0, binding = SPRITE_DATA_SSBO_UNIFORM_BINDING) readonly buffer sprite_data_ssbo_uniform {
    SpriteDataSSBO sprites[];
};

#define SPRITE_TEXTURES_BINDING 1      
#define MAXIMUM_SPRITE_COUNT 16 //TODO: BAD BAD BAD figure out how to properly pass this from cpu side godot    
layout(set = 0, binding = SPRITE_TEXTURES_BINDING) uniform sampler2D sprite_textures_uniform[MAXIMUM_SPRITE_COUNT]; 

#define PERSPECTIVE_TILT_MASK_UNIFORM_BINDING 2
layout(r32f, set = 0, binding = PERSPECTIVE_TILT_MASK_UNIFORM_BINDING) writeonly uniform image2D perspective_tilt_mask_uniform;


#define DISCARD_PIXELS_OUTSIDE_OF_ALTERED_UV_BOUNDS_COMPUTE(uv, texel_size) \
    if ((uv).x < (texel_size).x \
        || (uv).x > 1.0 - (texel_size).x \
        || (uv).y < (texel_size).y \
        || (uv).y > 1.0 - (texel_size).y) \
        return 0.0;


float compute_perspective_tilt_mask_for_sprite(in uint sprite_index, in SpriteDataSSBO sprite, in vec2 frag_coord_centers) {
     if (sprite.altitude_normal <= 0.0) {
        return 0.0;
    } 
    bool ascending = (sprite.ascending > 0.5);
    //WTF
    vec2 delta_from_center = abs(frag_coord_centers - sprite.center_px);
    if (delta_from_center.x > sprite.half_size_px.x || delta_from_center.y > sprite.half_size_px.y) {
        return 0.0;
    }
    //WTF
    vec2 uv = (frag_coord_centers - (sprite.center_px - sprite.half_size_px)) / (sprite.half_size_px * 2.0);
    vec2 altered_uv;
    float perspective_tilt;
    compute_perspective_tilt(
        uv,
        sprite.altitude_normal,
        (sprite.ascending > 0.5),
        altered_uv,
        perspective_tilt
    );
    vec2 texel_size = vec2(
        1.0 / (sprite.half_size_px.x * 2.0),
        1.0 / (sprite.half_size_px.y * 2.0)
    );
    DISCARD_PIXELS_OUTSIDE_OF_ALTERED_UV_BOUNDS_COMPUTE(altered_uv, texel_size);
    float sprite_textures_alpha = texture(sprite_textures_uniform[sprite_index], altered_uv).a; 
    if (sprite_textures_alpha == 0.0) {
        return 0.0;
    }
    return clamp(perspective_tilt, 0.0, 1.0);
}

void main() {
    ivec2 frag_coords = ivec2(gl_GlobalInvocationID.xy);
    if (frag_coords.x >= int(push_constants.iResolution.x) || frag_coords.y >= int(push_constants.iResolution.y)) {
        return;
    }
    vec2 frag_coord_centers = vec2(frag_coords) + vec2(MIDPOINT_UV);
    float max_perspective_tilt = 0.0;

    for (uint i = 0u; i < push_constants.sprite_count; ++i) {
        float perspective_tilt = compute_perspective_tilt_mask_for_sprite(i, sprites[i], frag_coord_centers);
        max_perspective_tilt = max(max_perspective_tilt, perspective_tilt);
    }
    imageStore(perspective_tilt_mask_uniform, frag_coords, vec4(max_perspective_tilt, 0.0, 0.0, 0.0));
}
