shader_type canvas_item;
render_mode blend_disabled;

#include "res://Resources/Shaders/IceSheets/noise.gdshaderinc"
#include "res://Resources/Shaders/IceSheets/projections.gdshaderinc"

uniform vec2  iResolution;
uniform float iTime;

void fragment() {
    vec2 frag_coord = FRAGCOORD.xy;
    frag_coord.y    = iResolution.y - frag_coord.y;
    vec2 normCoord = vec2(
        (frag_coord.x * 2.0 - iResolution.x) / iResolution.y,
        (frag_coord.y * 2.0 - iResolution.y) / iResolution.y
    );
    float localNoiseScale;
    vec2  projectedTop = projectTopLayerForParallax(normCoord, localNoiseScale);
    bool  solidTop     = isSolidAtCoord(projectedTop, localNoiseScale, iTime, 0);
    COLOR = vec4(solidTop ? 1.0 : 0.0, 0.0, 0.0, 0.0);
}
