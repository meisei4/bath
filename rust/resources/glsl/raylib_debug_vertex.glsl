#version 330

in vec3 vertexPosition;
in vec2 vertexTexCoord;

in vec3 vertexNormal;
in vec4 vertexColor;

out vec2 fragTexCoord;
out vec4 fragColor;
// TODO: this is the proof of how the triangles are oriented in the quad:
// flat out vec4 fragColor;

uniform mat4 mvp;

const vec4 RED    = vec4(1.0, 0.0, 0.0, 1.0);
const vec4 GREEN  = vec4(0.0, 1.0, 0.0, 1.0);
const vec4 BLUE   = vec4(0.0, 0.0, 1.0, 1.0);
const vec4 YELLOW = vec4(1.0, 1.0, 0.0, 1.0);

// TODO: To prove that there are only 4 vertices, these two colors will not show up anywhere
const vec4 MAGENTA = vec4(1.0, 0.0, 1.0, 1.0);
const vec4 CYAN    = vec4(0.0, 1.0, 1.0, 1.0);

const vec4 palette[6] = vec4[6](RED, GREEN, BLUE, YELLOW, MAGENTA, CYAN);

void main() {
    fragTexCoord = vertexTexCoord;
    // TODO: when gl_VertexID goes from 0~3 it gets RED, GREEN, BLUE, YELLOW
    // and then when it tries to get a 5th vertex it finds nothing
    fragColor   = palette[gl_VertexID % 6];
    gl_Position = mvp * vec4(vertexPosition, 1.0);
}
