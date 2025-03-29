shader_type canvas_item;
uniform sampler2D iChannel0;
uniform vec2 iResolution;

//void mainImage(out vec4 frag_color, in vec2 frag_coord) {
void fragment() {
    //vec2 uv = frag_coord.xy / iResolution.xy;
    vec2 uv = FRAGCOORD.xy / iResolution.xy;
    //frag_color = texture(iChannel0, uv) / 10.0;
    COLOR = texture(iChannel0, uv) / 10.0;
}
