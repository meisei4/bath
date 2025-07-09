shader_type canvas_item;

uniform sampler2D iChannel0;
uniform vec2      iResolution;

void fragment() {
    vec2 uv = FRAGCOORD.xy / iResolution.xy;
    COLOR   = texture(iChannel0, uv);
}