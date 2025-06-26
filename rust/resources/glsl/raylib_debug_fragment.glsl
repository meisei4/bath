#version 330

in vec2 fragTexCoord;
in vec4 fragColor;
// TODO: this is the proof of how the triangles are oriented in the quad:
// flat in vec4 fragColor;

out vec4 finalColor;

void main() { finalColor = fragColor; }
