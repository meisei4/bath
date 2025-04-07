shader_type canvas_item;

uniform vec2 iResolution;
uniform sampler2D iChannel0;
//uniform float total_fft_buffer_size_in_bins;
#define total_fft_buffer_size_in_bins 512.0

#define WHITE vec4(1.0, 1.0, 1.0, 1.0)
#define BLACK vec4(0.0, 0.0, 0.0, 1.0)
#define FFT_ROW 0.0 //TODO: figure this out please
#define FREQ_BIN_BAR_WIDTH 1.0 // results in "1 pixel", in the case of a 512 width screen

void fragment() {
//void mainImage(out vec4 frag_color, in vec2 frag_coord){
    vec2 uv = FRAGCOORD.xy / iResolution.xy;
    //vec2 uv = frag_coord.xy / iResolution.xy;

    float pixel_position_x = uv.x * iResolution.x; //TODO: verbose but clear i guess
    float fft_bin_width_in_pixels = iResolution.x / total_fft_buffer_size_in_bins;
    float fft_bin_index = floor(pixel_position_x / fft_bin_width_in_pixels);
    float pixel_offset_in_bin = mod(pixel_position_x, fft_bin_width_in_pixels);
    vec4 color = BLACK;
    if (pixel_offset_in_bin <= FREQ_BIN_BAR_WIDTH) {
        //TODO: how does this work in shadertoy!!!
        //float texture_sample_x = (fft_bin_index + 0.5) / total_fft_buffer_size_in_bins;
        vec2 fft_row_coords = vec2(uv.x, FFT_ROW);
        float fft_amplitude_value = texture(iChannel0, fft_row_coords).r;
        if (uv.y < fft_amplitude_value) {
            color = WHITE;
        }
    }
    COLOR = color;
    //frag_color = color;
}


