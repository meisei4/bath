shader_type canvas_item;

uniform sampler2D iChannel0 : filter_nearest, repeat_enable; //REPEAT ENABLED!!!!! HUGE!!!
uniform sampler2D iChannel1 : hint_screen_texture, filter_nearest;

uniform vec2  iResolution;
uniform vec2  dither_zone_bounds;
uniform float BAYER_DIVISION_SCALAR = 8.0;
uniform float dither_interpolation_weight = 0.5;

void fragment() {
    vec2 uv = UV;
    // 1) sample the current screen (with umbral already applied)
    vec4 color = texture(iChannel1, uv);

    // 2) only dither inside your zone
    if (uv.x < dither_zone_bounds.x && uv.y < dither_zone_bounds.y) {
        // FRAGCOORD.xy is in pixel coordinates now?
        vec2 frag_coord = FRAGCOORD.xy;

        // 3) map each pixel into your 8×8 Bayer tile
        //    dividing by 8 means “one Bayer‐cell covers 8×8 screen pixels”
        float bayer_threshold = texture(iChannel0, frag_coord / BAYER_DIVISION_SCALAR).r;

        // 4) step() gives you a 0 or 1 mask per channel:
        //    if (color.rgb > threshold) -> 1, else -> 0
        vec4 dithered = step(bayer_threshold, color);

        // 5) mix back with the original color to soften the effect
        //    weight = 0 -> no dither, 1 -> full binary dither
        color = mix(color, dithered, dither_interpolation_weight);
    }
    COLOR = color;
}
