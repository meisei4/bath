#[compute]
#version 450
#extension GL_KHR_shader_subgroup_basic : enable  

layout(local_size_x = 2, local_size_y = 2, local_size_z = 1) in;

layout(push_constant, std430) uniform PushConstants {
    vec2 iResolution; // at byte‐offset 0, occupies bytes 0..7
    float iTime;      // at byte‐offset 8, occupies bytes 8..11
    uint _pad;        // at byte‐offset 12, occupies bytes 12..15  
} push_constants;

#define COLLISION_MASK_SSBO_UNIFORM_BINDING 0
layout(set = 0, binding = COLLISION_MASK_SSBO_UNIFORM_BINDING, r32ui) writeonly uniform uimage2D collision_mask_ssbo;

float hash12(vec2 p) {
    vec3 p3  = fract(vec3(p.xyx) * .1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

vec2 hash22(vec2 p) {
    vec3 p3 = fract(vec3(p.xyx) * vec3(.1031, .1030, .0973));
    p3 += dot(p3, p3.yzx+33.33);
    return fract((p3.xx+p3.yz)*p3.zy);
}

vec2 grad(ivec2 z) {
    return hash22(vec2(z)*123.456) * 2.0 - 1.0;
}

float perlin_noise_iq( in vec2 p ) {
    ivec2 i = ivec2(floor( p ));
     vec2 f =       fract( p );
    vec2 u = smoothstep(0.0, 1.0, f);
    return mix( mix( dot( grad( i+ivec2(0,0) ), f-vec2(0.0,0.0) ),
                     dot( grad( i+ivec2(1,0) ), f-vec2(1.0,0.0) ), u.x),
                mix( dot( grad( i+ivec2(0,1) ), f-vec2(0.0,1.0) ),
                     dot( grad( i+ivec2(1,1) ), f-vec2(1.0,1.0) ), u.x), u.y);
}

#define M_PI     3.14159265358979323846
#define M_PI_4   0.785398163397448309616
#define TAU      (M_PI + M_PI)

#define NOISE_SCROLL_VELOCITY       vec2(0.0, 0.05)
#define GLOBAL_COORD_SCALAR         180.0

#define STRETCH_SCALAR_X            1.0
#define STRETCH_SCALAR_Y            2.0

#define NOISE_COORD_OFFSET          vec2(2.0, 0.0)
#define PERLIN_SOLID_THRESHOLD      -0.03

#define ENABLE_ROTATION
    #define ROTATION_ANGLE          -M_PI_4
    #define ROTATION_MATRIX         mat2(vec2(cos(ROTATION_ANGLE), -sin(ROTATION_ANGLE)), \
                                         vec2(sin(ROTATION_ANGLE),  cos(ROTATION_ANGLE)))

#define ENABLE_STRETCH_CORRECTION
    #define UNIFORM_STRETCH_CORRECTION_SCALAR    sqrt(2.0)

float sampleNoise(vec2 coord, float localNoiseScale, float iTime) {
    float x_displacement = iTime * NOISE_SCROLL_VELOCITY.x;
    float y_displacement = iTime * NOISE_SCROLL_VELOCITY.y;
    vec2 displaced_coordinate = vec2(
        coord.x + x_displacement,
        coord.y + y_displacement
    );
    vec2 scaled_coordinate = displaced_coordinate * GLOBAL_COORD_SCALAR;
    vec2 stretched_coordinate = vec2(
        scaled_coordinate.x * STRETCH_SCALAR_X,
        scaled_coordinate.y * STRETCH_SCALAR_Y
    );

    #ifdef ENABLE_COMPOSED_AFFINE_TRANSFORMATIONS
        vec2 stretch_corrected_and_rotated_coordinate = UNIFORM_STRETCH_CORRECTION_AND_ROTATION_COMPOSITION * stretched_coordinate;
        vec2 local_noise_scaled_coordinate = stretch_corrected_and_rotated_coordinate * localNoiseScale;
    #else
        #ifdef ENABLE_STRETCH_CORRECTION
            stretched_coordinate = UNIFORM_STRETCH_CORRECTION_SCALAR * stretched_coordinate;
        #endif
        #ifdef ENABLE_ROTATION
            stretched_coordinate = ROTATION_MATRIX * stretched_coordinate;
        #endif
        vec2 local_noise_scaled_coordinate = stretched_coordinate * localNoiseScale;
    #endif

    vec2 final_noise_coordinate = local_noise_scaled_coordinate - NOISE_COORD_OFFSET;
    float sampled_noise = perlin_noise_iq(final_noise_coordinate);
    return sampled_noise;
}

bool isSolidAtCoord(vec2 coord, float localNoiseScale, float iTime) {
    float noiseValue = sampleNoise(coord, localNoiseScale, iTime);
    return (noiseValue < PERLIN_SOLID_THRESHOLD);
}

#define PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR 6.0
#define PARALLAX_NEAR_SCALAR                        0.025

vec2 projectLayer(vec2 originalCoord, out float chosenNoiseScale) {
    chosenNoiseScale = PARALLAX_NEAR_SCALAR;
    originalCoord = originalCoord / (PARALLAX_PROJECTION_ASYMPTOTIC_DEPTH_SCALAR - originalCoord.y);
    return originalCoord;
}

vec2 projectTopLayerForParallax(vec2 normCoord, out float noiseScale) {
    return projectLayer(normCoord, noiseScale);
}

#define SOLID_REGION_BRIGHTNESS       0.9

vec3 getTerrainColor(bool isSolid, vec2 normCoord) {
    if (isSolid) {
        return vec3(SOLID_REGION_BRIGHTNESS);
    } else {
        return vec3(0.0);
    }
}

void main() {
    ivec2 gid = ivec2(gl_GlobalInvocationID.xy);
    if (gid.x >= int(push_constants.iResolution.x) ||
        gid.y >= int(push_constants.iResolution.y)) {
        return;
    }
    vec2 pixelCenter = vec2(gid) + vec2(0.5, 0.5);
    vec2 normCoord = vec2(
        (pixelCenter.x * 2.0 - push_constants.iResolution.x) / push_constants.iResolution.y,
        (pixelCenter.y * 2.0 - push_constants.iResolution.y) / push_constants.iResolution.y
    );
    float localNoiseScale;
    vec2  projectedTop = projectTopLayerForParallax(normCoord, localNoiseScale);
    bool  solidTop     = isSolidAtCoord(projectedTop, localNoiseScale, push_constants.iTime);
    vec3 terrainColor = getTerrainColor(solidTop, normCoord);
    float gammaCorrect = sqrt(terrainColor.r);

    // float maskValue = (gammaCorrect > 0.9) ? 1.0 : 0.0;
    // imageStore(collision_mask_ssbo, gid, vec4(maskValue, 0.0, 0.0, 0.0));
    
    uint maskValue = (gammaCorrect > 0.9) ? 1u : 0u;
    imageStore(collision_mask_ssbo, gid, uvec4(maskValue, 0u, 0u, 0u));
}
