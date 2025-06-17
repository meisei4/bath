#version 330
in vec2 fragTexCoord;
in vec4 fragColor;

out vec4 finalColor;

uniform sampler2D iChannel0;

void main() {
    vec2 uv       = fragTexCoord;
    finalColor    = texture(iChannel0, uv);
}
