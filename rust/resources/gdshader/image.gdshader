shader_type canvas_item;
uniform sampler2D iChannel0;
void fragment() {
    vec2 uv = UV;
    COLOR = texture(iChannel0, uv);
}