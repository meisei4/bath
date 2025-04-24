#[compute]
#version 450
#extension GL_KHR_shader_subgroup_basic : enable  

/*  var groups_x = ceil(iResolution.x / 8)
    var groups_y = ceil(iResolution.y / 8)
    rendering_device.compute_list_dispatch(cl, groups_x, groups_y, 1)
*/
layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

/*  var compute_list_int: int = rendering_device.compute_list_begin()
    rendering_device.compute_list_bind_compute_pipeline(compute_list_int, compute_pipeline_rid)
    rendering_device.compute_list_bind_uniform_set(
        compute_list_int, sprite_data_buffer_uniform_set_rid, 0
    )
    push_constants = PackedByteArray()
    push_constants.encode_float(0, iResolution.x)  # float at bytes 0–3
    push_constants.encode_float(4, iResolution.y)  # float at bytes 4–7
    push_constants.encode_u32(8, sprite_data_buffer_array.size()) # uint at bytes 8–11
    rendering_device.compute_list_push_constants(
        compute_list_int, 0, push_constants.size(), push_constants
    )*/
layout(push_constant, std430) uniform PushConstants {
    vec2 iResolution;    // at byte‐offset 0, occupies bytes 0..7
    uint sprite_count;   // at byte‐offset 8, occupies bytes 8..11
    uint _pad;           // at byte‐offset 12, occupies bytes 12..15
}
push_constants;

struct SpriteData {
    vec2  center_px;        //  8 bytes (two floats)
    vec2  half_size_px;     //  8 bytes (two floats)
    float altitude_normal;  //  4 bytes (one float)
    float  ascending;       //  4 bytes (one float)
    vec2  _pad;             //  8 bytes (two floats to align to 32-byte multiples)
};

/*  sprite_data_buffer_id = rendering_device.storage_buffer_create(
        SSBO_TOTAL_BYTES, PackedByteArray()
    )
    ssbo_uniform = RDUniform.new()
    ssbo_uniform.uniform_type = RenderingDevice.UNIFORM_TYPE_STORAGE_BUFFER
    ssbo_uniform.binding = 0
    ssbo_uniform.add_id(sprite_data_buffer_id)*/
#define SSBO_UNIFORM_BINDING 0
layout(set = 0, binding = SSBO_UNIFORM_BINDING, std430) readonly buffer ssbo_uniform {
    SpriteData sprites[];
};

//See:
/*  perspective_tilt_mask_texture_format = RDTextureFormat.new()
    perspective_tilt_mask_texture_format.texture_type = RenderingDevice.TEXTURE_TYPE_2D
    perspective_tilt_mask_texture_format.format = RenderingDevice.DATA_FORMAT_R32_SFLOAT
    perspective_tilt_mask_texture_format.width = iResolution.x as int
    perspective_tilt_mask_texture_format.height = iResolution.y as int
    perspective_tilt_mask_texture_format.depth = 1
    perspective_tilt_mask_texture_format.array_layers = 1
    perspective_tilt_mask_texture_format.mipmaps = 1
    perspective_tilt_mask_texture_format.usage_bits = (
        RenderingDevice.TEXTURE_USAGE_STORAGE_BIT
        | RenderingDevice.TEXTURE_USAGE_CAN_UPDATE_BIT
        | RenderingDevice.TEXTURE_USAGE_SAMPLING_BIT
    )
    perspective_tilt_mask_view = RDTextureView.new()
    perspective_tilt_mask_texture_view_rid = rendering_device.texture_create(
        perspective_tilt_mask_texture_format, perspective_tilt_mask_view
    )
    perspective_tilt_mask_texture = Texture2DRD.new()
    perspective_tilt_mask_texture.set_texture_rd_rid(perspective_tilt_mask_texture_view_rid)
    perspective_tilt_mask_uniform = RDUniform.new()
    perspective_tilt_mask_uniform.uniform_type = RenderingDevice.UNIFORM_TYPE_IMAGE
    perspective_tilt_mask_uniform.binding = 1
    perspective_tilt_mask_uniform.add_id(perspective_tilt_mask_texture_view_rid)*/
#define PERSPECTIVE_TILT_MASK_UNIFORM_BINDING 1
layout(set = 0, binding = PERSPECTIVE_TILT_MASK_UNIFORM_BINDING, r32f) writeonly uniform image2D perspective_tilt_mask_texture_image_uniform;

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

float run_jump_trig_fragment_shader_as_compute_shader(in SpriteData sprite, in vec2 frag_coords) {
     if (sprite.altitude_normal <= 0.0) {
        return 0.0;
    } 
    bool ascending = (sprite.ascending > 0.5);
    vec2 delta_from_center = abs(frag_coords - sprite.center_px);
    if (delta_from_center.x > sprite.half_size_px.x || delta_from_center.y > sprite.half_size_px.y) {
        return 0.0;
    }
    // → frag_coords is pixel XY on screen
    // → (center-half_size) is the sprite's bottom-left in pixel coords
    // → dividing by (half_size * 2) maps the pixel into [0..1]×[0..1] sprite UV
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
            sprites[i],
            frag_coord_centers
        );
        max_perspective_tilt = max(max_perspective_tilt, perspective_tilt);
    }
    imageStore(
      perspective_tilt_mask_texture_image_uniform,
      frag_coords,
      vec4(max_perspective_tilt, 0.0, 0.0, 0.0)
    );
}
