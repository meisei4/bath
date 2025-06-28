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

uniform float iTime;
uniform vec2  iResolution;

uniform float depthScalar;
uniform int   tileSize;

uniform mat3x2 uvXlinear;
uniform mat3x2 uvXbilinear;
uniform mat3x2 uvXnonlinear;
uniform mat3x2 uvXaffine;

// #define PASS
// #define BARYCENTRIC
// #define ANIMATE_UV
// #define AFFINE_UV
#define PERSPECTIVE_PROJECTION_UV
// #define XY_POLYNOMIAL_APPROX_PROJECTION_UV
// TODO: I am the dumbest person alive
// #define X_LINEAR_Y_POLYNOMIAL_APPROX_PROJECTION_UV
// #define LINEAR_UV
// #define BILINEAR_UV

// #define ANIMATE_GEOM
// #define AFFINE_GEOM
// #define NONAFFINE_GEOM
// #define LINEAR_GEOM
// #define BILINEAR_GEOM
// #define NONLINEAR_GEOM

const vec4 RED    = vec4(1.0, 0.0, 0.0, 1.0);
const vec4 GREEN  = vec4(0.0, 1.0, 0.0, 1.0);
const vec4 BLUE   = vec4(0.0, 0.0, 1.0, 1.0);
const vec4 YELLOW = vec4(1.0, 1.0, 0.0, 1.0);
// TODO: To prove that there are only 4 vertices, these two colors will not show up anywhere
const vec4 MAGENTA    = vec4(1.0, 0.0, 1.0, 1.0);
const vec4 CYAN       = vec4(0.0, 1.0, 1.0, 1.0);
const vec4 palette[6] = vec4[6](RED, GREEN, BLUE, YELLOW, MAGENTA, CYAN);

void main() {
    vec3 vPosition      = vertexPosition;
    vec2 vTexCoord      = vertexTexCoord;
    vec2 vPixelCoord    = vertexTexCoord * iResolution;
    vertexDebugPayload0 = vertexNormal;
    vertexDebugPayload1 = vertexColor;
#ifdef BARYCENTRIC
    fragColor = palette[gl_VertexID % 6];
#endif
#ifdef ANIMATE_UV
// TODO:
#endif
#ifdef AFFINE_UV
    vTexCoord = (uvXaffine * vec3(vTexCoord, 1.0));
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
    vec2  vAspectNormal         = (vPixelCoord * 2.0 - iResolution) / iResolution.y;
    float k                     = log((depthScalar + 1.0)
                            / (depthScalar - 1.0)); // solved least common squares or something from the true perspective divide
    float focalScale            = 0.5 * k;
    float gradScale             = 1.5 * (depthScalar * k - 2.0);
    float scaleY                = focalScale + gradScale * vAspectNormal.y;
    vec2  polynomialApproxProjY = vAspectNormal * vec2(focalScale, scaleY);
    vTexCoord                   = (polynomialApproxProjY + 1.0) * 0.5;
#endif
#ifdef NONAFFINE_UV
    vTexCoord = (uvXnonaffine * vec3(vTexCoord, 1.0));
#endif
#ifdef LINEAR_UV
    vTexCoord = (uvXlinear * vec3(vTexCoord, 1.0));
#endif
#ifdef BILINEAR_UV
    vTexCoord = (uvXbilinear * vec3(vTexCoord, 1.0));
#endif
#ifdef AFFINE_GEOM
    vPosition.xy = (uvXaffine * vec3(vPosition.xy, 1.0));
#endif
#ifdef NONAFFINE_GEOM
    vPosition.xy = (uvXnonaffine * vec3(vPosition.xy, 1.0));
#endif
#ifdef LINEAR_GEOM
    vPosition.xy = (uvXlinear * vec3(vPosition.xy, 1.0));
#endif
#ifdef BILINEAR_GEOM
    vPosition.xy = (uvXbilinear * vec3(vPosition.xy, 1.0));
#endif
#ifdef NONLINEAR_GEOM
    vPosition.xy = (uvXnonlinear * vec3(vPosition.xy, 1.0));
#endif
#ifdef PASS
    fragColor    = vertexColor;
    fragCoord    = vPixelCoord;
    fragTexCoord = vertexTexCoord;
    gl_Position  = mvp * vec4(vertexPosition, 1.0);
    return;
#endif
    fragCoord    = vPixelCoord; // keep unwarped
    fragTexCoord = vTexCoord; // potentially warped
    gl_Position  = mvp * vec4(vPosition, 1.0);
}
