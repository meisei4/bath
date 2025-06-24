#version 330

in vec3 vertexPosition;
in vec2 vertexTexCoord;
in vec4 vertexColor;

uniform vec2 iResolution;
uniform float uParDepth;
uniform float iTime;

out vec2 v_normCoord;
out vec2 v_noiseCoord;

void main() {
    vec2 n = vertexTexCoord * vec2(2.0 * iResolution.x / iResolution.y, 2.0) - vec2(iResolution.x / iResolution.y, 1.0);

    float w = uParDepth - n.y;
    gl_Position = vec4(n * w, 0.0, w);

    v_normCoord = n;
    v_noiseCoord = n / w;
}
