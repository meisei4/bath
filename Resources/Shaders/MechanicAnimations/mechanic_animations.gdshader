shader_type canvas_item;

uniform float lateral_amount_normal = 0.0;  // –1 full left … +1 full right
uniform float max_shift             = 0.15; // how far (fraction of width) each slice moves
uniform float pinch_factor          = 0.5;  // how narrow slice gets before sliding
const   float SLICES                = 6.0;  // number of horizontal strips

void fragment() {
    vec2 uv = UV;
    float mag = abs(lateral_amount_normal);
    float dir = sign(lateral_amount_normal);

    if (mag > 0.001) {
        // shrink the sampling band so shifting never bleeds
        uv.x = mix(max_shift, 1.0 - max_shift, uv.x);

        // which slice is this?
        float idx = floor(UV.y * SLICES);

        // when does this slice start? [ idx/SLICES … (idx+1)/SLICES ]
        float start = idx / SLICES;
        float end   = (idx + 1.0) / SLICES;
        // normalized progress of this slice: 0→1
        float t = clamp((mag - start) / (end - start), 0.0, 1.0);

        // 1) pinch from t=0…0.5,  then
        // 2) slide   from t=0.5…1.0
        float pinch_t = smoothstep(0.0, 0.5, t);
        float slide_t = smoothstep(0.5, 1.0, t);

        // apply pinch around pivot edge
        float pivot = dir > 0.0 ? 0.0 : 1.0;
        float scaleX = mix(1.0, pinch_factor, pinch_t);
        uv.x = pivot + (uv.x - pivot) * scaleX;

        // then apply slide
        uv.x += dir * slide_t * max_shift;

        // clamp so no pixel‑bleed
        uv.x = clamp(uv.x, TEXTURE_PIXEL_SIZE.x, 1.0 - TEXTURE_PIXEL_SIZE.x);
    }

    COLOR = texture(TEXTURE, uv) * COLOR;
}




//uniform float altitude_location = 0.0;
//uniform bool ascending = true;
//uniform float max_angle_degrees = 45.0;
//uniform float apex_sharpness = 2.0;
////TODO: Unsure on how to better control the actual rate of the warping...


////void fragment() {
    //vec2 uv_mod = UV;
    ////TODO: this part is not a smooth transition when jumping consecutively
    //if (altitude_location <= 0.0 || max_angle_degrees <= 0.0) {
        //if (abs(lateral_amount_normal) > 0.0) {
            //// 1) Shrink the sampling region so we can shift without bleeding edges
            //uv_mod.x = mix(max_shift, 1.0 - max_shift, uv_mod.x);
//
            //// 2) Which slice are we in?
            //float slice_idx = floor(UV.y * SLICES);
            //// each slice waits its turn: it starts moving when |lateral_amount| ≥ slice_idx/SLICES
            //float t_base    = abs(lateral_amount_normal);
            //float slice_t   = clamp((t_base - slice_idx / SLICES) * SLICES, 0.0, 1.0);
//
            //// 3) Compute horizontal offset for this slice
            //float dir    = sign(lateral_amount_normal);
            //float offset = dir * slice_t * max_shift;
            //uv_mod.x    += offset;
//
            //// 4) Clamp to texture bounds
            //uv_mod.x = clamp(uv_mod.x, TEXTURE_PIXEL_SIZE.x, 1.0 - TEXTURE_PIXEL_SIZE.x);
        //}
        //COLOR = texture(TEXTURE, uv_mod) * COLOR;
    //} else {
        //float t = pow(clamp(altitude_location, 0.0, 1.0), apex_sharpness);
        //float a = (1.0 - t) * 2.0;
        ////float a = (1.0 - altitude_location) * 2.0;
        //float max_rad = radians(max_angle_degrees);
        //float angle = a * max_rad;
        ////TODO: whatever little hack here for smoothness?
        //float scale_y = max(cos(angle), 0.20);
        //float pivot = ascending ? 0.0 : 1.0;
        //uv_mod.y = pivot + (uv_mod.y - pivot) * scale_y;
        //if (uv_mod.y <= TEXTURE_PIXEL_SIZE.y || uv_mod.y >= 1.0 - TEXTURE_PIXEL_SIZE.y) discard; //this is the only way i can prevent sampling outside warped sprite texture
        //COLOR = texture(TEXTURE, uv_mod) * COLOR;
    //}
//}
