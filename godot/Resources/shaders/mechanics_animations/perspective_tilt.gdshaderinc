#define MIDPOINT_UV 0.5
#define NEGATE_TILT_ANGLE -1.0
#define MAXIMUM_TILT_ANGLE_ACHIEVED_AT_IMMEDIATE_ASCENSION_AND_FINAL_DESCENT 0.785398
#define FOCAL_LENGTH 1.0
#define DISCARD_PIXELS_OUTSIDE_OF_ALTERED_UV_BOUNDS_FRAG(uv, texel_size) \
    if ((uv).x < (texel_size).x \
        || (uv).x > 1.0 - (texel_size).x \
        || (uv).y < (texel_size).y \
        || (uv).y > 1.0 - (texel_size).y) \
        discard;

void compute_perspective_tilt(
    in vec2 uv,
    in float altitude_normal,
    in bool ascending,
    in float max_tilt_angle,
    out vec2 altered_uv,
    out float perspective_tilt //TODO: this is unused in the frag shader, only used in compute shader

) {
    // 1) normalized 0->1 fraction through jump cycle (0 = lift‑off/land, 1 = apex, back to 0 for landing)
    float jump_phase_progress = altitude_normal;
    // 2) how much ground‑tilt remains (1 = full tilt on ground, 0 = no tilt at apex)
    float sprite_tilt_phase_progress = 1.0 - jump_phase_progress;
    // 3) Calculate tilt angle based on jump progress (0 at apex, max at ground)
    // 3) TODO: this is still hurting my head
    float current_tilt_angle = sprite_tilt_phase_progress * max_tilt_angle;
    // 4) Negate to tilt sprite toward the screen (pseudo-3D lean)
    // 4) TODO: idk why but it needs to happen... because of the mess i got myself into up until not
    current_tilt_angle = NEGATE_TILT_ANGLE * current_tilt_angle;
    // 5) horizontal UV offset from center (0.5)
    float midpoint_uv_x_of_sprite_quad = uv.x - MIDPOINT_UV;
    /* introducing... highscool trig... dang:
        Ascending (pivot at bottom of sprite)
                         Top of sprite
                               *
                              /|
                             / |
                            /  |
                           /   |   ← depth = L × sin(θ)
                          /    |
                         /     |
                        /θ     |
            Pivot o────/───────o──→ screen
                 Adjacent = L × cos(θ)
                 L = uv.y (distance from bottom to current fragment)
                - cos(θ) gives the *visible vertical height* on screen after tilt.
                  Think of it as compressing the height as the sprite leans back.
                - sin(θ) gives the *depth offset* — how far the tip of the sprite moves
                  into or out of the screen (used for perspective scaling).

            Descending (pivot at top of sprite)
                   *
                   |\
                   | \
                   |  \
                   |   \    ← depth = L × sin(θ)
                   |    \
                   |     \
                   |θ     \
            screen o───────* Pivot
                            Top of sprite
                 Adjacent = L × cos(θ)
                 L = 1.0 - uv.y (distance from top to current fragment)
                - cos(θ) again gives the vertical squish from tilt.
                - sin(θ) gives the pseudo-depth as if the sprite is leaning forward.
            In both cases:
                - cos(θ) = adjacent / hypotenuse → visible vertical compression
                - sin(θ) = opposite / hypotenuse → depth shift for perspective */
    float cosine_of_tilt_angle = cos(current_tilt_angle);
    float sine_of_tilt_angle   = sin(current_tilt_angle);
    float L = uv.y;
    //float L = ascending ? uv.y : 1.0 - uv.y;
    if (!ascending) {
        // Flip the pivot basis for the tilt - switch from bottom to top.
        L = 1.0 - uv.y;
    }
    // 6) sprite_quad_height_after_tilt_applied = L × cos(θ)
    float sprite_quad_height_after_tilt_applied = L * cosine_of_tilt_angle;
    // 7) sprite_quad_depth_after_tilt = L × sin(θ)
    float sprite_quad_depth_after_tilt = L * sine_of_tilt_angle;
    // 8) Build asymptotic perspective divisor: (1 + depth/f)
    //     - depth > 0 (farther) → divisor > 1 → compress
    //     - depth < 0 (closer) → divisor < 1 → expand
    float perspective_asymptotic_scalar =
        1.0 + sprite_quad_depth_after_tilt
                        /
                  FOCAL_LENGTH;
    // 9) Foreshortened vertical position = (L cos θ) / (1 + (L sin θ)/f)
    float projected_y_vertically_from_perspective_tilt =
            sprite_quad_height_after_tilt_applied
                            /
               perspective_asymptotic_scalar;
    // 10) Apply same perspective scale to horizontal offset
    float projected_x_horizontal_squish_from_perspective_tilt =
            midpoint_uv_x_of_sprite_quad
                        /
            perspective_asymptotic_scalar;

    // 11) Convert projected offsets back into standard UV space [0.0 – 1.0]:
    // For the X coordinate, the warp offset was computed relative to the sprite center (UV = 0.5)
    //   -> This results in a signed range of [-0.5 … +0.5]
    //   -> To restore standard UV space, shift it back by 0.5:
    altered_uv.x = projected_x_horizontal_squish_from_perspective_tilt + MIDPOINT_UV;
    // For the Y coordinate, tilting occurs around either the bottom or top pivot:
    // - ascending -> pivot is anchored at bottom  (pivot uv = 0.0)
    // - descending -> pivot is anchored at top  (pivot uv = 1.0)
    altered_uv.y = ascending
        ? projected_y_vertically_from_perspective_tilt
        : 1.0 - projected_y_vertically_from_perspective_tilt;

    perspective_tilt = ascending
        ? sprite_tilt_phase_progress * altered_uv.y
        : sprite_tilt_phase_progress * (1.0 - altered_uv.y);
}
