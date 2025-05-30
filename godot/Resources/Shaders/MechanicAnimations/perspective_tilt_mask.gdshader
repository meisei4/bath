shader_type canvas_item;
//render_mode blend_disabled;
#include "res://Resources/Shaders/MechanicAnimations/perspective_tilt.gdshaderinc"

uniform vec2    iResolution;
//uniform int     sprite_count;
//uniform vec4    sprite_data0[12];
//uniform vec4    sprite_data1[12];
//uniform sampler2D sprite_textures[12];
uniform vec4    sprite_data0;
uniform vec4    sprite_data1;
//TODO: I think somewhere during the shader handoff in the mechanics there is a tiny fuck up which is
// causing that tiny little white sliver to show up in the non-dithered regions when the jump shader activates
// and thus this mask is a little late or something, i have no idea, ill figure it out when working more with mechanic
// shader handovers
uniform sampler2D iChannel0: filter_nearest;

const float MAX_TILT_ANGLE =
    MAXIMUM_TILT_ANGLE_ACHIEVED_AT_IMMEDIATE_ASCENSION_AND_FINAL_DESCENT;

void fragment() {
    vec2 frag_coord = FRAGCOORD.xy;
    float max_tilt = 0.0;
    float alpha = 0.0;
    //for (int i = 0; i < sprite_count; ++i) {
    for (int i = 0; i < 1; ++i) {
        //vec2  center_px    = sprite_data0[i].xy;
        //vec2  half_size_px = sprite_data0[i].zw;
        //float altitude_n   = sprite_data1[i].x;
        //bool  ascending    = (sprite_data1[i].y > 0.5);
        vec2  center_px    = sprite_data0.xy;
        vec2  half_size_px = sprite_data0.zw;
        float altitude_normal   = sprite_data1.x;
        bool  ascending    = (sprite_data1.y > 0.5);
        if (altitude_normal <= 0.0) {
            continue;
        }
        vec2 uv = (frag_coord + vec2(0.5) - (center_px - half_size_px)) / (half_size_px * 2.0);
        vec2  base_texel       = floor(uv * iResolution);
        vec2  center_pixel_uv  = (base_texel + 0.5) / iResolution;
        vec2 warped_uv_continious;
        float perspective_tilt;
        compute_perspective_tilt(
            center_pixel_uv,
            altitude_normal,
            ascending,
            MAX_TILT_ANGLE,
            warped_uv_continious,
            perspective_tilt
        );
        vec2 warped_virtual_pixel = floor(warped_uv_continious * iResolution);
        if (warped_virtual_pixel.x < 0.0
            || warped_virtual_pixel.x >= iResolution.x
            || warped_virtual_pixel.y < 0.0
            || warped_virtual_pixel.y >= iResolution.y) {
                continue;
        }
        vec2 warped_virtual_pixel_uv = (warped_virtual_pixel + 0.5) / iResolution;
        float sprite_alpha = texture(iChannel0, warped_virtual_pixel_uv).a;
        if (sprite_alpha == 0.0) {
            continue;
        }
        max_tilt = max(max_tilt, clamp(perspective_tilt, 0.0, 1.0));
        alpha = 1.0;
    }
    vec4 col = vec4(max_tilt, 0.0, 0.0, alpha);
    COLOR = col;
}