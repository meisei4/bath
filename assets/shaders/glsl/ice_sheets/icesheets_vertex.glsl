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
uniform float globalCoordinateScale;
uniform vec2  noiseScrollVelocity;
uniform float uniformStretchCorrection;
uniform float stretchScalarY;
uniform vec2  noiseCoordinateOffset;

uniform float a;
uniform float b;
uniform float invResolutionY;
uniform mat4  combinedLinearPart;

void main() {
    fragTexCoord = vertexTexCoord;
    fragColor    = vertexColor;
    vec2 aspectNorm
        = vertexTexCoord * vec2(2.0 * iResolution.x * invResolutionY, 2.0) - vec2(iResolution.x * invResolutionY, 1.0);
    float scaleY       = a + b * aspectNorm.y;
    vec2  uvOrig       = vec2(aspectNorm.x * a, aspectNorm.y * scaleY);
    vec2  uvStep       = vec2(0.0, scaleY / globalCoordinateScale);
    mat2  scaleStretch = uniformStretchCorrection * mat2(1.0, 0.0, 0.0, stretchScalarY);
    vec2  timePan      = (combinedLinearPart * vec4(scaleStretch * (noiseScrollVelocity * iTime), 0.0, 0.0)).xy;
    vec2  offset       = timePan - noiseCoordinateOffset;
    uvNoiseOrigin      = (combinedLinearPart * vec4(scaleStretch * uvOrig, 0.0, 0.0)).xy + offset;
    uvNoiseStep        = (combinedLinearPart * vec4(scaleStretch * uvStep, 0.0, 0.0)).xy;
    gl_Position        = mvp * vec4(vertexPosition, 1.0);
}