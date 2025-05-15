shader_type canvas_item;
render_mode blend_disabled;

uniform vec2 iResolution;
uniform sampler2D iChannel0: repeat_disable, filter_nearest;

#define WHITE vec4(1.0, 1.0, 1.0, 1.0)
#define BLACK vec4(0.0, 0.0, 0.0, 1.0)
#define IOI_ROW 0.0

#define IOI_BIN_COUNT       32.0
#define IOI_STRIP_WIDTH_PX  32.0

void fragment(){
    vec2 frag_coord = FRAGCOORD.xy;
    float x0 = iResolution.x - IOI_STRIP_WIDTH_PX;
    if (frag_coord.x < x0) {
        discard;
    }
    float y_norm   = (frag_coord.y - 0.5 * iResolution.y) / (0.5 * iResolution.y);
    float bin_f    = (y_norm * 0.5 + 0.5) * IOI_BIN_COUNT;
    float bin_idx  = floor(bin_f);
    if (bin_idx < 0.0 || bin_idx >= IOI_BIN_COUNT)
        discard;
    float amp = texture(iChannel0, vec2(bin_idx / IOI_BIN_COUNT, IOI_ROW)).r;
    float local_x = (frag_coord.x - x0) / IOI_STRIP_WIDTH_PX;
    COLOR = (local_x <= amp) ? WHITE : BLACK;
}

