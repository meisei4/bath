shader_type canvas_item;

uniform float altitude_location = 0.0;
uniform bool ascending = true;
uniform float max_angle_degrees = 0.0;
uniform float apex_sharpness = 0.0;
//TODO: Unsure on how to better control the actual rate of the warping...

void fragment() {
    vec2 uv_mod = UV;
    //TODO: this part is not a smooth transition when jumping consecutively
    if (altitude_location <= 0.0 || max_angle_degrees <= 0.0) {
        COLOR = texture(TEXTURE, uv_mod) * COLOR;
    } else {
        float t = pow(clamp(altitude_location, 0.0, 1.0), apex_sharpness);
        float a = (1.0 - t) * 2.0;
        //float a = (1.0 - altitude_location) * 2.0;
        float max_rad = radians(max_angle_degrees);
        float angle = a * max_rad;
        //TODO: whatever little hack here for smoothness?
        float scale_y = max(cos(angle), 0.20);
        float pivot = ascending ? 0.0 : 1.0;
        uv_mod.y = pivot + (uv_mod.y - pivot) * scale_y;
        if (uv_mod.y <= TEXTURE_PIXEL_SIZE.y || uv_mod.y >= 1.0 - TEXTURE_PIXEL_SIZE.y) {
            discard; //this is the only way i can prevent sampling outside warped sprite texture
        }
        COLOR = texture(TEXTURE, uv_mod) * COLOR;
    }
}

//TODO: test more in shadertoy for this wackyness to actually do what i want it to do
//uniform float apex_sharpness = 5.0;
//uniform float flat_duration = 0.5; // From 0.0 to ~0.5
//
//void fragment() {
    //vec2 uv_mod = UV;
    //if (altitude_location <= 0.0) {
        //COLOR = texture(TEXTURE, uv_mod) * COLOR;
    //} else {
        //float t = clamp(altitude_location, 0.0, 1.0);
        //float flat_half = flat_duration * 0.5;
        //float dist_from_center = abs(t - 0.5);
        //float norm = clamp((dist_from_center - flat_half) / (0.5 - flat_half), 0.0, 1.0);
        //float shaped = pow(norm, apex_sharpness);
        //float scale_y = mix(1.0, 0.01, shaped);
        //float pivot = t < 0.5 ? 1.0 : 0.0;
        //uv_mod.y = pivot + (uv_mod.y - pivot) * scale_y;
        //if (uv_mod.y <= TEXTURE_PIXEL_SIZE.y || uv_mod.y >= 1.0 - TEXTURE_PIXEL_SIZE.y) {
            //discard;
        //}
        //COLOR = texture(TEXTURE, uv_mod) * COLOR;
    //}
//}
//
