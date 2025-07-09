shader_type canvas_item;
render_mode blend_disabled;

#define ALPHA_OVERRIDE 1.0

uniform sampler2D iChannel0 : filter_nearest;

void fragment() {
    vec4 src_color = texture(iChannel0, UV);
    COLOR          = vec4(src_color.r, src_color.g, src_color.b, ALPHA_OVERRIDE);
}
