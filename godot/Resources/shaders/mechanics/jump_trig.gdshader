shader_type canvas_item;
#include "res://Resources/shaders/mechanics/perspective_tilt.gdshaderinc"
uniform vec2      iResolution;
uniform sampler2D iChannel0 : filter_nearest;
uniform float     altitude_normal;
uniform bool      ascending;
const float       max_tilt_angle = MAXIMUM_TILT_ANGLE_ACHIEVED_AT_IMMEDIATE_ASCENSION_AND_FINAL_DESCENT;

// TODO: this is the next most important part is trying to figure out the best way to clean up the
// pixel flicker during the warp sequence

// #define BILINEAR_MIX
#define PIXEL_NEAREST
// #define PIXEL_LOCK
// #define SUBTEXEL_BLEND

void fragment() {
    vec2 pixel           = floor(UV * iResolution);
    vec2 center_pixel_uv = (pixel + 0.5) / iResolution;
    if (altitude_normal <= 0.0) {
        COLOR = texture(iChannel0, center_pixel_uv);
    } else {
        vec2  warped_uv_continious;
        float perspective_tilt;
        compute_perspective_tilt(
            center_pixel_uv, altitude_normal, ascending, max_tilt_angle, warped_uv_continious, perspective_tilt);
        vec2 warped_virtual_pixel = floor(warped_uv_continious * iResolution);
        if (warped_virtual_pixel.x < 0.0 || warped_virtual_pixel.x >= iResolution.x || warped_virtual_pixel.y < 0.0
            || warped_virtual_pixel.y >= iResolution.y) {
            COLOR = vec4(0.0);
        } else {
#if defined(PIXEL_LOCK)
            vec2 round_px = floor(warped_uv_continious * iResolution + 0.5);
            vec2 round_uv = (round_px + 0.5) / iResolution;
            COLOR         = texture(iChannel0, round_uv);
#elif defined(SUBTEXEL_BLEND)
            vec2 float_px = warped_uv_continious * iResolution;
            vec2 f        = fract(float_px);
            vec2 base     = floor(float_px);
            vec2 uv00     = (base + 0.5) / iResolution;
            vec2 uv10     = (base + vec2(1, 0) + 0.5) / iResolution;
            vec2 uv01     = (base + vec2(0, 1) + 0.5) / iResolution;
            vec2 uv11     = (base + vec2(1, 1) + 0.5) / iResolution;
            vec4 c0       = mix(texture(iChannel0, uv00), texture(iChannel0, uv10), f.x);
            vec4 c1       = mix(texture(iChannel0, uv01), texture(iChannel0, uv11), f.x);
            COLOR         = mix(c0, c1, f.y);
#elif defined(BILINEAR_MIX)
            vec2 float_px = warped_uv_continious * iResolution;
            vec2 f        = fract(float_px);
            vec2 base     = floor(float_px);
            vec2 uv00     = (base + 0.5) / iResolution;
            vec2 uv10     = (base + vec2(1, 0) + 0.5) / iResolution;
            vec2 uv01     = (base + vec2(0, 1) + 0.5) / iResolution;
            vec2 uv11     = (base + vec2(1, 1) + 0.5) / iResolution;
            vec4 c0       = mix(texture(iChannel0, uv00), texture(iChannel0, uv10), f.x);
            vec4 c1       = mix(texture(iChannel0, uv01), texture(iChannel0, uv11), f.x);
            COLOR         = mix(c0, c1, f.y);
#else
            vec2 warped_virtual_pixel_uv = (warped_virtual_pixel + 0.5) / iResolution;
            COLOR                        = texture(iChannel0, warped_virtual_pixel_uv);
#endif
        }
    }
}
