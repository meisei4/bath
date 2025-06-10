shader_type canvas_item;
#include "res://Resources/shaders/mechanics_animations/perspective_tilt.gdshaderinc"

uniform vec2    iResolution;
uniform vec2    sprite_texture_size;
uniform vec2    sprite_scale;
uniform vec2    sprite_position;
uniform float   altitude_normal;
uniform bool    ascending;

uniform sampler2D iChannel0 : filter_nearest;

const float MAX_TILT_ANGLE =
    MAXIMUM_TILT_ANGLE_ACHIEVED_AT_IMMEDIATE_ASCENSION_AND_FINAL_DESCENT;

void fragment() {
    if (altitude_normal <= 0.0) {
        discard;
    }
    vec2 frag_coord = FRAGCOORD.xy;
    vec2 full_size_px = sprite_texture_size * sprite_scale;
    vec2 half_size_px = full_size_px * 0.5;
    vec2 top_left     = sprite_position - half_size_px;
    vec2 bottom_right = sprite_position + half_size_px;
    if (frag_coord.x < top_left.x || frag_coord.x >= bottom_right.x ||
        frag_coord.y < top_left.y || frag_coord.y >= bottom_right.y) {
        discard;
    }
    vec2 normalized_uv = (frag_coord - top_left) / full_size_px;
    vec2 src_texel = floor(normalized_uv * sprite_texture_size);
    vec2 src_uv    = (src_texel + 0.5) / sprite_texture_size;
    vec2 warped_uv_continuous;
    float perspective_tilt;
    compute_perspective_tilt(
        src_uv,
        altitude_normal,
        ascending,
        MAX_TILT_ANGLE,
        warped_uv_continuous,
        perspective_tilt
    );
    vec2 warped_texel = floor(warped_uv_continuous * sprite_texture_size);
    if (warped_texel.x < 0.0 || warped_texel.x > (sprite_texture_size.x - 1.0) ||
        warped_texel.y < 0.0 || warped_texel.y > (sprite_texture_size.y - 1.0)) {
        discard;
    }
    vec2 warped_uv_texcenter = (warped_texel + 0.5) / sprite_texture_size;
    float sprite_alpha = texture(iChannel0, warped_uv_texcenter).a;
    if (sprite_alpha < 0.5) {
        discard;
    }
    float  quantize_strength = 2.0;
    bool quantize_in_screen_space = false;
    float base_step  = quantize_in_screen_space
                  ? 1.0 / full_size_px.y
                  : 1.0 / sprite_texture_size.y;

    float q_step    = base_step * max(1.0, quantize_strength);
    float qauntized_tilt    = floor(perspective_tilt / q_step + 0.5) * q_step;
    COLOR = vec4(qauntized_tilt, 0.0, 0.0, 1.0);
}
