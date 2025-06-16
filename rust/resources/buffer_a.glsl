#version 330
out vec4 fragColor;

uniform sampler2D iChannel0;
uniform vec2      iResolution;
uniform float     iTime;

const float DECAY_FACTOR = 0.95;
const float CIRCLE_OUTER_RADIUS = 0.08;
const float CIRCLE_INNER_RADIUS = 0.06;
const vec4  RED = vec4(1.0, 0.0, 0.0, 1.0);

void main() {
    vec2 uv = gl_FragCoord.xy / iResolution;
    vec4 prev = texture(iChannel0, uv) * DECAY_FACTOR;
    vec2  centre = 0.5 + vec2(cos(iTime), sin(iTime)) * 0.3;
    float mask   = smoothstep(CIRCLE_OUTER_RADIUS, CIRCLE_INNER_RADIUS, distance(uv, centre));
    fragColor = prev + mask * RED;
}
