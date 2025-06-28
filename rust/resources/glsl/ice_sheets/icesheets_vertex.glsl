#version 330

in vec3 vertexPosition;
in vec2 vertexTexCoord;

in vec3 vertexNormal;
in vec4 vertexColor;

out vec2 fragTexCoord;
out vec4 fragColor;

out vec2  vertexNormCoord;
out float projectionScaleFactor;
out vec2  vertexNoiseOrigin;
out vec2  vertexNoiseStep;

uniform mat4  mvp;
uniform vec2  iResolution;
uniform float iTime;
uniform float parallaxDepth;
uniform float strideLength;
uniform float globalCoordinateScale;
uniform vec2  noiseScrollVelocity;
uniform float uniformStretchCorrection;
uniform float stretchScalarY;
uniform vec2  noiseCoordinateOffset;
uniform float parallaxNearScale;

const float PI             = 3.141592653589793;
const mat2  rotationMatrix = mat2(cos(-PI * 0.25), -sin(-PI * 0.25), sin(-PI * 0.25), cos(-PI * 0.25));
const float M_PI_4         = 0.785398163397448309616;

float affineScale(float d, float y) {
    float logTerm = log((d + 1.0) / (d - 1.0));
    float a       = 0.5 * logTerm;
    float b       = 1.5 * (d * logTerm - 2.0);
    return a + b * y;
}

float affineConstsA(float d) { return 0.5 * log((d + 1.0) / (d - 1.0)); }
float affineConstsB(float d) { return 1.5 * (d * log((d + 1.0) / (d - 1.0)) - 2.0); }

// TODO MAIN!!!!
//  1. move all projection to geometry/vertex positions just to see how it works
//  2. learn how to add more vertices for the non-affine projection (lol one vertex per pixel would be funny)
void main() {
    fragTexCoord    = vertexTexCoord;
    fragColor       = vertexColor;
    vec2 fragCoord  = vertexTexCoord * iResolution;
    vertexNormCoord = (fragCoord * 2.0 - iResolution) / iResolution.y;

    // TODO: UV AFFINE CORRECTION ONLY -> bilinear/quadratic somewhere in x still?? -> barycentric interpolation warp)
    // float scale              = affineScale(parallaxDepth, vertexNormCoord.y);
    // projectionScaleFactor    = scale;
    // vec2 vertexProjectedOrigin    = vertexNormCoord * scale;
    // vec2 vertexProjectedStep      = vec2(0.0, (1.0 / globalCoordinateScale) * scale);

    // TODO: UV AFFINE CORRECTION Y ONLY PROJECTION -> ZERO barycentric interpolation warp!?
    float a                    = affineConstsA(parallaxDepth);
    float b                    = affineConstsB(parallaxDepth);
    float kX                   = a;
    float scaleY               = a + b * vertexNormCoord.y;
    projectionScaleFactor      = scaleY;
    vec2 vertexProjectedOrigin = vec2(vertexNormCoord.x * kX, vertexNormCoord.y * scaleY);
    vec2 vertexProjectedStep   = vec2(0.0, (1.0 / globalCoordinateScale) * scaleY);

    // TODO: ORIGINAL UV NON-AFFINE CURVED PROJECITON -> barycentric interpolation warp
    // projectionScaleFactor    = 1.0 / (parallaxDepth - vertexNormCoord.y);
    // vec2 vertexProjectedOrigin    = (fragCoord * 2.0 - iResolution) / iResolution.y * projectionScaleFactor;
    // vec2 vertexProjectedStep      = vec2(0.0, (1.0 / globalCoordinateScale) * (1.0 / (parallaxDepth -
    // vertexNormCoord.y)));

    mat2 scaleStretch = uniformStretchCorrection * mat2(1.0, 0.0, 0.0, stretchScalarY);
    mat2 linearPart   = rotationMatrix * scaleStretch * (globalCoordinateScale * parallaxNearScale);
    vec2 timePan      = linearPart * (noiseScrollVelocity * iTime);
    vec2 totalOffset  = timePan - noiseCoordinateOffset;
    vertexNoiseOrigin = linearPart * vertexProjectedOrigin + totalOffset;
    vertexNoiseStep   = linearPart * vertexProjectedStep;

    gl_Position = mvp * vec4(vertexPosition, 1.0);
}
