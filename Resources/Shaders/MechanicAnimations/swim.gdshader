shader_type canvas_item;
#include "res://Resources/Shaders/MechanicAnimations/perspective_tilt.gdshaderinc"
uniform vec2 iResolution;
uniform sampler2D iChannel0: filter_nearest;
uniform float depth_normal;
uniform bool ascending;

void fragment() {
//void mainImage(out vec4 frag_color, in vec2 frag_coord) {
    vec2 uv = UV;
    //vec2 uv = FRAGCOORD.xy / iResolution.xy;
    //vec2 uv = frag_coord.xy / iResolution.xy;
    if (depth_normal <= 0.0) {
        COLOR = texture(iChannel0, uv);
    } else {
        vec2 altered_uv;
        float perspective_tilt;
        compute_perspective_tilt(
            uv,
            depth_normal,
            ascending,
            altered_uv,
            perspective_tilt
        );
        DISCARD_PIXELS_OUTSIDE_OF_ALTERED_UV_BOUNDS_FRAG(altered_uv, TEXTURE_PIXEL_SIZE)
        COLOR = texture(iChannel0, altered_uv);
        //frag_color =  texture(iChannel0, warped_uv);
    }
}
