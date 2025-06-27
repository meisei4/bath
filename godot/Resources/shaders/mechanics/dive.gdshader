shader_type canvas_item;
#include "res://Resources/shaders/mechanics/perspective_tilt.gdshaderinc"
uniform vec2      iResolution;
uniform sampler2D iChannel0 : filter_nearest;
uniform float     depth_normal;
uniform bool      ascending;
const float       max_tilt_angle = MAXIMUM_TILT_ANGLE_ACHIEVED_AT_IMMEDIATE_ASCENSION_AND_FINAL_DESCENT;

void fragment() {
    vec2 uv = UV;
    if (depth_normal <= 0.0) {
        COLOR = texture(iChannel0, uv);
    } else {
        vec2  altered_uv;
        float perspective_tilt;
        compute_perspective_tilt(uv, depth_normal, ascending, max_tilt_angle, altered_uv, perspective_tilt);
        DISCARD_PIXELS_OUTSIDE_OF_ALTERED_UV_BOUNDS_FRAG(altered_uv, TEXTURE_PIXEL_SIZE)
        COLOR = texture(iChannel0, altered_uv);
    }
}
