#version 330

in vec3 vertexPosition;
in vec2 vertexTexCoord;

in vec3 vertexNormal;
in vec4 vertexColor;

out vec2 fragCoord;
out vec2 fragTexCoord;
// TODO: this is the proof of how the triangles are oriented in the quad:
// flat out vec4 fragColor;
out vec4 fragColor;

out vec2 vertexNormCoord;

out vec3 vertexDebugPayload0;
out vec4 vertexDebugPayload1;

uniform mat4 mvp;
uniform mat4 mode7_mvp;

uniform float iTime;
uniform vec2  iResolution;

const float depthScalar    = 6.0;
const vec2  scrollVelocity = vec2(0.0, 0.05);
const float macroScale     = 180.0;
const float microScale     = 0.025;
const float uniformStretch = 1.414213562;
const float stretchY       = 2.0;
const float PI             = 3.141592653589793;
const mat2  rotationMatrix = mat2(cos(-PI * 0.25), -sin(-PI * 0.25), sin(-PI * 0.25), cos(-PI * 0.25));
const vec2  staticOffset   = vec2(2.0, 0.0);

#define PASS
// #define BARYCENTRIC
//  #define PERSPECTIVE_PROJECTION_UV
//  #define XY_POLYNOMIAL_APPROX_PROJECTION_UV
//  #define X_LINEAR_Y_POLYNOMIAL_APPROX_PROJECTION_UV
// #define AFFINE_UV

const vec4 RED    = vec4(1.0, 0.0, 0.0, 1.0);
const vec4 GREEN  = vec4(0.0, 1.0, 0.0, 1.0);
const vec4 BLUE   = vec4(0.0, 0.0, 1.0, 1.0);
const vec4 YELLOW = vec4(1.0, 1.0, 0.0, 1.0);
// TODO: To prove that there are only 4 vertices, these two colors will not show up anywhere
const vec4 MAGENTA    = vec4(1.0, 0.0, 1.0, 1.0);
const vec4 CYAN       = vec4(0.0, 1.0, 1.0, 1.0);
const vec4 palette[6] = vec4[6](RED, GREEN, BLUE, YELLOW, MAGENTA, CYAN);

void main() {
    vec3 vPosition   = vertexPosition;
    vec2 vTexCoord   = vertexTexCoord;
    vec2 vPixelCoord = vertexTexCoord * iResolution;
    mat4 vmpv        = mvp;

    vertexDebugPayload0 = vertexNormal;
    vertexDebugPayload1 = vertexColor;
#ifdef BARYCENTRIC
    // TODO: this perfectly demonstrates the difference between the custom geom and the default geom with flipping
    fragColor = palette[gl_VertexID % 6];
#endif
#ifdef PERSPECTIVE_PROJECTION_UV
    vec2 vAspectNormal          = (vPixelCoord * 2.0 - iResolution) / iResolution.y;
    vec2 vProperPerspectiveProj = vAspectNormal / (depthScalar - vAspectNormal.y);
    vTexCoord                   = (vProperPerspectiveProj + 1.0) * 0.5; // back to 0-1 UV
#endif
#ifdef XY_POLYNOMIAL_APPROX_PROJECTION_UV
    vec2  vAspectNormal          = (vPixelCoord * 2.0 - iResolution) / iResolution.y;
    float a                      = 0.5 * log((depthScalar + 1.0) / (depthScalar - 1.0));
    float b                      = 1.5 * (depthScalar * log((depthScalar + 1.0) / (depthScalar - 1.0)) - 2.0);
    float scale                  = a + b * vAspectNormal.y;
    vec2  polynomialApproxProjXY = vAspectNormal * scale;
    vTexCoord                    = (polynomialApproxProjXY + 1.0) * 0.5;
#endif
#ifdef X_LINEAR_Y_POLYNOMIAL_APPROX_PROJECTION_UV
    vec2 vAspectNormal = (vPixelCoord * 2.0 - iResolution) / iResolution.y;
    // TODO: solved least common squares or something from the true perspective divide = Taylor expansion
    float k                     = log((depthScalar + 1.0) / (depthScalar - 1.0));
    float focalScale            = 0.5 * k;
    float gradScale             = 1.5 * (depthScalar * k - 2.0);
    float scaleY                = focalScale + gradScale * vAspectNormal.y;
    vec2  polynomialApproxProjY = vAspectNormal * vec2(focalScale, scaleY);
    vTexCoord                   = (polynomialApproxProjY + 1.0) * 0.5;
#endif
#ifdef AFFINE_UV
    vTexCoord += scrollVelocity * iTime;
    vTexCoord *= macroScale * microScale;
    vTexCoord.x *= uniformStretch;
    vTexCoord.y *= stretchY;
    vTexCoord = rotationMatrix * vTexCoord;
    vTexCoord += staticOffset;
#endif
#ifdef PASS
    fragColor = vertexColor;
#endif
    fragCoord    = vPixelCoord;
    fragTexCoord = vTexCoord;
    // gl_Position  = mvp * vec4(vPosition, 1.0);

    float zoom = 1.0;
    float z_x  = (1.0 - zoom) * 0.5 * iResolution.x;
    float z_y  = (1.0 - zoom) * 0.5 * iResolution.y;
    // clang-format off
    mat4 zoomOut = mat4(zoom, 0.0,  0.0, 0.0,
                        0.0,  zoom, 0.0, 0.0,
                        0.0,  0.0,  1.0, 0.0,
                        z_x,  z_y,  0.0, 1.0);
    // clang-format on
    float tilt  = 20.0;
    float t_cos = cos(radians(tilt));
    float t_sin = sin(radians(tilt));
    float t_y1  = (0.5 * iResolution.y) * (1.0 - t_cos);
    float t_y2  = -(0.5 * iResolution.y) * t_sin;
    // clang-format off
    mat4 tiltBack = mat4(1.0,  0.0,    0.0,   0.0,
                        0.0,  t_cos,  t_sin, 0.0,
                        0.0, -t_sin,  t_cos, 0.0,
                        0.0,  t_y1,   t_y2,  1.0);
    // clang-format on

    vmpv = vmpv * zoomOut * tiltBack;

    gl_Position = vmpv * vec4(vPosition, 1.0);
}
