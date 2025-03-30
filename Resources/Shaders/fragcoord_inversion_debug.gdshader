shader_type canvas_item;

uniform vec2 iResolution; 
uniform vec3 iMouse;      

//void mainImage(out vec4 fragColor, in vec2 fragCoord) {
void fragment() {
    //vec2 uv = fragCoord.xy / iResolution.xy;
    //vec2 uv = FRAGCOORD.xy / iResolution.xy;  
    vec2 uv = vec2(FRAGCOORD.x, iResolution.y - FRAGCOORD.y) / iResolution.xy;
    vec2 mouseUV = iMouse.xy / iResolution.xy;
    float dist_from_mouse = distance(uv, mouseUV);
    float is_pressed = step(0.5, iMouse.z);  // 1.0 if mouse pressed, 0.0 otherwise
    float debugImpulse = smoothstep(0.2, 0.0, dist_from_mouse) * is_pressed;
    COLOR = vec4(uv.y, mouseUV.y, debugImpulse, 1.0);
    //fragColor = vec4(uv.y, mouseUV.y, debugImpulse, 1.0);
}
