shader_type canvas_item;
#define MAXIMUM_TILT_ANGLE_ACHIEVED_AT_IMMEDIATE_ASCENSION_AND_FINAL_DESCENT  0.785398 // 45° in radians
#define MIDPOINT_UV 0.5
#define FOCAL_LENGTH  1.0 

uniform float altitude_normal; //value from 0 -> 1 -> 0 tracing the jump state as a normal scale for altitude
uniform bool  ascending;

void fragment() {
//void mainImage(out vec4 frag_color, in vec2 frag_coord) {
    vec2 uv = UV;
    //vec2 uv = frag_coord.xy / iResolution.xy;
    // If on ground (no jump) or tilt disabled, draw sprite normally
    if (altitude_normal <= 0.0 || MAXIMUM_TILT_ANGLE_ACHIEVED_AT_IMMEDIATE_ASCENSION_AND_FINAL_DESCENT <= 0.0) {
        COLOR = texture(TEXTURE, uv) * COLOR;
        //frag_color =  texture(iChannel0, uv) * frag_color;
    } else {
        // 1) normalized 0->1 fraction through jump cycle (0 = lift‑off/land, 1 = apex, back to 0 for landing)
        float jump_phase_progress = altitude_normal;
        // 2) how much ground‑tilt remains (1 = full tilt on ground, 0 = no tilt at apex)
        float sprite_tilt_phase_progress = 1.0 - jump_phase_progress;
        // 3) TODO: this is still hurting my head
        float current_tilt_angle = (1.0 - altitude_normal) * MAXIMUM_TILT_ANGLE_ACHIEVED_AT_IMMEDIATE_ASCENSION_AND_FINAL_DESCENT;
        // 4) TODO: idk why but it needs to happen... because of the mess i got myself into up until not
        current_tilt_angle = -1.0 * current_tilt_angle; 
        // 5) flip the texture because idk any other way...
        if (!ascending) {
            uv.y = 1.0 - uv.y; 
        }
        // 6) horizontal UV offset from center (0.5)
        float midpoint_uv_x_of_sprite_quad = uv.x - MIDPOINT_UV;
        /* introducing... highscool trig... dang:
                     Tip of sprite undergoing tilt
                            *
                           /|
                          / |   ← Opposite = depth into/out of screen = L × sin(θ)
                         /  |
                        /   |
                       /    |
                      /     |
                     /θ     |
        Pivot ●─────/───────●───  
              Adj = apparent height on screen = L × cos(θ)
       - L = current_uv_y_distance_from_pivot_uv
       - cos(θ) = adjacent/hypotenuse → gives screen‑visible height
       - sin(θ) = opposite/hypotenuse → gives depth offset for perspective */
        float cosine_of_tilt_angle = cos(current_tilt_angle);
        float sine_of_tilt_angle   = sin(current_tilt_angle);
        // 7) sprite_quad_height_after_tilt_applied = L × cos(θ)
        float sprite_quad_height_after_tilt_applied = uv.y * cosine_of_tilt_angle;
        // 8) sprite_quad_depth_after_tilt = L × sin(θ)
        float sprite_quad_depth_after_tilt = uv.y * sine_of_tilt_angle;
        // 9) Build asymptotic perspective divisor: (1 + depth/f)
        //     - depth > 0 (farther) → divisor > 1 → compress
        //     - depth < 0 (closer) → divisor < 1 → expand
        float perspective_asymptotic_scalar =
            1.0 + sprite_quad_depth_after_tilt 
                        / 
                  FOCAL_LENGTH;
        // 10) Foreshortened vertical position = (L cos θ) / (1 + (L sin θ)/f)
        float projected_y_verticality_from_perspective_tilt =
            sprite_quad_height_after_tilt_applied
                            / 
               perspective_asymptotic_scalar;
        // 11) Apply same perspective scale to horizontal offset
        float projected_x_horizontal_squish_from_perspective_tilt =
            midpoint_uv_x_of_sprite_quad
                        / 
            perspective_asymptotic_scalar;

        vec2 projected_uv;
        // 12) Re‑anchor the warped offsets back into standard UV [0…1]:
        //    X‑axis: we centered X around MIDPOINT_UV (0.5) earlier,
        //    so projected_x_horizontal_squish_from_perspective_tilt
        //    is a signed offset in roughly [–0.5…+0.5]. Adding MIDPOINT_UV shifts
        //    that back into the proper 0…1 UV range:
        //        newUV.x = offsetFromCenter + 0.5
        projected_uv.x =  projected_x_horizontal_squish_from_perspective_tilt + MIDPOINT_UV;
        //    Y‑axis: we measured Y as a signed offset from the hinge line
        //    (uv_pivot_for_tilting = 0 for bottom hinge, 1 for top hinge)
        //    projected_y_verticality_from_perspective_tilt is that offset
        //    Adding uv_pivot_for_tilting rebases it into [0…1]:
        //        newUV.y = offsetFromHinge + pivotUV
        projected_uv.y = projected_y_verticality_from_perspective_tilt;
        // 13) discard samples that fall at least 1 pixel outside of the projected texture’s new bounds 
        //(to avoid sampling outside of a fragment warped quad)
        float one_pixel_uv_width  = TEXTURE_PIXEL_SIZE.x;
        float one_pixel_uv_height = TEXTURE_PIXEL_SIZE.y;
        //float one_pixel_uv_width  = 1.0 / iResolution.x;
        //float one_pixel_uv_height = 1.0 / iResolution.y;
        if (projected_uv.x < one_pixel_uv_width 
            || projected_uv.x > 1.0 - one_pixel_uv_width 
            || projected_uv.y < one_pixel_uv_height 
            || projected_uv.y > 1.0 - one_pixel_uv_height) discard;
        COLOR = texture(TEXTURE, projected_uv) * COLOR;
        //frag_color =  texture(iChannel0, projected_uv) * frag_color;
    }
}
