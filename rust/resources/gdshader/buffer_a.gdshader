shader_type canvas_item;
render_mode blend_disabled;
uniform sampler2D iChannel0 : hint_screen_texture;
uniform vec2 iResolution;
uniform float iTime;
const float DECAY_FACTOR = 0.95;
const float CIRCLE_OUTER_RADIUS = 0.08;
const float CIRCLE_INNER_RADIUS = 0.06;
const vec4 RED = vec4(1.0, 0.0, 0.0, 1.0);
void fragment() {
    vec2 uv = UV;
    vec4 prev = texture(iChannel0, uv) * DECAY_FACTOR;
    vec2 center = vec2(0.5) + vec2(cos(iTime), sin(iTime)) * 0.3;
    float d = distance(uv, center);
    float mask = smoothstep(CIRCLE_OUTER_RADIUS, CIRCLE_INNER_RADIUS, d);
    COLOR = prev + mask * RED;
}