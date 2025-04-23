shader_type canvas_item;

uniform float u_occluder[8];
uniform vec2 u_light_pos;
uniform float u_light_radius;
uniform vec4 u_light_color;
uniform vec4 u_shadow_color;
uniform vec2 u_screen_size;  // New uniform

bool segment_intersects_light_ray(vec2 pixel, vec2 light, vec2 p1, vec2 p2) {
    float denom = (p2.y - p1.y) * (light.x - pixel.x) - (p2.x - p1.x) * (light.y - pixel.y);
    if (abs(denom) < 1e-6) {
        return false;
    }
    float ua = ((p2.x - p1.x) * (pixel.y - p1.y) - (p2.y - p1.y) * (pixel.x - p1.x)) / denom;
    float ub = ((light.x - pixel.x) * (pixel.y - p1.y) - (light.y - pixel.y) * (pixel.x - p1.x)) / denom;
    return (ua >= 0.0 && ua <= 1.0 && ub >= 0.0 && ub <= 1.0);
}

void fragment() {
    // Correct pixel calculation using screen size
    vec2 pixel = SCREEN_UV * u_screen_size;
    float dist = distance(pixel, u_light_pos);
    float attenuation = 1.0 - clamp(dist / u_light_radius, 0.0, 1.0);

    vec2 v1 = vec2(u_occluder[0], u_occluder[1]);
    vec2 v2 = vec2(u_occluder[2], u_occluder[3]);
    vec2 v3 = vec2(u_occluder[4], u_occluder[5]);
    vec2 v4 = vec2(u_occluder[6], u_occluder[7]);

    bool shadowed = false;
    if (segment_intersects_light_ray(pixel, u_light_pos, v1, v2)) {
        shadowed = true;
    } else if (segment_intersects_light_ray(pixel, u_light_pos, v2, v3)) {
        shadowed = true;
    } else if (segment_intersects_light_ray(pixel, u_light_pos, v3, v4)) {
        shadowed = true;
    } else if (segment_intersects_light_ray(pixel, u_light_pos, v4, v1)) {
        shadowed = true;
    }

    if (shadowed) {
        COLOR = u_shadow_color;
    } else {
        COLOR = u_light_color * attenuation;
    }
}
