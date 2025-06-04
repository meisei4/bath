//NOISE FUNKS
float hash12(vec2 p) // https://www.shadertoy.com/view/4djSRW Hash without Sine by Dave_Hoskins
{
    vec3 p3  = fract(vec3(p.xyx) * .1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

vec2 hash22(vec2 p) // https://www.shadertoy.com/view/4djSRW Hash without Sine by Dave_Hoskins
{
    vec3 p3 = fract(vec3(p.xyx) * vec3(.1031, .1030, .0973));
    p3 += dot(p3, p3.yzx+33.33);
    return fract((p3.xx+p3.yz)*p3.zy);
}

vec2 grad(ivec2 z)
{
    //return hash22(vec2(z)) * 2.0 - 1.0;
    return hash22(vec2(z)*123.456) * 2.0 - 1.0;
}

float perlin_noise_iq( in vec2 p ) // Perlin noise by inigo quilez - iq/2013   https://www.shadertoy.com/view/XdXGW8
{
    ivec2 i = ivec2(floor( p ));
     vec2 f =       fract( p );
    //vec2 u = f*f*(3.0-2.0*f); // feel free to replace by a quintic smoothstep instead
    vec2 u = smoothstep(0.0, 1.0, f);
    return mix( mix( dot( grad( i+ivec2(0,0) ), f-vec2(0.0,0.0) ),
                     dot( grad( i+ivec2(1,0) ), f-vec2(1.0,0.0) ), u.x),
                mix( dot( grad( i+ivec2(0,1) ), f-vec2(0.0,1.0) ),
                     dot( grad( i+ivec2(1,1) ), f-vec2(1.0,1.0) ), u.x), u.y);
}


//whacked-out voronoi still.. will come back to voronoi concepts in future studies
float voronoi_edge_fast(vec2 p) {
    vec2 ip = floor(p);
    vec2 fp = fract(p);
    float F1 = 1e9;
    float F2 = 1e9;
    vec2 OFFS[5] = vec2[5]( vec2( 0, 0),
                            vec2(-1, 0), vec2( 1, 0),
                            vec2( 0,-1), vec2( 0, 1) );
    for(int k = 0; k < 5; ++k) {
        vec2  n   = OFFS[k];
        vec2  seed = n + hash22(ip + n);
        vec2  d    = fp - seed;
        float d2   = dot(d, d);

        if(d2 < F1) { F2 = F1; F1 = d2; }
        else if(d2 < F2) { F2 = d2; }
    }
    return sqrt(F2) - sqrt(F1);
}

//APPLIED NOISE FUNKS
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

//#define ENABLE_ROTATION
    #define ROTATION_ANGLE          -M_PI_4
    //#define ROTATION_ANGLE        (-M_PI_4 * 0.5)
    #define ROTATION_MATRIX         mat2(vec2(cos(ROTATION_ANGLE), -sin(ROTATION_ANGLE)), \
                                         vec2(sin(ROTATION_ANGLE),  cos(ROTATION_ANGLE)))

//#define ENABLE_STRETCH_CORRECTION
    #define UNIFORM_STRETCH_CORRECTION_SCALAR    sqrt(2.0)
    /*
    #define UNIFORM_STRETCH_CORRECTION_SCALAR  (sqrt((pow(STRETCH_SCALAR_X, 2.0) \
                                                       + pow(STRETCH_SCALAR_Y, 2.0)) \
                                                       / 2.0))
   */
#define ENABLE_COMPOSED_AFFINE_TRANSFORMATIONS
    #define UNIFORM_STRETCH_CORRECTION_AND_ROTATION_COMPOSITION mat2(vec2(1.0,  1.0), \
                                                                     vec2(-1.0,  1.0))

#define UNIFORM_DEPTH_MARCH_FIRST_STEP_SCALAR                    0.0
//#define ENABLE_DEPTH_MARCH_FIRST_STEP_SCRAMBLING
    #define FIRST_STEP_SCRAMBLER_SCALAR                          123.456

//TODO: the following characters get complaints when imported into glsl compute shader and then imported into godot
// it works fine but just a lot of errors in the output because of this ascii table
//U+2500  ─   BOX DRAWINGS LIGHT HORIZONTAL
//U+2502  │   BOX DRAWINGS LIGHT VERTICAL
//U+252C  ┬   BOX DRAWINGS LIGHT DOWN AND HORIZONTAL
//U+253C  ┼   BOX DRAWINGS LIGHT VERTICAL AND HORIZONTAL

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

float sampleNoise(vec2 coord, float localNoiseScale, float iTime, int iFrameCount) {
    float x_displacement = iTime * NOISE_SCROLL_VELOCITY.x;
    //float y_displacement = iTime * NOISE_SCROLL_VELOCITY.y;
    // float x_displacement = TIME * NOISE_SCROLL_VELOCITY.x;
    // float y_displacement = TIME * NOISE_SCROLL_VELOCITY.y;
    float arbitrary_virtual_frame_rate = 1.0 / 60.0;
    float continuous_full = float(iFrameCount) * arbitrary_virtual_frame_rate
        * NOISE_SCROLL_VELOCITY.y
        * GLOBAL_COORD_SCALAR
        * STRETCH_SCALAR_Y
        * UNIFORM_STRETCH_CORRECTION_SCALAR;

    float discrete_full = floor(continuous_full);
    float y_displacement = discrete_full
                           / (GLOBAL_COORD_SCALAR * STRETCH_SCALAR_Y * UNIFORM_STRETCH_CORRECTION_SCALAR);
    // float continuous_full = iTime
    //     * NOISE_SCROLL_VELOCITY.y
    //     * GLOBAL_COORD_SCALAR
    //     * STRETCH_SCALAR_Y
    //     * UNIFORM_STRETCH_CORRECTION_SCALAR;
    // float discrete_full = floor(continuous_full);
    // float y_displacement = discrete_full / (GLOBAL_COORD_SCALAR * STRETCH_SCALAR_Y * UNIFORM_STRETCH_CORRECTION_SCALAR);
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

bool isSolidAtCoord(vec2 coord, float localNoiseScale, float iTime, int iFrameTime) {
    float noiseValue = sampleNoise(coord, localNoiseScale, iTime, iFrameTime);
    #ifdef USE_VORONOI
        return (noiseValue > VORONOI_SOLID_THRESHOLD);
    #else
        return (noiseValue < PERLIN_SOLID_THRESHOLD);
    #endif
}
