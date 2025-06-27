shader_type canvas_item;
#include "res://Resources/shaders/mechanics/perspective_tilt.gdshaderinc"

const int MAXIMUM_SPRITE_COUNT = 16;

uniform sampler2D _sprite_textures[MAXIMUM_SPRITE_COUNT] : filter_nearest;
uniform vec2      sprite_texture_sizes[MAXIMUM_SPRITE_COUNT];
uniform vec2      sprite_scales[MAXIMUM_SPRITE_COUNT];
uniform vec2      sprite_position_data[MAXIMUM_SPRITE_COUNT];
uniform float     altitude_normal_data[MAXIMUM_SPRITE_COUNT];
uniform bool      ascending_data[MAXIMUM_SPRITE_COUNT];
uniform vec2      iResolution;

const float MAX_TILT_ANGLE = MAXIMUM_TILT_ANGLE_ACHIEVED_AT_IMMEDIATE_ASCENSION_AND_FINAL_DESCENT;

void fragment() {
    vec2 frag_coord = FRAGCOORD.xy;
    bool hit        = false;
    vec4 out_color  = vec4(0.0);
    for (int i = 0; i < MAXIMUM_SPRITE_COUNT; i++) {
        float altitude_n = altitude_normal_data[i];
        if (altitude_n <= 0.0) {
            continue;
        }
        vec2 tex_size     = sprite_texture_sizes[i];
        vec2 scale        = sprite_scales[i];
        vec2 pos          = sprite_position_data[i];
        bool ascend       = ascending_data[i];
        vec2 full_size_px = tex_size * scale;
        vec2 half_size_px = full_size_px * 0.5;
        vec2 top_left     = pos - half_size_px;
        vec2 bottom_right = pos + half_size_px;
        if (frag_coord.x < top_left.x || frag_coord.x >= bottom_right.x || frag_coord.y < top_left.y
            || frag_coord.y >= bottom_right.y) {
            continue;
        }
        vec2  normalized_uv = (frag_coord - top_left) / full_size_px;
        vec2  src_texel     = floor(normalized_uv * tex_size);
        vec2  src_uv        = (src_texel + 0.5) / tex_size;
        vec2  warped_uv_continuous;
        float perspective_tilt;
        compute_perspective_tilt(src_uv, altitude_n, ascend, MAX_TILT_ANGLE, warped_uv_continuous, perspective_tilt);
        vec2 warped_texel = floor(warped_uv_continuous * tex_size);
        if (warped_texel.x < 0.0 || warped_texel.x > (tex_size.x - 1.0) || warped_texel.y < 0.0
            || warped_texel.y > (tex_size.y - 1.0)) {
            continue;
        }
        vec2  warped_uv_texcenter = (warped_texel + 0.5) / tex_size;
        float sprite_alpha        = texture(_sprite_textures[i], warped_uv_texcenter).a;
        if (sprite_alpha < 0.5) {
            continue;
        }
        float quantize_strength        = 2.0;
        bool  quantize_in_screen_space = false;
        float base_step                = quantize_in_screen_space ? 1.0 / full_size_px.y : 1.0 / tex_size.y;
        float q_step                   = base_step * max(1.0, quantize_strength);
        float qauntized_tilt           = floor(perspective_tilt / q_step + 0.5) * q_step;
        out_color                      = vec4(qauntized_tilt, 0.0, 0.0, 1.0);
        hit                            = true;
        break;
    }
    if (hit) {
        COLOR = out_color;
    } else {
        discard;
    }
}
