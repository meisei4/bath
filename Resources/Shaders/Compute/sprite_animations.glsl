#[compute]
#version 450
#extension GL_KHR_shader_subgroup_basic : enable  

/*  var groups_x: int = int(ceil(iResolution.x / float(WORKGROUP_TILE_PIXELS_X)))
    var groups_y: int = int(ceil(iResolution.y / float(WORKGROUP_TILE_PIXELS_Y)))
    var groups_z: int = 1
    rendering_device.compute_list_dispatch(compute_list_int, groups_x, groups_y, groups_z)
*/
layout(local_size_x = 2, local_size_y = 2, local_size_z = 1) in;

layout(push_constant, std430) uniform PushConstants {
    vec2 iResolution;    // at byte‐offset 0, occupies bytes 0..7
    uint sprite_count;   // at byte‐offset 8, occupies bytes 8..11
    uint _pad;           // at byte‐offset 12, occupies bytes 12..15
} push_constants;

struct SpriteDataSSBO {
    vec2  center_px;        //  8 bytes (two floats)
    vec2  half_size_px;     //  8 bytes (two floats)
    float altitude_normal;  //  4 bytes (one float)
    float  ascending;       //  4 bytes (one float)
    vec2  _pad;             //  8 bytes (two floats to align to 32-byte multiples)
};

#define SPRITE_DATA_SSBO_UNIFORM_BINDING 0
layout(set = 0, binding = SPRITE_DATA_SSBO_UNIFORM_BINDING, std430) readonly buffer sprite_data_ssbo_uniform {
    SpriteDataSSBO sprites[];
};

#define SPRITE_TEXTURES_BINDING 1      
#define MAXIMUM_SPRITE_COUNT 16 //TODO: BAD BAD BAD figure out how to properly pass this from cpu side godot    
layout(set = 0, binding = SPRITE_TEXTURES_BINDING) uniform sampler2D sprite_textures_uniform[MAXIMUM_SPRITE_COUNT]; 

#define PERSPECTIVE_TILT_MASK_UNIFORM_BINDING 2
layout(set = 0, binding = PERSPECTIVE_TILT_MASK_UNIFORM_BINDING, r32f) writeonly uniform image2D perspective_tilt_mask_uniform;


//JUMP SHADER COPY!!
const float MAXIMUM_TILT_ANGLE_ACHIEVED_AT_IMMEDIATE_ASCENSION_AND_FINAL_DESCENT = 0.785398;
//^^this should be a uniform???
#define FOCAL_LENGTH 1.0
#define NEGATE_TILT_ANGLE -1.0
#define MIDPOINT_UV 0.5
#define DISCARD_PIXELS_OUTSIDE_OF_ALTERED_UV_BOUNDS(uv, texel_size) \
    if ((uv).x < (texel_size).x \
        || (uv).x > 1.0 - (texel_size).x \
        || (uv).y < (texel_size).y \
        || (uv).y > 1.0 - (texel_size).y) \
        return 0.0;

float run_jump_trig_fragment_shader_as_compute_shader(uint sprite_index, in SpriteDataSSBO sprite, in vec2 frag_coords) {
     if (sprite.altitude_normal <= 0.0) {
        return 0.0;
    } 
    bool ascending = (sprite.ascending > 0.5);
    //WTF
    vec2 delta_from_center = abs(frag_coords - sprite.center_px);
    if (delta_from_center.x > sprite.half_size_px.x || delta_from_center.y > sprite.half_size_px.y) {
        return 0.0;
    }
    //WTF
    vec2 uv = (frag_coords - (sprite.center_px - sprite.half_size_px)) / (sprite.half_size_px * 2.0);
    float jump_phase_progress = sprite.altitude_normal;
    float sprite_tilt_phase_progress = 1.0 - jump_phase_progress;
    float current_tilt_angle = sprite_tilt_phase_progress * MAXIMUM_TILT_ANGLE_ACHIEVED_AT_IMMEDIATE_ASCENSION_AND_FINAL_DESCENT;
    current_tilt_angle = NEGATE_TILT_ANGLE * current_tilt_angle;
    float cosine_of_tilt_angle = cos(current_tilt_angle);
    float sine_of_tilt_angle   = sin(current_tilt_angle);
    float L = uv.y;
    if (!ascending) {
        L = 1.0 - uv.y;
    }
    float sprite_quad_height_after_tilt_applied = L * cosine_of_tilt_angle;
    float sprite_quad_depth_after_tilt  = L * sine_of_tilt_angle;
    float perspective_asymptotic_scalar =
            1.0 + sprite_quad_depth_after_tilt
                        /
                  FOCAL_LENGTH;
    float projected_y_vertically_from_perspective_tilt =
            sprite_quad_height_after_tilt_applied
                            /
               perspective_asymptotic_scalar;

    float midpoint_uv_x_of_sprite_quad = uv.x - MIDPOINT_UV;
    float projected_x_horizontal_squish_from_perspective_tilt =
            midpoint_uv_x_of_sprite_quad
                        /
            perspective_asymptotic_scalar;
    vec2 altered_uv;
    altered_uv.x = projected_x_horizontal_squish_from_perspective_tilt + MIDPOINT_UV;
    altered_uv.y = ascending
            ? projected_y_vertically_from_perspective_tilt
            : 1.0 - projected_y_vertically_from_perspective_tilt;
    vec2 texel_size = vec2(
        1.0 / (sprite.half_size_px.x * 2.0),
        1.0 / (sprite.half_size_px.y * 2.0)
    );

    DISCARD_PIXELS_OUTSIDE_OF_ALTERED_UV_BOUNDS(altered_uv, texel_size);
    float sprite_textures_alpha = texture(sprite_textures_uniform[sprite_index], altered_uv).a; 
    if (sprite_textures_alpha < 0.05) {
        return 0.0;
    } 

    float perspective_tilt = ascending
            ? sprite_tilt_phase_progress * altered_uv.y
            : sprite_tilt_phase_progress * (1.0 - altered_uv.y);
    return clamp(perspective_tilt, 0.0, 1.0);
}

//WTF
void main() {
    ivec2 frag_coords = ivec2(gl_GlobalInvocationID.xy);
    if (frag_coords.x >= int(push_constants.iResolution.x) || frag_coords.y >= int(push_constants.iResolution.y)) {
        return;
    }
    vec2 frag_coord_centers = vec2(frag_coords) + vec2(MIDPOINT_UV);
    float max_perspective_tilt = 0.0;
    for (uint i = 0u; i < push_constants.sprite_count; ++i) {
        float perspective_tilt = run_jump_trig_fragment_shader_as_compute_shader(
            i,
            sprites[i],
            frag_coord_centers
        );
        max_perspective_tilt = max(max_perspective_tilt, perspective_tilt);
    }
    imageStore(
      perspective_tilt_mask_uniform,
      frag_coords,
      vec4(max_perspective_tilt, 0.0, 0.0, 0.0)
    );
}
