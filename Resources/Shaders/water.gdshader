shader_type canvas_item;

#include "res://Resources/Shaders/constants.gdshaderinc"
uniform sampler2D iChannel0; // Assign: res://Assets/Textures/gray_noise_small.png (noise displacement map)
uniform sampler2D iChannel1; // Assign: res://Assets/Textures/rocks.jpg (background texture)
uniform sampler2D iChannel2; // Assign: res://Assets/Textures/pebbles.png (caustics displacement map)
uniform sampler2D iChannel3; // this should be whats getting updated by buffer

vec4 sample_background_with_disp_map(sampler2D tex, vec2 uv) {
    float height = texture(iChannel3, uv).r;

    vec3 normal = normalize(vec3(
        texture(iChannel3, uv + vec2(NORMAL_SAMPLE_OFFSET, 0.0)).r - texture(iChannel3, uv - vec2(NORMAL_SAMPLE_OFFSET, 0.0)).r,
        texture(iChannel3, uv + vec2(0.0, NORMAL_SAMPLE_OFFSET)).r - texture(iChannel3, uv - vec2(0.0, NORMAL_SAMPLE_OFFSET)).r,
        2.0 * NORMAL_SAMPLE_OFFSET
    ));

    vec2 refracted_offset = refract(INCIDENT_DIRECTION, normal, REFRACTION_INDEX_RATIO).xy;
    vec2 bg_uv = uv + refracted_offset * 0.05;
    return texture(tex, bg_uv);
}

void fragment() {
//void mainImage(out vec4 frag_color, in vec2 frag_coord) {
    //vec2 UV = frag_coord / iResolution.xy;

    vec4 background = sample_background_with_disp_map(iChannel1, UV);
    //frag_color = background;
    COLOR = background;
}


