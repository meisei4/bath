Purpose:  
Build a minimal, inspectable rendering and collision testbed for a game using procedural and signal-based logic Focus is
on GPU-driven rendering, stepwise visual composition, and full debuggability at each stage

### Features

1 Simple Shader Scoping:
-Render simple geometry - quads, strips? simple tessellated surfaces
-Vertex shader handles macro space: projection, subdivision/meshes, macro motion
-Fragment shader handles micro space: color, reflection, blur/resolution, micro motion

2 Affine Projection notes (Mode 7):
-Projecting geometry vs UVs in vertex shader vs just UVs fragment shader
-Must be deterministic, invertible, and synchronizable with CPU physics logic in the future

3 Fragment Shader Effects:
-Procedural textures vs prebaked textures
-Depth-based effects (color ramps, particles, dithering)
-Debug shaders (flat color maps, tiling visualizers, wire frames, barycentric color gradients)

4 Space Debugger:
- Visualize each transformation step
- Overlays for raw UVs, "virtual pixel outlines?", depth gradients, tile lines, collision masks
- Toggleable effects to isolate rendering or logic issues

5 Ultimate goal:

- just learn how tf the render pipeline at a simple level works
- geometry vs uv, clipping, affine, non-affine, linear non-linear, bilinear, blending, masks, feedback channels, etc
  etc)

### Design

- Procedural Input → 2D Illusion Output:  
  Simple signals become visual effects through projection and layering

- Fully Inspectable Pipeline:  
  All transforms are simple, reversible, and traceable for debugging in cpu scanline mock-rasterization (physics),
  vertex staging, and fragment staging

- CPU–GPU Synchronization:  
  physics and gameplay logic share a feedback loop with shader rendering

- Incremental Composition:  
  Build and debug effects step by step with isolated control
