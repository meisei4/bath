#version 330
in vec3 vertexPosition;
in vec2 vertexTexCoord;
in vec4 vertexColor;
uniform mat4 matModel, matView, matProjection;
uniform vec2 iResolution;

out vec2 v_normCoord;

void main() {
    gl_Position = matProjection * matView * matModel * vec4(vertexPosition, 1);
    v_normCoord
    = vertexTexCoord * vec2(2.0 * iResolution.x / iResolution.y, 2.0) - vec2(iResolution.x / iResolution.y, 1.0);
}
