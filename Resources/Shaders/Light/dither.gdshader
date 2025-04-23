shader_type canvas_item;

uniform sampler2D iChannel0 : filter_linear_mipmap, repeat_enable; //REPEAT ENABLED!!!!! HUGE!!!
uniform sampler2D iChannel1 : hint_screen_texture;

uniform vec2  iResolution;
uniform vec2  dither_zone_bounds;
uniform float dither_strength = 1.0;

void fragment() {
    vec2 uv = UV;
    vec4 color = texture(iChannel1, uv);
    if (uv.x < dither_zone_bounds.x && uv.y < dither_zone_bounds.y) {
        vec2 i = FRAGCOORD.xy;
        float bayer_threshold = texture(iChannel0, i / 8.0).r;
        vec4 dithered = step(bayer_threshold, color);
        color = mix(color, dithered, dither_strength);
    }
    COLOR = color;
}
