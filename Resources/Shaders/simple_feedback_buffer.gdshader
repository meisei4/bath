//TODO: see shadertoy discrepancies with visual somethign about precision in data types?!?!?!?? no idea https://thebookofshaders.com/03/
shader_type canvas_item;

uniform sampler2D iChannel0;
uniform vec3 iMouse;
uniform vec2 iResolution;
const float DECAY_FACTOR = 0.985;

const float CIRCLE_OUTER_RADIUS = 0.08;
const float CIRCLE_INNER_RADIUS = 0.06;
const vec4 RED = vec4(1.0, 0.0, 0.0, 1.0);

//void mainImage(out vec4 frag_color, in vec2 frag_coord) {
void fragment() {
    //vec2 UV = frag_coord.xy / iResolution.xy;
    vec4 previous_frames_fragment_color = texture(iChannel0, UV);
    //in godot Limited texture precision (8/16-bit) and purely additive blending can prevent very small values from decaying to zero causing a persistent color burn.
    //Thus we combine multiplicative decay with a subtractive clamp to force near-zero values to black.
    vec4 decayed_previous_frames_fragment_color = previous_frames_fragment_color * DECAY_FACTOR - vec4(0.005);
    decayed_previous_frames_fragment_color = max(decayed_previous_frames_fragment_color, vec4(0.0)); // Clamp to 0
    vec2 mouse_uv = iMouse.xy / iResolution.xy;
    if (iMouse.z > 0.0){
        float current_fragments_distance_from_moving_circle = distance(UV, mouse_uv);
        // Create a soft-edged circle using smoothstep.
        // - If current fragment coordinates are within 13% screen-sized distance to the center of the circle
        //      - smoothstep returns 1.0 -> full strength mask will be applied to RED
        // - As the fragment coordinates get more than 13% radius away from the center of the circle towards 15%
        //      - the normalized multiplier/smoothstep mask [0, 1.0] smoothly fades to 0.0 -> weaker mask applied to RED
        // - Once a the fragment coordinates are further than 15% radius away
        //      - smoothstep reaches 0.0 → ZERO strength applied to red, i.e. NO MORE RED
        float mask_at_circle_edge = smoothstep(CIRCLE_OUTER_RADIUS, CIRCLE_INNER_RADIUS, current_fragments_distance_from_moving_circle);
        vec4 current_frames_fragment_color = mask_at_circle_edge * RED;
        // add the decayed value of the previous frames fragment color with the current frames calculated fragment color
        COLOR = decayed_previous_frames_fragment_color + current_frames_fragment_color;
        //frag_color = decayed_previous_frames_fragment_color + current_frames_fragment_color;
    } else {
        COLOR = decayed_previous_frames_fragment_color;
        //frag_color = decayed_previous_frames_fragment_color;
    }
}
