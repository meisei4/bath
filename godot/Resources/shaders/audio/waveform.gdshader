shader_type canvas_item;
render_mode blend_disabled;

uniform vec2      iResolution;
uniform sampler2D iChannel0;

#define WHITE vec4(1.0, 1.0, 1.0, 1.0)
#define BLACK vec4(0.0, 0.0, 0.0, 1.0)
#define WAVEFORM_ROW 0.0

void fragment() {
    vec2  uv                       = FRAGCOORD.xy / iResolution.xy;
    vec2  waveform_row_coords      = vec2(uv.x, WAVEFORM_ROW);
    float waveform_amplitude_value = texture(iChannel0, waveform_row_coords).r;
    vec4  color                    = BLACK;
    if (uv.y < waveform_amplitude_value) {
        color = WHITE;
    }
    COLOR = color;
}
