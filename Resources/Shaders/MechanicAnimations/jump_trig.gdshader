shader_type canvas_item;
#include "res://Resources/Shaders/MechanicAnimations/perspective_tilt.gdshaderinc"
uniform vec2 iResolution;
// TODO: mess with these sampling hints more, because it implies the Sprite2D node's texture is not
// defaulted to filter nearest despite how i explcitly set the texture to be as such in CapsuleDummyGen:
// sprite.set_texture_filter(CanvasItem.TextureFilter.TEXTURE_FILTER_NEAREST)
uniform sampler2D iChannel0: filter_nearest; //TODO: figure out filtering stuff with all the shaders for consistency
uniform float altitude_normal; //value from 0 -> 1 -> 0 tracing the jump state as a normal scale for altitude
uniform bool  ascending;

#define DISCARD_PIXELS_OUTSIDE_OF_ALTERED_UV_BOUNDS_FRAG(uv, texel_size) \
    if ((uv).x < (texel_size).x \
        || (uv).x > 1.0 - (texel_size).x \
        || (uv).y < (texel_size).y \
        || (uv).y > 1.0 - (texel_size).y) \
        discard;

void fragment() {
//void mainImage(out vec4 frag_color, in vec2 frag_coord) {
    vec2 uv = UV;
    //vec2 uv = FRAGCOORD.xy / iResolution.xy;
    //vec2 uv = frag_coord.xy / iResolution.xy;
    if (altitude_normal <= 0.0) {
        COLOR = texture(iChannel0, uv);
    } else {
        vec2 altered_uv;
        float perspective_tilt;
        compute_perspective_tilt(
            uv,
            altitude_normal,
            ascending,
            altered_uv,
            perspective_tilt
        );
        // Discard samples that fall =>1 pixel outside of the projected texture’s bounds
        //(to avoid sampling outside of a fragment warped quad)
        DISCARD_PIXELS_OUTSIDE_OF_ALTERED_UV_BOUNDS_FRAG(altered_uv, TEXTURE_PIXEL_SIZE)
        COLOR = texture(iChannel0, altered_uv);
        //frag_color =  texture(iChannel0, warped_uv);
    }
}
