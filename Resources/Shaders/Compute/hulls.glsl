#[compute]
#version 450
#extension GL_KHR_shader_subgroup_basic : enable  


#define MAX_HULL_POINTS 256

layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

#define HULL_BOUNDARY_SSBO_BINDING 0
layout(std430, binding = 0) buffer HullBoundarySSBO {
    vec2 boundaryPointList[];
};

#define HULL_SORTED_SSBO_BINDING 1
layout(std430, binding = HULL_SORTED_SSBO_BINDING) buffer HullSortedSSBO {
    vec2 sortablePointList[];
};

#define HULL_RESULT_SSBO_BINDING 2
layout(std430, binding = HULL_RESULT_SSBO_BINDING) buffer HullResultSSBO {
    uint hullCount;          // byte‐offset 0
    uint __pad;         // byte‐offset 4 (this brings us to 8)
    vec2 resultHullArray[];  // starts at byte‐offset 4
};


layout(push_constant) uniform PushConstants {
    int numberOfPoints;          // = boundaryPointList.length()
    int minVerticesForAndrew;    // e.g. 3
    int maxHullPoints;           // must be ≤ MAX_HULL_POINTS
} push_constants;

float orientation(vec2 a, vec2 b, vec2 c) {
    return (b.x - a.x)*(c.y - a.y) - (b.y - a.y)*(c.x - a.x);
}

shared vec2 lowerHull[MAX_HULL_POINTS];
shared int lowerSize;
shared vec2 upperHull[MAX_HULL_POINTS];
shared int upperSize;

void main() {
    if (gl_GlobalInvocationID.x != 0) {
        return;
    }

    int n = push_constants.numberOfPoints;
    if (n < push_constants.minVerticesForAndrew) {
        for (int i = 0; i < n; ++i) {
            resultHullArray[i] = boundaryPointList[i];
        }
        return;
    }

    for (int i = 0; i < n; ++i) {
        sortablePointList[i] = boundaryPointList[i];
    }

    for (int i = 0; i < n - 1; ++i) {
        for (int j = i + 1; j < n; ++j) {
            vec2 a = sortablePointList[i];
            vec2 b = sortablePointList[j];
            if ((b.x < a.x) || (b.x == a.x && b.y < a.y)) {
                sortablePointList[i] = b;
                sortablePointList[j] = a;
            }
        }
    }

    lowerSize = 0;
    barrier();

    for (int idx = 0; idx < n; ++idx) {
        vec2 p = sortablePointList[idx];
        while (lowerSize >= 2 &&
               orientation(lowerHull[lowerSize - 2],
                           lowerHull[lowerSize - 1],
                           p) <= 0.0) {
            lowerSize--;
        }
        lowerHull[lowerSize++] = p;
    }
    barrier();

    upperSize = 0;
    barrier();

    for (int idx = n - 1; idx >= 0; --idx) {
        vec2 p = sortablePointList[idx];
        while (upperSize >= 2 &&
               orientation(upperHull[upperSize - 2],
                           upperHull[upperSize - 1],
                           p) <= 0.0) {
            upperSize--;
        }
        upperHull[upperSize++] = p;
    }
    barrier();

    int lowerCount = lowerSize - 1;
    int upperCount = upperSize - 1;
    int w = 0;
    for (int i = 0; i < lowerCount; ++i) {
        resultHullArray[w++] = lowerHull[i];
    }
    for (int i = 0; i < upperCount; ++i) {
        resultHullArray[w++] = upperHull[i];
    }
    hullCount = uint(w);
    __pad = 0; // not strictly necessary, but keeps your data well-defined
 }
