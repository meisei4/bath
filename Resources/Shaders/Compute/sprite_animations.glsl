#[compute]
#version 450
#extension GL_KHR_shader_subgroup_basic : enable   // not required, but handy later

// One thread per screen pixel
layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

// ──────────────────────────────
// Push-constants  (very small)
// ──────────────────────────────
layout(push_constant, std430) uniform Push {
    vec2 iResolution;      // viewport size in px
    uint sprite_count;     // active rows in the SSBO
} pc;

// ──────────────────────────────
// SSBO layout  (binding = 0)
// ──────────────────────────────
struct SpriteData {
    vec2  center_px;
    vec2  half_size_px;
    float altitude;
    float ascending;
    vec2  _pad;            // std430 → keeps 16-byte alignment
};

layout(set = 0, binding = 0, std430) readonly buffer Sprites {
    SpriteData sprites[];
};

// ──────────────────────────────
// Output image  (binding = 1)
// ──────────────────────────────
layout(r32f, set = 0, binding = 1) writeonly uniform image2D warp_mask;

// ──────────────────────────────
// Shared tilt/escape function
// ──────────────────────────────
float compute_escape_strength(in SpriteData s, in vec2 pix)
{
    // 1) quick AABB reject
    vec2 d = abs(pix - s.center_px);
    if (d.x > s.half_size_px.x || d.y > s.half_size_px.y)
        return 0.0;

    // 2) convert to normalised [-1..1] local sprite coords
    vec2 local = (pix - s.center_px) / s.half_size_px;

    // 3) perspective tilt — identical math to your fragment shader
    //    (factorised into its own helper in real code!)
    float jump_phase = s.altitude;                  // 0 → 1
    float tilt_phase = 1.0 - jump_phase;
    float max_angle  = 0.785398;                    // 45°  (could be push-const too)
    float angle      = tilt_phase * max_angle * -1.0;

    float cos_t = cos(angle);
    float sin_t = sin(angle);

    float L = (s.ascending > 0.5) ? local.y * 0.5 + 0.5
                                  : (1.0 - (local.y * 0.5 + 0.5));

    float depth     = L * sin_t;
    float divisor   = 1.0 + depth / 1.0;            // focal_length = 1
    float proj_y    = (L * cos_t) / divisor;

    /* Escape strength:
       grounded→0, apex→1, plus small bias if pixel is near the sprite’s “top”.
       Replace with any formula you need – this is just a demo.
    */
    return clamp(jump_phase * (1.0 - proj_y), 0.0, 1.0);
}

// ──────────────────────────────
// Main kernel
// ──────────────────────────────
void main()
{
    ivec2 ipix = ivec2(gl_GlobalInvocationID.xy);
    if (ipix.x >= int(pc.iResolution.x) || ipix.y >= int(pc.iResolution.y))
        return;                                     // out-of-bounds thread (at frame edges)

    vec2  pix      = vec2(ipix) + vec2(0.5);        // centre-of-pixel
    float escape   = 0.0;

    // linear scan — 64 sprites is tiny, so branchless loop is fine
    for (uint i = 0u; i < pc.sprite_count; ++i)
        escape = max(escape, compute_escape_strength(sprites[i], pix));

    imageStore(warp_mask, ipix, vec4(escape, 0.0, 0.0, 0.0));
}
