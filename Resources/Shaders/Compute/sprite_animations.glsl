#[compute]
#version 450

// Invocations in the (x, y, z) dimension
layout(local_size_x = 2, local_size_y = 1, local_size_z = 1) in;

// binding 0 = our per-sprite buffer
layout(set = 0, binding = 0, std430) buffer SpriteData {
    vec4 data[];
} 
sprite_data;

// The code we want to execute in each invocation
void main() {
    // gl_GlobalInvocationID.x uniquely identifies this invocation across all work groups
    sprite_data.data[gl_GlobalInvocationID.x] *= 2.0;
}
