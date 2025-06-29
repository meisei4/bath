#version 330

in vec3 vertexPosition;
in vec2 vertexTexCoord;

in vec3 vertexNormal;
in vec4 vertexColor;

out vec2 fragTexCoord;
out vec4 fragColor;

out vec2 uvNoiseOrigin;
out vec2 uvNoiseStep;

uniform mat4  mvp;
uniform vec2  iResolution;
uniform float iTime;
uniform float parallaxDepth;
uniform float globalCoordinateScale;
uniform vec2  noiseScrollVelocity;
uniform float uniformStretchCorrection;
uniform float stretchScalarY;
uniform vec2  noiseCoordinateOffset;
uniform float parallaxNearScale;

const float PI             = 3.141592653589793;
const mat2  rotationMatrix = mat2(cos(-PI * 0.25), -sin(-PI * 0.25), sin(-PI * 0.25), cos(-PI * 0.25));
const float M_PI_4         = 0.785398163397448309616;

void main() {
    fragTexCoord       = vertexTexCoord;
    fragColor          = vertexColor;
    vec2 vPixelCoord   = vertexTexCoord * iResolution;
    vec2 vAspectNormal = (vPixelCoord * 2.0 - iResolution) / iResolution.y;

    float a                 = 0.5 * log((parallaxDepth + 1.0) / (parallaxDepth - 1.0));
    float b                 = 1.5 * (parallaxDepth * log((parallaxDepth + 1.0) / (parallaxDepth - 1.0)) - 2.0);
    float kX                = a;
    float scaleY            = a + b * vAspectNormal.y;
    vec2  uvProjectedOrigin = vec2(vAspectNormal.x * kX, vAspectNormal.y * scaleY);
    vec2  uvProjectedStep   = vec2(0.0, (1.0 / globalCoordinateScale) * scaleY);

    mat2 scaleStretch = uniformStretchCorrection * mat2(1.0, 0.0, 0.0, stretchScalarY);
    mat2 linearPart   = rotationMatrix * scaleStretch * (globalCoordinateScale * parallaxNearScale);
    vec2 timePan      = linearPart * (noiseScrollVelocity * iTime);
    vec2 totalOffset  = timePan - noiseCoordinateOffset;
    uvNoiseOrigin     = linearPart * uvProjectedOrigin + totalOffset;
    uvNoiseStep       = linearPart * uvProjectedStep;

    gl_Position = mvp * vec4(vertexPosition, 1.0);
}
