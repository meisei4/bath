// https://github.com/raysan5/raylib/blob/master/examples/shaders/resources/shaders/glsl330/base.vs
#version 330

in vec3 vertexPosition;
in vec2 vertexTexCoord;

in vec3 vertexNormal;
in vec4 vertexColor;

out vec2 fragTexCoord;
out vec4 fragColor;

uniform mat4 mvp;

void main() {
    fragTexCoord = vertexTexCoord;
    fragColor    = vertexColor;
    gl_Position  = mvp * vec4(vertexPosition, 1.0);
}

/*
 v3 (  0,384) *-------------* v2 (256,384)
              |            /|
              |          /  |
              |        /    |
              |      /      |
              |    /        |
              |  /          |
              |/            |
 v0 (  0,  0) *-------------* v1 (256,  0)
        Triangles
        { v0, v1, v2 }
        { v0, v2, v3 }

    vec3 vertexPosition (Pixel Space)
        v0 = (  0,   0, 0)
        v1 = (256,   0, 0)
        v2 = (256, 384, 0)
        v3 = (  0, 384, 0)

    vec2 vertexTexCoord (Normal/UV Space)
        UVv0 = (0.0, 0.0)
        UVv1 = (1.0, 0.0)
        UVv2 = (1.0, 1.0)
        UVv3 = (0.0, 1.0)

    mat4 mvp (Model View Projection matrix)
        m[0][0] =  2 / (v1.x - v0.x)
        m[1][1] =  2 / (v2.y - v0.y)
        m[2][2] = -2 / (far - near)
        m[3][0] = -(v1.x + v0.x) / (v1.x - v0.x)
        m[3][1] = -(v2.y + v0.y) / (v2.y - v0.y)
        m[3][2] = -(far + near)  / (far - near)
            v0.x = v3.x = 0
            v1.x = v2.x = 256
            v0.y = v1.y = 0
            v2.y = v3.y = 384
            near = v0.z = v1.z = v2.z = v3.z = 0
            far  = 1 (arbitrary depth far plane for 2D rendering)

    vec4 gl_Position (Clip Space with depth and perspective)
        v0 = (-1.0, -1.0, 0.0, 1.0)
        v1 = ( 1.0, -1.0, 0.0, 1.0)
        v2 = ( 1.0,  1.0, 0.0, 1.0)
        v3 = (-1.0,  1.0, 0.0, 1.0)

    x, y - screen position in "clip space" (-1.0 to +1.0)
    z    - depth into the screen (used for depth testing; usually 0 in 2D)
    w    - perspective divisor; after this stage, GPU computes:
            x_ndc = x / w
            y_ndc = y / w
            z_ndc = z / w


1. Vertex shader runs once per corner and outputs vertexTexCoord (vec2's from above):
    vertexTexCoord₀, vertexTexCoord₁, vertexTexCoord₂

2. GPU rasterizes the triangles into fragments/pixels
    - i.e. calculates barycentric weights: λ₀, λ₁, λ₂,
    - barycentric weights measure how close the fragment is to the three vertices
    - barycentric weights always sum to 1

3. GPU then interpolates vertexTexCoord using barycentric weights:
    - fragTexCoord = λ₀·vertexTexCoord₀ + λ₁·vertexTexCoord₁ + λ₂·vertexTexCoord₂
    - THIS IS WHAT THE FRAG SHADER GETS AS INPUT!!!!

If you apply any F() to vertices in the vertex shader, the fragment recieves it as:
    F_interp = λ₀·F(vertexTexCoord₀) + λ₁·F(vertexTexCoord₁) + λ₂·F(vertexTexCoord₂)

If you apply F() in the fragment shader it is a function of:
    F(λ₀·vertexTexCoord₀ + λ₁·vertexTexCoord₁ + λ₂·vertexTexCoord₂)


Pre-Interpolated Vertex vs. Fragment Evaluation:

Barycentric / λ₀ λ₁ λ₂ formula: λ’s are the “how-close” weights the rasteriser produces.

Pre-Interpolated Vertex
F_interp = λ₀·F(UV₀) + λ₁·F(UV₁) + λ₂·F(UV₂)

Fragment evaluation:
F(λ₀·UV₀ + λ₁·UV₁ + λ₂·UV₂)

These are equal if and only if F is affine:

Affine Functions - safe to migrate from fragment shader to vertex shader):
F(UV) = A·UV + B

Safe (Affine) Operations:
- add/subtract scalar: UV + scalar
- scale/divide by scalar: UV * scalar
- matrix multiplication: mat2 * UV
- swizzle????: vec2(UV.y, UV.x) -- changing the vector element order?

Unsafe (Non-Affine) Operations:
- nonlinear math: sin(UV.x), pow(UV.x, 2.0)
- functions of components: fract(UV), smoothstep(a, b, UV)
- varying × varying: UV.x * UV.y, UV / UV
- piecewise: min, max, step, clamp
- inverse/root: 1.0 / UV.x, sqrt(UV.x)
*/