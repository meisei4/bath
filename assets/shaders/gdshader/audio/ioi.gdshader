shader_type canvas_item;
render_mode blend_disabled;

uniform vec2      iResolution;
uniform sampler2D iChannel0 : repeat_disable, filter_nearest;

#define TEMPOGRAM_BIN_COUNT 32.0
#define WHITE vec4(1.0)
#define ONSET_ROW_V 0.0
#define BPM_ROW_V 1.0

void fragment() {
    vec2  fc    = FRAGCOORD.xy;
    float col_w = iResolution.x / TEMPOGRAM_BIN_COUNT;
    float col   = floor(fc.x / col_w);
    if (col < 0.0 || col >= TEMPOGRAM_BIN_COUNT)
        discard;

    float s_x    = (col + 0.5) / TEMPOGRAM_BIN_COUNT;
    float half_h = iResolution.y * 0.5;

    if (fc.y >= half_h) {
        float onset = texture(iChannel0, vec2(s_x, ONSET_ROW_V)).r;
        if (onset > 0.5)
            COLOR = WHITE;
        else
            discard;
    }

    if (col == TEMPOGRAM_BIN_COUNT - 1.0) {
        float norm_bpm = texture(iChannel0, vec2(s_x, BPM_ROW_V)).r;
        float y_target = (1.0 - norm_bpm) * half_h;
        if (abs(fc.y - y_target) < 1.0)
            COLOR = WHITE;
        else
            discard;
    } else {
        discard;
    }
}
