#define ENABLE_WATER
    #define WATER_COLOR_R                      0.1
    #define WATER_COLOR_G                      0.7
    #define WATER_COLOR_B                      0.8
    #define WATER_DARKEN_MULTIPLIER            0.5
    #define WATER_DEPTH_DARKEN_DIVISOR         5.0
    #define WATER_STATIC_DARKEN_THRESHOLD      12

#define ENABLE_WATER_ANIMATION
    #define WATER_ANIM_DEPTH_MIN              8.0
    #define WATER_ANIM_DEPTH_MAX              20.0
    #define WATER_ANIM_COS_OFFSET             0.5
    #define WATER_ANIM_COS_AMPLITUDE          0.5
    #define WATER_ANIM_COS_FREQUENCY          0.1


// MISC COLOR
#define SKY_COLOR_R                        0.1
#define SKY_COLOR_G                        0.7
#define SKY_COLOR_B                        0.8
#define SKY_GRADIENT_STRENGTH              15.0
#define SOLID_REGION_BRIGHTNESS            0.9


//MISC MATH
#define pi 3.1415926
#define tau (pi+pi)
#define PI_4 0.785398163397448309615660845819875721


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


//TODO: whacked-out gpt-assisted voronoi??? I need to look into proper understanding of voronoi from other proper shaders
float voronoi_edge_fast(vec2 p) {
    vec2 ip = floor(p);
    vec2 fp = fract(p);

    float F1 = 1e9;
    float F2 = 1e9;

    vec2 OFFS[5] = vec2[5]( vec2( 0, 0),
                            vec2(-1, 0), vec2( 1, 0),
                            vec2( 0,-1), vec2( 0, 1) );
    for(int k = 0; k < 4; ++k) {
        vec2  n   = OFFS[k];
        vec2  seed = n + hash22(ip + n);
        vec2  d    = fp - seed;
        float d2   = dot(d, d);

        if(d2 < F1) { F2 = F1; F1 = d2; }
        else if(d2 < F2) { F2 = d2; }
    }
    return sqrt(F2) - sqrt(F1);
}
