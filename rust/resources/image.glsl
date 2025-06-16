#version 330
out vec4 fragColor;
uniform sampler2D iChannel0;
uniform vec2      iResolution;

void main() {
    vec2 uv = gl_FragCoord.xy / iResolution;
    fragColor = texture(iChannel0, uv);
}
