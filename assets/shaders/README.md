### SIMD BITCH MODE

**Context refresh**

* **Today:** everything runs as *pure* GLSL (330 → 110), hot-reloadable, Shadertoy-friendly.
* **Someday:** you’ll translate the *same* branch-masked math 1-for-1 into CPU intrinsics for platforms with no programmable GPU (e.g. NDS).
* **Rule:** no “halfway” off-loading; a calc only leaves the shader when you deliberately create the CPU variant.

### 0 Glossary (plain words)

| Term                       | Meaning                                                                                                    |
| -- | - |
| **Lane / SIMD lane**       | One pixel’s worth of data inside the GPU’s 4-wide (Intel) or 32-wide (NVIDIA) vector machine.              |
| **Divergence**             | Some lanes take the *if* branch, others don’t → they serialize and waste cycles.                           |
| **Masking**                | Replace a branch with `mix()` / `step()` so every lane runs the same instructions; data decide the result. |
| **Trip count**             | How many times a loop body actually runs.  Fixed trip count = compiler can unroll; variable = harder.      |
| **CAP**                    | A *hard* maximum loop count you set so the compiler sees a fixed trip count (e.g. `DEPTH_CAP = 128`).      |
| **Unroll**                 | Write the loop body multiple times or use `#pragma unroll`. Helps fill vector units.                       |
| **Hoist (⚠️ future-only)** | Move math to an earlier stage (vertex/CPU). *We are **not** doing this in the GPU build.*                  |

### 1  GPU-SIMD Rules of Thumb

1. **Fixed loops only**
   *Give every loop an explicit constant bound* (`for(int i=0;i<16;++i)`).
   If logically variable, add a **CAP** and use masking to bail out early.

2. **Mask instead of branch**
   *Use* `step()`, `mix()`, `clamp()`, `sign()`, `>=` *expressions* instead of `if/else`.
   Typical pattern:

```glsl
   float keep = step(threshold, value); // 0 or 1
   vec3 result = mix(result, new_value, keep);
   float active_lane *= 1.0 - keep; // disable lane
```

3. **Pack similar work together**
   ✦ Marching ice: sample in **STRIDE** chunks (e.g. 8) so the outer loop trip count is small and fixed.
   ✦ After a hit, optional 2-3 extra samples inside that chunk for accuracy.

4. **Avoid integer ops in 110 path**
   Convert `ivec` → `float`, encode lookup tables in textures.  Keeps the 110 fallback identical to 330.

5. **Prefer math to texture *only* when cheaper**
   A trig call on desktop ≈ a texture fetch; on older GPUs trig may dominate.  Benchmark at 256 × 384.

6. **Keep uniform blocks small & static**
   · Fixed-size arrays (`vec2 onsets[16]`).
   · Scalar uniforms for truly frame-constant stuff (`iTime`, `bpm`).
   · Everything else stays inside the shader math so hot-reload shows full effect.

7. **Profile by budget**
   256 × 384 = 98 k pixels.
   60 FPS budget per pixel ≈ **165 ns** on GPU.
   Use a timer query or look at fan noise: if you gain < 10 ns/px, optimisation not worth the clarity hit.

### 2  Workflow Checklist

1. **Write** the effect in GLSL 330 with the above rules.
2. **Toggle** KHR\_debug GPU timers → check ms/frame.
3. If slow → identify hot loop/function with Nsight or RenderDoc statistics.
4. **Apply** mask-unroll-cap techniques there only.
5. **Port** to GLSL 110:

    * replace `uint/int` math,
    * remove `in/out` layout qualifiers,
    * keep same fixed loops & masking.
6. **Bank** this version as the “GPU edition”.
7. When ready for CPU-only build *(future)*:

    * Translate the same masked loops to SSE/NEON intrinsics.
    * Only then consider hoisting per-frame constants to CPU.


### 3 ⃣  Quick Decision Tree

```text
Is the calculation the same for every pixel?
├── Yes → Precompute ON GPU (vertex stage or LUT texture)
└── No
    └── Does it branch per pixel?
        ├── Yes
        │    └── Can I mask it? (step/mix)
        │        ├── Yes → Mask it
        │        └── No
        │            └── Can I cap & unroll loop?
        │                ├── Yes → Fixed CAP + mask
        │                └── No
        │                    └── Performance fine?
        │                        ├── Yes → Leave as is
        │                        └── No → (Future CPU variant will handle it) → Accept cost for now
        └── No
             └── Performance fine?
                 ├── Yes → Leave as is
                 └── No → (Future CPU variant will handle it) → Accept cost for now

```

### 4 Mindset
* **Two clean lanes:**
  *“GPU build = 100 % shader; CPU build = 100 % intrinsics.”*
  Never half-mix unless you intentionally create a hybrid prototype.

* **SIMD-style today helps tomorrow**—masking & fixed loops translate directly to `vblendvps` / `vbsl` etc.

* **Learning focus:** treat every optimisation as *shader code-level* first; postponing hoisting keeps hot-reload and Shadertoy parity intact.


SNES	Tile/sprite	No mapping possible
Genesis	Tile/sprite	No mapping possible
TurboGrafx-16	Tile/sprite	No mapping possible
PlayStation	Fixed-function 3D	Limited (OpenGL 1.0 via emulation)
Saturn	Quad-based 2D+3D	Extremely limited
Nintendo 64	RCP microcode	Partial OpenGL 1.0 approximation
Dreamcast	PowerVR2	OpenGL 1.1 mapping possible
PlayStation 2	Vector units + GS	Partial OpenGL 1.1 (manual transforms)
Xbox	NV2A (GF3)	OpenGL 1.3–1.5 via wrappers
GameCube	ATI Flipper	OpenGL 1.3-like via GX API
PSP	Fixed-function GU	Similar to OpenGL ES 1.0
Nintendo DS	Fixed-pipeline 3D	Basic OpenGL 1.0 emulation
Xbox 360	ATI Xenos	OpenGL 2.1–3.3 equivalency possible
PlayStation 3	NVIDIA RSX	OpenGL 2.1–3.3 compatible
Wii	ATI Hollywood	OpenGL 1.1–1.3 approximation