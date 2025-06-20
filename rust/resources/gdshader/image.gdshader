shader_type canvas_item;
uniform sampler2D iChannel1;
uniform vec2 iResolution;
void fragment() {
    vec2 uv = FRAGCOORD.xy / iResolution.xy;
    COLOR = texture(iChannel1, uv);
}