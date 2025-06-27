shader_type canvas_item;

uniform sampler2D iChannel0 : filter_nearest, repeat_enable; // REPEAT ENABLED!!!!! HUGE!!!
uniform sampler2D iChannel1 : hint_screen_texture, filter_nearest;

uniform vec2  iResolution;
uniform vec2  dither_zone_bounds;
uniform float BAYER_DIVISION_SCALAR       = 8.0;
uniform float dither_interpolation_weight = 0.5;

void fragment() {
    vec2 uv    = UV;
    vec4 color = texture(iChannel1, uv);
    if (uv.x < dither_zone_bounds.x && uv.y < dither_zone_bounds.y) {
        vec2  frag_coord      = FRAGCOORD.xy;
        float bayer_threshold = texture(iChannel0, frag_coord / BAYER_DIVISION_SCALAR).r;
        vec4  dithered        = step(bayer_threshold, color);
        color                 = mix(color, dithered, dither_interpolation_weight);
    }
    COLOR = color;
}
