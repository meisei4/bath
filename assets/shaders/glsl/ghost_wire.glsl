#version 100
precision mediump float;

varying vec2 fragTexCoord;
varying vec4 fragColor;

uniform float iTime;
uniform vec2  iResolution;

#define HALF                    0.5
#define GRID_SCALE              4.0
#define GRID_CELL_SIZE          (vec2(1.0) / GRID_SCALE)
#define GRID_ORIGIN_INDEX       vec2(0.0)
#define GRID_ORIGIN_OFFSET_CELLS vec2(2.0, 2.0)
#define GRID_ORIGIN_UV_OFFSET   ((GRID_ORIGIN_INDEX + GRID_ORIGIN_OFFSET_CELLS) * GRID_CELL_SIZE)

#define LIGHT_WAVE_SPATIAL_FREQ_X  8.0
#define LIGHT_WAVE_SPATIAL_FREQ_Y  8.0
#define LIGHT_WAVE_TEMPORAL_FREQ_X 80.0
#define LIGHT_WAVE_TEMPORAL_FREQ_Y 2.3
#define LIGHT_WAVE_AMPLITUDE_X     0.0
#define LIGHT_WAVE_AMPLITUDE_Y     0.1

#define UMBRAL_MASK_OUTER_RADIUS 0.40
#define UMBRAL_MASK_CENTER       vec2(HALF, HALF)

#define PI         3.14159265
#define INV_PI     (1.0 / PI)
#define INV_TWO_PI (1.0 / (2.0 * PI))
#define WIRE_ROT_SPEED      0.5
#define WIRE_SEGMENT_COUNT  8.0
#define WIRE_SMOOTH_EDGE    0.05
#define USE_HORIZONTAL_ROTATE 1.0

vec2 uv_to_grid_space(vec2 uv) {
    uv -= GRID_ORIGIN_UV_OFFSET;
    return uv * GRID_SCALE;
}

vec2 spatial_phase(vec2 gridCoords) {
    return vec2(
    gridCoords.y * LIGHT_WAVE_SPATIAL_FREQ_X,
    gridCoords.x * LIGHT_WAVE_SPATIAL_FREQ_Y
    );
}

vec2 temporal_phase(float t) {
    return vec2(
    t * LIGHT_WAVE_TEMPORAL_FREQ_X,
    t * LIGHT_WAVE_TEMPORAL_FREQ_Y
    );
}

vec2 add_phase(vec2 phase) {
    float dx = LIGHT_WAVE_AMPLITUDE_X * cos(phase.x);
    float dy = LIGHT_WAVE_AMPLITUDE_Y * sin(phase.y);
    return vec2(dx, dy);
}

vec4 white_circle(vec2 gridCoords) {
    float dist = length(gridCoords - UMBRAL_MASK_CENTER);
    return (dist < UMBRAL_MASK_OUTER_RADIUS)
    ? vec4(1.0)
    : vec4(0.0);
}

vec4 add_wireframe_overlay(vec4 baseColor, vec2 gridCoords, float time) {
    if (!all(equal(floor(gridCoords), vec2(0.0)))) return baseColor;
    vec2 cellUV = fract(gridCoords);
    vec2 delta  = cellUV - UMBRAL_MASK_CENTER;
    float R     = UMBRAL_MASK_OUTER_RADIUS;
    float rsq   = dot(delta, delta);
    if (rsq > R * R) return baseColor;
    float z = sqrt(R * R - rsq);
    vec3 n0 = normalize(vec3(delta, z));

    float angle = time * WIRE_ROT_SPEED;
    vec3 nh = normalize(vec3(
    cos(angle) * n0.x + sin(angle) * n0.z,
    n0.y,
    -sin(angle) * n0.x + cos(angle) * n0.z
    ));
    vec3 nv = normalize(vec3(
    n0.x,
    cos(angle) * n0.y - sin(angle) * n0.z,
    sin(angle) * n0.y + cos(angle) * n0.z
    ));
    vec3 n = mix(nv, nh, USE_HORIZONTAL_ROTATE);

    float u = atan(n.z, n.x) * INV_TWO_PI + HALF;
    float v = acos(n.y)   * INV_PI;
    float mu = abs(mod(u * WIRE_SEGMENT_COUNT, 1.0) - HALF);
    float mv = abs(mod(v * WIRE_SEGMENT_COUNT, 1.0) - HALF);
    float maskU = smoothstep(WIRE_SMOOTH_EDGE, 0.0, mu);
    float maskV = smoothstep(WIRE_SMOOTH_EDGE, 0.0, mv);
    float mask  = max(maskU, maskV);

    return mix(baseColor, vec4(0.0, 0.0, 0.0, 1.0), mask);
}

void main() {
    float time = iTime;
    vec2 gridCoords = uv_to_grid_space(fragTexCoord);
    vec2 phase = spatial_phase(gridCoords) + temporal_phase(time);
    gridCoords += add_phase(phase);
    vec4 white_circle = white_circle(gridCoords);
//    vec4 color     = add_wireframe_overlay(white_circle, gridCoords, time);
    gl_FragColor = color;
    gl_FragColor.a = 1.0;
}
