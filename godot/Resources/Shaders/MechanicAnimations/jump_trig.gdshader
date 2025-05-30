shader_type canvas_item;
#include "res://Resources/Shaders/MechanicAnimations/perspective_tilt.gdshaderinc"
uniform vec2 iResolution;
uniform sampler2D iChannel0: filter_nearest;
uniform float altitude_normal;
uniform bool  ascending;
const float max_tilt_angle = MAXIMUM_TILT_ANGLE_ACHIEVED_AT_IMMEDIATE_ASCENSION_AND_FINAL_DESCENT;

void fragment() {
    vec2 pixel = floor(UV * iResolution);
    vec2 center_pixel_uv = (pixel + 0.5) / iResolution;
    if (altitude_normal <= 0.0) {
        COLOR = texture(iChannel0, center_pixel_uv);
    } else {
        vec2 warped_uv_continious;
        float perspective_tilt;
        compute_perspective_tilt(
            center_pixel_uv,
            altitude_normal,
            ascending,
            max_tilt_angle,
            warped_uv_continious,
            perspective_tilt
        );
        vec2 warped_virtual_pixel = floor(warped_uv_continious * iResolution);
        if (warped_virtual_pixel.x < 0.0
            || warped_virtual_pixel.x >= iResolution.x
            || warped_virtual_pixel.y < 0.0
            || warped_virtual_pixel.y >= iResolution.y) {
                COLOR = vec4(0.0);
        } else {
            vec2 warped_virtual_pixel_uv = (warped_virtual_pixel + 0.5) / iResolution;
            COLOR = texture(iChannel0, warped_virtual_pixel_uv);
        }
    }
}
