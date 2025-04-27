#[compute]
#version 450
#extension GL_KHR_shader_subgroup_basic : enable  

// maximum vertices per hull, as before
#define MAX_HULL_POINTS 256
// must match your MAX_COLLISION_SHAPES
#define MAX_REGIONS 12

layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

// ——— SSBO #0: all region boundary points, flattened
#define HULL_BOUNDARY_SSBO_BINDING 0
layout(std430, binding = HULL_BOUNDARY_SSBO_BINDING) buffer HullBoundarySSBO {
    vec2 boundaryPointList[];    // length = sum_i N_i
};

// ——— SSBO #1: per-region (offset, length) metadata
#define HULL_SORTED_SSBO_BINDING 1
layout(std430, binding = HULL_SORTED_SSBO_BINDING) buffer HullMetaSSBO {
    uvec2 regionMeta[MAX_REGIONS]; // .x = start index in boundaryPointList, .y = number of pts
};

// ——— SSBO #2: per-region hull counts + big result array
#define HULL_RESULT_SSBO_BINDING 2
layout(std430, binding = HULL_RESULT_SSBO_BINDING) buffer HullResultSSBO {
    uint hullCount[MAX_REGIONS];              // hullCount[r] = vertex count for region r
    // MAX_REGIONS·4 bytes is divisible by 8 if MAX_REGIONS is even → no pad needed
    vec2 resultHullArray[];                   // flattened: region r writes into 
                                               //  resultHullArray[r * push_constants.maxHullPoints + 0…hullCount[r]-1]
};

// ——— push constants
layout(push_constant) uniform PushConstants {
    int regionCount;            // how many regions you dispatched
    int minVerticesForAndrew;   // e.g. 3
    int maxHullPoints;          // ≤ MAX_HULL_POINTS
} push_constants;

// orientation test (same as before)
float orientation(vec2 a, vec2 b, vec2 c) {
    return (b.x - a.x)*(c.y - a.y)
         - (b.y - a.y)*(c.x - a.x);
}

// shared stacks for Andrew’s algorithm
shared vec2 lowerHull[MAX_HULL_POINTS];
shared int lowerSize;
shared vec2 upperHull[MAX_HULL_POINTS];
shared int upperSize;

void main() {
    uint regionId = gl_GlobalInvocationID.x;
    if (regionId >= uint(push_constants.regionCount)) {
        return;
    }

    // fetch this region’s metadata
    uvec2 m = regionMeta[regionId];
    int baseIdx = int(m.x);
    int n       = int(m.y);

    // trivial case: too few points → copy straight back
    if (n < push_constants.minVerticesForAndrew) {
        for (int i = 0; i < n; ++i) {
            resultHullArray[regionId * push_constants.maxHullPoints + i]
                = boundaryPointList[baseIdx + i];
        }
        hullCount[regionId] = uint(n);
        return;
    }

    // load into a local array for sorting
    vec2 pts[MAX_HULL_POINTS];
    for (int i = 0; i < n; ++i) {
        pts[i] = boundaryPointList[baseIdx + i];
    }

    // simple O(N^2) sort by x, then y
    for (int i = 0; i < n - 1; ++i) {
        for (int j = i + 1; j < n; ++j) {
            vec2 a = pts[i], b = pts[j];
            if ( (b.x < a.x) || (b.x == a.x && b.y < a.y) ) {
                pts[i] = b;
                pts[j] = a;
            }
        }
    }

    // build lower hull
    lowerSize = 0;
    for (int i = 0; i < n; ++i) {
        vec2 p = pts[i];
        while (lowerSize >= 2 &&
               orientation(lowerHull[lowerSize - 2],
                           lowerHull[lowerSize - 1],
                           p) <= 0.0) {
            lowerSize--;
        }
        lowerHull[lowerSize++] = p;
    }

    // build upper hull
    upperSize = 0;
    for (int i = n - 1; i >= 0; --i) {
        vec2 p = pts[i];
        while (upperSize >= 2 &&
               orientation(upperHull[upperSize - 2],
                           upperHull[upperSize - 1],
                           p) <= 0.0) {
            upperSize--;
        }
        upperHull[upperSize++] = p;
    }

    // write out hull, skipping the last point of each chain (duplicate endpoints)
    int w = 0;
    for (int i = 0; i < lowerSize - 1; ++i) {
        resultHullArray[regionId * push_constants.maxHullPoints + w++] = lowerHull[i];
    }
    for (int i = 0; i < upperSize - 1; ++i) {
        resultHullArray[regionId * push_constants.maxHullPoints + w++] = upperHull[i];
    }

    hullCount[regionId] = uint(w);
}
