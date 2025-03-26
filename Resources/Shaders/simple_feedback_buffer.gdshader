shader_type canvas_item;

uniform sampler2D iChannel0; //TODO: maybe call this iChannel0 or something like that
uniform float iTime;

const float DECAY_FACTOR = 0.985; // decay all fragments by 99.5% per frame

const float CIRCLE_OUTER_RADIUS = 0.15; // in UV dimensions (15% of the screen)
const float CIRCLE_INNER_RADIUS = 0.13; // this is only used to "soften the circle's edge"
const vec4 RED = vec4(1.0, 0.0, 0.0, 1.0);

//void mainImage(out vec4 frag_color, in vec2 frag_coord) {
void fragment() {
    //vec2 UV = frag_coord.xy / iResolution.xy;
    vec4 previous_frames_fragment_color = texture(iChannel0, UV);
    vec4 decayed_previous_frames_fragment_color = previous_frames_fragment_color * DECAY_FACTOR;

    vec2 screen_center = vec2(0.5, 0.5);  // (50%, 50%) of the screen
    float radius_of_motion = 0.3; // UV dimensions, i.e. 30% of the screen
    float speed = 2.0;
    float angle_of_rotation = iTime * speed;  // angle = time × speed???

    // Formula for positional offset from the center using sine and cosine???
    // x = radius_of_motion × sin(angle_of_rotation)
    // y = radius_of_motion × cos(angle_of_rotation)
    vec2 motion_offset = vec2(radius_of_motion * sin(angle_of_rotation), radius_of_motion * cos(angle_of_rotation));
    vec2 position_of_moving_circle = screen_center + motion_offset;
    float current_fragments_distance_from_moving_circle = distance(UV, position_of_moving_circle);
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

}
