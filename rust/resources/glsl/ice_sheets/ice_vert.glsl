#version 330

in vec3 vertexPosition;
in vec2 vertexTexCoord;
in vec4 vertexColor;

uniform mat4 matModel, matView, matProjection;
uniform vec2 iResolution;

uniform float iTime;
uniform float uParDepth;

out vec2 v_normCoord;
out vec2 v_projCoord;

void main() {
    gl_Position = matProjection * matView * matModel * vec4(vertexPosition, 1);
    vec2 n = vertexTexCoord * vec2(2.0 * iResolution.x / iResolution.y, 2.0) - vec2(iResolution.x / iResolution.y, 1.0);
    v_normCoord = n;
    v_projCoord = n / (uParDepth - n.y);
}
