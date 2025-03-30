shader_type canvas_item;

uniform vec3 iMouse;
uniform vec2 iResolution;
uniform sampler2D iChannel0; 

//void mainImage(out vec4 fragColor, in vec2 fragCoord) {
void fragment() {
    //vec2 uv = fragCoord.xy / iResolution.xy;
    //vec2 uv = vec2(FRAGCOORD.x, iResolution.y - FRAGCOORD.y) / iResolution.xy;
    vec2 uv = FRAGCOORD.xy / iResolution.xy;  
    vec4 dbg = texture(iChannel0, uv);
    float ringVal = dbg.b;
    float stepSize = 0.005;
    float sumRing = texture(iChannel0, uv + vec2(+stepSize, 0.0)).b +
                    texture(iChannel0, uv + vec2(-stepSize, 0.0)).b +
                    texture(iChannel0, uv + vec2(0.0, +stepSize)).b +
                    texture(iChannel0, uv + vec2(0.0, -stepSize)).b +
                    ringVal; 
    float avgRing = sumRing / 5.0;
    avgRing *= 0.98;
    float yCoord = dbg.r;
    vec3 color = vec3(avgRing * yCoord, avgRing, avgRing * (1.0 - yCoord));
    COLOR = vec4(color, 1.0);
    //fragColor =  vec4(color, 1.0);
}
