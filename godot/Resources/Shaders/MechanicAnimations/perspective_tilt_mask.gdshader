shader_type canvas_item;
render_mode blend_disabled; 
#include "res://Resources/Shaders/MechanicAnimations/perspective_tilt.gdshaderinc"


//TODO: this is the fragment shader rewrite of the compute shader perspective tilt mask
// still not 100% confident on it but it seems to work...
// there might be some issues with the sprite boundaries not being followed in the mask shape.. idk
uniform vec2    iResolution;
uniform int     sprite_count;
uniform vec4    sprite_data0[16];
uniform vec4    sprite_data1[16];
uniform sampler2D sprite_textures[16];

const float MAX_TILT_ANGLE =
    MAXIMUM_TILT_ANGLE_ACHIEVED_AT_IMMEDIATE_ASCENSION_AND_FINAL_DESCENT;

void fragment() {
    vec2 frag_coord = FRAGCOORD.xy;
    float max_tilt = 0.0;
    for (int i = 0; i < sprite_count; ++i) {
        vec2  center_px    = sprite_data0[i].xy;
        vec2  half_size_px = sprite_data0[i].zw;
        float altitude_n   = sprite_data1[i].x;
        bool  ascending    = (sprite_data1[i].y > 0.5);
        if (altitude_n <= 0.0) {
            continue;
        }
        vec2 d = abs(frag_coord - center_px);
        if (d.x > half_size_px.x || d.y > half_size_px.y) {
            continue;
        }
        vec2 uv = (frag_coord - (center_px - half_size_px)) / (half_size_px * 2.0);
        vec2 warped_uv;
        float tilt_val;
        compute_perspective_tilt(
            uv,
            altitude_n,
            ascending,
            MAX_TILT_ANGLE,
            warped_uv,
            tilt_val
        );
        vec2 sprite_uv_pixel_step = 1.0 / (half_size_px * 2.0);
        DISCARD_PIXELS_OUTSIDE_OF_ALTERED_UV_BOUNDS_FRAG(warped_uv, sprite_uv_pixel_step);
        if (texture(sprite_textures[i], warped_uv).a == 0.0) {
            continue;
        }
        max_tilt = max(max_tilt, clamp(tilt_val, 0.0, 1.0));
    }
    COLOR = vec4(max_tilt, 0.0, 0.0, 0.0);
}
