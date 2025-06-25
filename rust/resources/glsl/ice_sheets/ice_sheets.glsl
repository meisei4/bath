#version 330

vec2 hash22(vec2 p) {
    vec3 p3 = fract(vec3(p.xyx) * vec3(.1031, .1030, .0973));
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.xx + p3.yz) * p3.zy);
}

vec2 grad(ivec2 z) { return hash22(vec2(z) * 123.456) * 2.0 - 1.0; }

float perlin_noise_iq(vec2 p) {
    ivec2 i = ivec2(floor(p));
    vec2  f = fract(p);
    vec2  u = smoothstep(0.0, 1.0, f);

    float v00 = dot(grad(i + ivec2(0, 0)), f - vec2(0.0, 0.0));
    float v10 = dot(grad(i + ivec2(1, 0)), f - vec2(1.0, 0.0));
    float v01 = dot(grad(i + ivec2(0, 1)), f - vec2(0.0, 1.0));
    float v11 = dot(grad(i + ivec2(1, 1)), f - vec2(1.0, 1.0));

    return mix(mix(v00, v10, u.x), mix(v01, v11, u.x), u.y);
}

uniform vec2  iResolution;
uniform float iTime;
in vec2       fragTexCoord;
out vec4      finalColor;

const float PI                       = 3.141592653589793;
const vec2  noiseScrollVelocity      = vec2(0.0, 0.1);
const float globalCoordinateScale    = 180.0;
const float stretchScalarY           = 2.0;
const vec2  noiseCoordinateOffset    = vec2(2.0, 0.0);
const float perlinSolidThreshold     = -0.03;
const mat2  rotationMatrix           = mat2(cos(-PI * 0.25), -sin(-PI * 0.25), sin(-PI * 0.25), cos(-PI * 0.25));
const float uniformStretchCorrection = 1.41421356237; // sqrt(2)

const vec3  waterColor            = vec3(0.1, 0.7, 0.8);
const float waterDarkenMultiplier = 0.5;
const float waterDepthDivision    = 9.0;
const int   waterStaticThreshold  = 12;

const float parallaxNearScale     = 0.025;
const float parallaxDepthDistance = 6.0;
const float strideLength          = 1.0;

float samplePerlinNoise(vec2 coordinate) {
    vec2 scrollPosition    = (coordinate + iTime * noiseScrollVelocity) * globalCoordinateScale;
    vec2 stretchedPosition = uniformStretchCorrection * vec2(scrollPosition.x, scrollPosition.y * stretchScalarY);
    vec2 rotatedPosition   = rotationMatrix * stretchedPosition;
    return perlin_noise_iq(rotatedPosition * parallaxNearScale - noiseCoordinateOffset);
}

int marchDepth(vec2 normalizedCoordinate) {
    for (int depthStep = 0;; depthStep++) {
        vec2 samplePosition = normalizedCoordinate + vec2(0.0, float(depthStep) / globalCoordinateScale) * strideLength;
        vec2 projectedPosition = samplePosition / (parallaxDepthDistance - samplePosition.y);
        if (samplePerlinNoise(projectedPosition) < perlinSolidThreshold || samplePosition.y >= iResolution.y) {
            return depthStep;
        }
    }
}

void main() {
    vec2 screenPosition       = fragTexCoord * iResolution;
    vec2 normalizedCoordinate = (screenPosition * 2.0 - iResolution) / iResolution.y;
    vec2 topProjection        = normalizedCoordinate / (parallaxDepthDistance - normalizedCoordinate.y);
    int  depthSteps           = marchDepth(normalizedCoordinate);
    bool isSolid              = samplePerlinNoise(topProjection) < perlinSolidThreshold ? true : false;
    vec3 baseColor            = isSolid ? vec3(0.9) : pow(waterColor, vec3(float(depthSteps) / waterDepthDivision));
    if (!isSolid && depthSteps > waterStaticThreshold) {
        baseColor *= waterDarkenMultiplier;
    }
    finalColor = vec4(sqrt(baseColor), 1.0);
}
