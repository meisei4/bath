#version 330
in vec2 fragTexCoord;
in vec4 fragColor;

out vec4 finalColor;

uniform sampler2D iChannel0;
uniform float        iTime;

const float DECAY_FACTOR       = 0.95;
const float CIRCLE_INNER_RADIUS = 0.06;
const float CIRCLE_OUTER_RADIUS = 0.08;
const vec4  RED                = vec4(1.0, 0.0, 0.0, 1.0);

void main() {
    vec2 uv    = fragTexCoord;
    vec4 prev  = texture(iChannel0, uv) * DECAY_FACTOR;
    vec2 center = 0.5 + vec2(cos(iTime), sin(iTime)) * 0.3;
    float d    = distance(uv, center);
    float mask = 1.0 - smoothstep(CIRCLE_INNER_RADIUS, CIRCLE_OUTER_RADIUS, d);
    finalColor = prev + mask * RED;
}
