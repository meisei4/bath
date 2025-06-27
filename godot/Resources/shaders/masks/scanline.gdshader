shader_type canvas_item;
render_mode blend_disabled;

uniform sampler2D iChannel0;
uniform vec2      iResolution;

void fragment() {
    float u = FRAGCOORD.x / iResolution.x;
    float v = 1.0 - (FRAGCOORD.y / iResolution.y);
    COLOR   = texture(iChannel0, vec2(u, v));
}
