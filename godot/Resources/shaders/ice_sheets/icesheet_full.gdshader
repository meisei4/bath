shader_type canvas_item;
render_mode skip_vertex_transform;
render_mode blend_disabled;

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

varying vec2  fragTexCoord;
varying vec4  fragColor;
varying vec2  vertexNormCoord;
varying vec2  vertexTopLayerProjection;
varying vec2  vertexProjectedOrigin;
varying vec2  vertexProjectedStep;
varying float projectionScaleFactor;
varying vec2  vertexNoiseOrigin;
varying vec2  vertexNoiseStep;

const float M_PI_4         = 0.7853981633974483;
const mat2  rotationMatrix = mat2(vec2(cos(-M_PI_4), -sin(-M_PI_4)), vec2(sin(-M_PI_4), cos(-M_PI_4)));

float affineConstsA(float d) { return 0.5 * log((d + 1.0) / (d - 1.0)); }
float affineConstsB(float d) { return 1.5 * (d * log((d + 1.0) / (d - 1.0)) - 2.0); }

void vertex() {
    fragTexCoord             = UV;
    fragColor                = COLOR;
    vec2 fragCoord           = fragTexCoord * iResolution;
    vertexNormCoord          = (fragCoord * 2.0 - iResolution) / iResolution.y;
    float a                  = affineConstsA(parallaxDepth);
    float b                  = affineConstsB(parallaxDepth);
    float kX                 = a;
    float scaleY             = a + b * vertexNormCoord.y;
    vertexTopLayerProjection = vec2(vertexNormCoord.x * kX, vertexNormCoord.y * scaleY);
    projectionScaleFactor    = scaleY;
    vertexProjectedOrigin    = vertexTopLayerProjection;
    vertexProjectedStep      = vec2(0.0, (1.0 / globalCoordinateScale) * scaleY);
    mat2 scaleStretch        = uniformStretchCorrection * mat2(vec2(1.0, 0.0), vec2(0.0, stretchScalarY));
    mat2 linearPart          = rotationMatrix * scaleStretch * (globalCoordinateScale * parallaxNearScale);
    vec2 timePan             = linearPart * (noiseScrollVelocity * iTime);
    vec2 totalOffset         = timePan - noiseCoordinateOffset;
    vertexNoiseOrigin        = linearPart * vertexProjectedOrigin + totalOffset;
    vertexNoiseStep          = linearPart * vertexProjectedStep;
    VERTEX                   = (MODEL_MATRIX * vec4(VERTEX, 0.0, 1.0)).xy;
}

vec2 hash22(vec2 p) {
    vec3 p3 = fract(vec3(p.xyx) * vec3(.1031, .1030, .0973));
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.xx + p3.yz) * p3.zy);
}
vec2 grad(ivec2 z) { return hash22(vec2(z) * 123.456) * 2.0 - 1.0; }

float perlin_noise_iq(vec2 p) {
    ivec2 i   = ivec2(floor(p));
    vec2  f   = fract(p);
    vec2  u   = smoothstep(0.0, 1.0, f);
    float v00 = dot(grad(i + ivec2(0, 0)), f - vec2(0, 0));
    float v10 = dot(grad(i + ivec2(1, 0)), f - vec2(1, 0));
    float v01 = dot(grad(i + ivec2(0, 1)), f - vec2(0, 1));
    float v11 = dot(grad(i + ivec2(1, 1)), f - vec2(1, 1));
    return mix(mix(v00, v10, u.x), mix(v01, v11, u.x), u.y);
}

const float perlinSolidThreshold  = -0.03;
const vec3  waterColor            = vec3(0.1, 0.7, 0.8);
const float waterDarkenMultiplier = 0.5;
const float waterDepthDivision    = 9.0;
const int   waterStaticThreshold  = 12;

int marchDepthInProjectedNoiseSpace() {
    vec2 sampleProj = vertexNoiseOrigin;
    for (int depthStep = 0;; depthStep++) {
        if (perlin_noise_iq(sampleProj) < perlinSolidThreshold)
            return depthStep;
        sampleProj += vertexNoiseStep;
    }
}

void fragment() {
    int  depthSteps = marchDepthInProjectedNoiseSpace();
    bool isSolid    = perlin_noise_iq(vertexNoiseOrigin) < perlinSolidThreshold;
    vec3 baseColor  = isSolid ? vec3(0.9) : pow(waterColor, vec3(float(depthSteps) / waterDepthDivision));
    if (!isSolid && depthSteps > waterStaticThreshold) {
        baseColor *= waterDarkenMultiplier;
    }

    float solidness_alpha_mask_for_collision = isSolid ? 1.0 : 0.0;
    vec3  gammaCorrectedColor                = sqrt(baseColor);
    COLOR
        = vec4(gammaCorrectedColor.r, gammaCorrectedColor.g, gammaCorrectedColor.b, solidness_alpha_mask_for_collision);
}
