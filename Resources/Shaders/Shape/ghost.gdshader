// this is a verbose, constant-driven rewrite with “smooth” ASCII diagrams and detailed explanations
shader_type canvas_item;
render_mode blend_disabled; //THIS WILL MAYBE MAKE THE ALPHA CHANNEL 1.0 assignment not needed/optimized???

uniform vec2   iResolution;
//uniform float  iTime;
uniform sampler2D iChannel0: filter_nearest_mipmap, repeat_enable;

const float SCALE             = 2.0;                     // zoom factor for tile grid
const vec2  TILE_SIZE         = vec2(1.0) / SCALE;       // size of one tile in UV space
const vec2  TARGET_TILE       = vec2(0.0, 0.0);           // base tile index
const vec2  MOVE_TILE         = vec2(1.33, 0.5);          // offset tile index
const vec2  ORIGIN            = (TARGET_TILE + MOVE_TILE) * TILE_SIZE;

const float DX_AMP            = 0.02;    // horizontal wave amplitude
const float DX_FREQ           = 6.0;     // horizontal wave spatial frequency
const float DX_SPEED          = 80.0;   // horizontal wave temporal frequency

const float DY_AMP            = 0.10;    // vertical wave amplitude
const float DY_FREQ           = 8.0;     // vertical wave spatial frequency
const float DY_SPEED          = 2.3;     // vertical wave temporal frequency

const float CIRCLE_RADIUS     = 0.35;    // radius of circular mask
const float EDGE_SOFT         = 0.01;    // softness width for smooth edge

const float BAYER_SCALE       = 8.0;
const float DITHER_WEIGHT     = 0.75;

//void mainImage(out vec4 frag_color, in vec2 frag_coord) {
void fragment() {
    vec2 frag_coord = UV * iResolution;
    vec2 uv        = frag_coord / iResolution.yy;
    vec2 local = (uv - ORIGIN) * SCALE;
    float t = TIME;
    //float t = iTime;
    float x_jittered_time = t * DX_SPEED;
    float y_jittered_time = t * DY_SPEED;

    float dx = DX_AMP * cos(local.y * DX_FREQ + uv.y + x_jittered_time );
    float dy = DY_AMP * sin(local.x * DY_FREQ + y_jittered_time);
    local.x += dx;
    local.y += dy;

    local.xy += sin(y_jittered_time + local.yx)*.3;

    float distance_to_local = length(local - vec2(0.5));
    float mask  = smoothstep(CIRCLE_RADIUS, CIRCLE_RADIUS - EDGE_SOFT, distance_to_local);

    vec2 centerOffset = vec2(
        DX_AMP * cos(0.9 * DX_FREQ + uv.y + x_jittered_time),
        DY_AMP * cos(0.2 * DY_FREQ + y_jittered_time)
    );
    vec2 shapeCenter = vec2(0.5) + centerOffset;
    vec2 lightPos    = shapeCenter + vec2(-CIRCLE_RADIUS/2.5, -CIRCLE_RADIUS);
    float lightD     = length(local - lightPos);
    float shade      = smoothstep(0.1, CIRCLE_RADIUS, lightD * 0.5);
    float bayer_threshold = texture(iChannel0, frag_coord / BAYER_SCALE).r;
    vec4 masked_color = vec4(mask * shade);
    vec4 dithered_color = step(bayer_threshold, masked_color);
    vec4 src_color = mix(masked_color, dithered_color, DITHER_WEIGHT);

    //src_color.a = 1.0;
    //frag_color = src_color;
    COLOR = src_color;
}
