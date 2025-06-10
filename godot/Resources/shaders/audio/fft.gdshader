shader_type canvas_item;
render_mode blend_disabled;

uniform vec2 iResolution;
uniform sampler2D iChannel0: repeat_disable, filter_nearest;
#define total_fft_buffer_size_in_bins 512.0

#define WHITE vec4(1.0, 1.0, 1.0, 1.0)
#define BLACK vec4(0.0, 0.0, 0.0, 1.0)
#define FFT_ROW 0.0

void fragment() {
    vec2 frag_coord = FRAGCOORD.xy;
    vec2 uv = frag_coord.xy / iResolution.xy;
    float cell_width = iResolution.x / total_fft_buffer_size_in_bins;
    float bin_index = floor(frag_coord.x / cell_width);
    float local_x = mod(frag_coord.x, cell_width);
    float bar_width = cell_width - 1.0;
    vec4 color = BLACK;
    if (local_x <= bar_width) {
        float sample_x = (bin_index + 0.5) / total_fft_buffer_size_in_bins;
        vec2 sample_coord = vec2(sample_x, FFT_ROW);
        float amplitude = texture(iChannel0, sample_coord).r;
        if (uv.y < amplitude) {
            color = WHITE;
        }
    }
    COLOR = color;
}
