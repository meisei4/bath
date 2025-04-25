//APPLIED NOISE FUNKS
#include "res://Resources/Shaders/Glacier/noise_pure.gdshaderinc"
//TRANSCENDENTALS
#define M_PI     3.14159265358979323846
#define M_PI_4   0.785398163397448309616
#define TAU      (M_PI + M_PI)

#define NOISE_SCROLL_VELOCITY       vec2(0.0, 0.05)
#define GLOBAL_COORD_SCALAR         180.0

#define STRETCH_SCALAR_X            1.0
#define STRETCH_SCALAR_Y            2.0

#define NOISE_COORD_OFFSET          vec2(2.0, 0.0)
#define PERLIN_SOLID_THRESHOLD      -0.03
//#define USE_VORONOI
    #define VORONOI_SOLID_THRESHOLD 0.34

#define ENABLE_ROTATION
    #define ROTATION_ANGLE          -M_PI_4
    //#define ROTATION_ANGLE        (-M_PI_4 * 0.5)
    #define ROTATION_MATRIX         mat2(vec2(cos(ROTATION_ANGLE), -sin(ROTATION_ANGLE)), \
                                         vec2(sin(ROTATION_ANGLE),  cos(ROTATION_ANGLE)))

#define ENABLE_STRETCH_CORRECTION
    #define UNIFORM_STRETCH_CORRECTION_SCALAR    sqrt(2.0)
    //#define UNIFORM_STRETCH_CORRECTION_SCALAR  (sqrt((pow(STRETCH_SCALAR_X, 2.0) \
    //                                                   + pow(STRETCH_SCALAR_Y, 2.0)) \
    //                                                   / 2.0))

//#define ENABLE_COMPOSED_AFFINE_TRANSFORMATIONS
    #define UNIFORM_STRETCH_CORRECTION_AND_ROTATION_COMPOSITION mat2(vec2(1.0,  1.0), \
                                                                     vec2(-1.0,  1.0))

#define UNIFORM_DEPTH_MARCH_FIRST_STEP_SCALAR                    0.0
#define ENABLE_DEPTH_MARCH_FIRST_STEP_SCRAMBLING
    #define FIRST_STEP_SCRAMBLER_SCALAR                          123.456

//DEPTH MARCH STEP SCRAMBLING EXPLANATION:
//DISABLED:
//───────── slice 0 ─────────   <- all fragments hit the same slice first
//───────── slice 1 ─────────
//───────── slice 2 ─────────

//ENABLED:
//─────┬──────────┬─────────
//0.2  │  0.6     │ 0.95        <- each fragment starts somewhere inside slice 0
//─────┼──────────┼─────────
//1.2  │  1.6     │ 1.95
//─────┼──────────┼─────────
//2.2  │  2.6     │ 2.95

float sampleNoise(vec2 coord, float localNoiseScale, float iTime) {
    float x_displacement = iTime * NOISE_SCROLL_VELOCITY.x;
    float y_displacement = iTime * NOISE_SCROLL_VELOCITY.y;
    //float x_displacement = TIME * NOISE_SCROLL_VELOCITY.x;
    //float y_displacement = TIME * NOISE_SCROLL_VELOCITY.y;
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
    #ifdef USE_VORONOI
        float sampled_noise = voronoi_edge_fast(final_noise_coordinate);
    #else
        float sampled_noise = perlin_noise_iq(final_noise_coordinate);
    #endif
    return sampled_noise;
}

bool isSolidAtCoord(vec2 coord, float localNoiseScale, float iTime) {
    float noiseValue = sampleNoise(coord, localNoiseScale, iTime);
    #ifdef USE_VORONOI
        return (noiseValue > VORONOI_SOLID_THRESHOLD);
    #else
        return (noiseValue < PERLIN_SOLID_THRESHOLD);
    #endif
}