shader_type canvas_item;

//In OpenGL/Compatibility AND Vulkan/Forward+ mode, this iChannel0 screen sampling works fine
//TODO: I am going to add BackBufferCopy to anticipate double screen sampling shaders though!!!
uniform sampler2D iChannel0 : hint_screen_texture, filter_nearest; //TODO: FIGURE OUT FILTERING

//TODO: hard coded MainViewport.get_texture() sampling results in the classic vulkan error only in Forward+/non compatibility mode (OpenGl has no such error
//draw_list_bind_uniform_set: Attempted to use the same texture in framebuffer attachment and a uniform (set: 1, binding: 1), this is not allowed.
//<C++ Error> Condition "attachable_ptr[i].texture == bound_ptr[j]" is true.
//<C++ Source> servers/rendering/rendering_device.cpp:4538 @ draw_list_bind_uniform_set()
// source: https://github.com/godotengine/godot/issues/44473

//In OpenGL/Compatibility mode it results in the upper left quadrant triangle visibility bug
// source: https://github.com/godotengine/godot/issues/17234
//uniform sampler2D iChannel0: filter_nearest; //filter hint ust to remind me that later if background texture is pixel perfect or mip mapped...

uniform sampler2D iChannel1 : filter_nearest;   // tilt mask generated in compute shader

uniform vec2 iResolution;
uniform vec2 umbral_zone_bounds;
uniform float luminance_attenuation_coefficient = 0.75;

void fragment() {
    vec2 uv = UV;
    vec4 color = texture(iChannel0, uv);
    float perspective_tilt = texture(iChannel1, UV).r;
    if (uv.x < umbral_zone_bounds.x && uv.y < umbral_zone_bounds.y) {
        //these mixes apply the umbral effect uniformly, and then only in areas of perspective
        //tilt mask do we apply the sprite shadows
        color.rgb = mix(color.rgb, color.rgb * luminance_attenuation_coefficient, perspective_tilt);
        float shade = mix(luminance_attenuation_coefficient, 0.0, perspective_tilt);
        color.rgb *= shade;
    }
    COLOR = color;
}
