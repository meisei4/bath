shader_type canvas_item;

uniform float altitude_location = 0.0;
uniform bool ascending = true;
uniform float max_angle_degrees = 0.0;
uniform float apex_sharpness = 1.0;
//TODO: Unsure on how to better control the actual rate of the warping...

void fragment() {
    vec2 uv_mod = UV;
    if (altitude_location <= 0.0 || max_angle_degrees <= 0.0) {
        COLOR = texture(TEXTURE, uv_mod) * COLOR;
    } else {
        float t = pow(clamp(altitude_location, 0.0, 1.0), apex_sharpness);
        float a = (1.0 - t) * 2.0;
        //float a = (1.0 - altitude_location) * 2.0;
        float max_rad = radians(max_angle_degrees);
        float angle = a * max_rad;
        float scale_y = max(cos(angle), 0.0);
        float pivot = ascending ? 0.0 : 1.0;
        uv_mod.y = pivot + (uv_mod.y - pivot) * scale_y;
        if (uv_mod.y <= TEXTURE_PIXEL_SIZE.y || uv_mod.y >= 1.0 - TEXTURE_PIXEL_SIZE.y) {
            discard; //this is the only way i can prevent sampling outside warped sprite texture
        }
        COLOR = texture(TEXTURE, uv_mod) * COLOR;
    }
}
