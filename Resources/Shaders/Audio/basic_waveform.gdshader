shader_type canvas_item;
//TODO: alpha is dead channel, hope it doesnt fuck with stuff
//render_mode blend_disabled; // TODO you have to do this if you want to send feedback vals in the alpha channel idiot

//#iChannel0 "file://assets/audio/Hellion_v2.ogg"
//#iChannel0 "file://assets/audio/experiment.mp3"

uniform vec2 iResolution;
uniform sampler2D iChannel0;

#define WHITE vec4(1.0, 1.0, 1.0, 1.0)
#define BLACK vec4(0.0, 0.0, 0.0, 1.0)
#define WAVEFORM_ROW 1.0

//void mainImage(out vec4 frag_color, in vec2 frag_coord){
void fragment(){
    //vec2 uv = frag_coord.xy / iResolution.xy;
    vec2 uv = FRAGCOORD.xy / iResolution.xy;
    vec2 waveform_row_coords = vec2(uv.x, WAVEFORM_ROW);
    float waveform_amplitude_value = texture(iChannel0, waveform_row_coords).r; //R-channel is the only one i write to in the texture builder

    vec4 color = BLACK;
    if(uv.y < waveform_amplitude_value) {
        color = WHITE; // white for all values under the waveform y-value
    }

    //frag_color = color;
    COLOR = color;
}
