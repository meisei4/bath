#version 330

in vec3 vertexPosition;
in vec2 vertexTexCoord;

in vec3 vertexNormal;
in vec4 vertexColor;

out vec2 fragTexCoord;
out vec4 fragColor;

out vec2  vertexNormCoord;
out vec2  vertexTopLayerProjection;
out vec2  vertexProjectedOrigin;
out vec2  vertexProjectedStep;
out float projectionScaleFactor;
out vec2  vertexNoiseOrigin;
out vec2  vertexNoiseStep;

uniform mat4  mvp;
uniform vec2  iResolution;
uniform float parallaxDepth;
uniform float strideLength;
uniform float globalCoordinateScale;
uniform float iTime;

uniform mat2  rotationMatrix;
uniform vec2  noiseScrollVelocity;
uniform float uniformStretchCorrection;
uniform float stretchScalarY;
uniform vec2  noiseCoordinateOffset;
uniform float parallaxNearScale;

#define M_PI_4 0.785398163397448309616

void main() {
    fragTexCoord             = vertexTexCoord;
    fragColor                = vertexColor;
    vec2 fragCoord           = vertexTexCoord * iResolution;
    vertexNormCoord          = (fragCoord * 2.0 - iResolution) / iResolution.y;
    vertexTopLayerProjection = vertexNormCoord / (parallaxDepth - vertexNormCoord.y);
    projectionScaleFactor    = 1.0 / (parallaxDepth - vertexNormCoord.y);
    vertexProjectedOrigin    = (fragCoord * 2.0 - iResolution) / iResolution.y * projectionScaleFactor;
    vertexProjectedStep      = vec2(0.0, (1.0 / globalCoordinateScale) * (1.0 / (parallaxDepth - vertexNormCoord.y)));

    mat2 scaleStretch   = uniformStretchCorrection * mat2(1.0, 0.0, 0.0, stretchScalarY);
    mat2 rotationMatrix = mat2(vec2(cos(M_PI_4), -sin(M_PI_4)), vec2(sin(M_PI_4), cos(M_PI_4)));
    mat2 linearPart     = rotationMatrix * scaleStretch * (globalCoordinateScale * parallaxNearScale);
    vec2 timePan        = linearPart * (noiseScrollVelocity * iTime);
    vec2 totalOffset    = timePan - noiseCoordinateOffset * parallaxNearScale;
    vertexNoiseOrigin   = linearPart * vertexProjectedOrigin + totalOffset;
    vertexNoiseStep     = linearPart * vertexProjectedStep;

    gl_Position = mvp * vec4(vertexPosition, 1.0);
}