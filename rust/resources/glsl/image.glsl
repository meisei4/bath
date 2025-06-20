#version 330
uniform sampler2D iChannel1;
uniform vec2 iResolution;
in vec2 fragTexCoord;
in vec4 fragColor;
out vec4 finalColor;
void main() {
    vec2 uv = fragTexCoord;
    finalColor = texture(iChannel1, uv);
}