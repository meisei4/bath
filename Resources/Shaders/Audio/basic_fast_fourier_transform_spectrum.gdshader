shader_type canvas_item;
render_mode blend_disabled;

uniform vec2 iResolution;
uniform sampler2D iChannel0;
//#iChannel0 "file://assets/audio/Hellion_v2.ogg"
//#iChannel0 "file://assets/audio/experiment.mp3"

#define total_fft_buffer_size_in_bins 512.0

#define WHITE vec4(1.0, 1.0, 1.0, 1.0)
#define BLACK vec4(0.0, 0.0, 0.0, 1.0)
#define FFT_ROW 0.0 //TODO: figure this out please

//void mainImage(out vec4 frag_color, in vec2 frag_coord) {
void fragment() {
    //vec2 uv = frag_coord.xy / iResolution.xy;
    vec2 uv = FRAGCOORD.xy / iResolution.xy;
    float cell_width = iResolution.x / total_fft_buffer_size_in_bins;
    //float bin_index = floor(frag_coord.x / cell_width);
    float bin_index = floor(FRAGCOORD.x / cell_width);
    //float local_x = mod(frag_coord.x, cell_width);
    float local_x = mod(FRAGCOORD.x, cell_width);
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
    //frag_color = color;
    COLOR = color;
}
