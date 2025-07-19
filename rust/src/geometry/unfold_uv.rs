use raylib::ffi::MemAlloc;
use raylib::math::glam::Vec3;
use raylib::models::{Mesh, RaylibMesh, WeakMesh};
use std::cmp::Ordering;
use std::collections::{HashMap, VecDeque};
use std::f32::consts::PI;
use std::hash::Hash;
use std::mem::zeroed;
use std::ptr::null_mut;

fn demo(mesh: &mut Mesh) {
    let vertices = unsafe { std::slice::from_raw_parts(mesh.vertices, mesh.vertexCount as usize * 3) };
    let peel_options = OrangePeelOptions {
        gore_count: 8,
        meridian_band_fraction: 0.25,
    };
    let cut_config = sphere_orange_peel_cut_config(vertices, mesh.indices_mut(), peel_options);
    let layout = UnfoldLayout {
        padding: 0.0,    // set to 0.0 if you want strips touching
        page_width: 2.0, // reduce or enlarge horizontal wrap width
        auto_scale_to_fit: true,
        target_max_extent: 1.5,
        recenter: true,
        packing: PackingStrategy::Shelf,
        grid_aspect: 1.0,
    };
}

pub trait Vec3Extensions {
    fn normalize_or_zero(self) -> Self;
}
impl Vec3Extensions for Vec3 {
    fn normalize_or_zero(self) -> Self {
        let len = self.length();
        if len > 1e-12 {
            self / len
        } else {
            self
        }
    }
}
pub enum PackingStrategy {
    Shelf,
    Grid,
    SingleRow,
}

pub struct UnfoldLayout {
    pub padding: f32,
    pub page_width: f32, // if <= 0 and Shelf: auto computed
    pub auto_scale_to_fit: bool,
    pub target_max_extent: f32,
    pub recenter: bool,
    pub packing: PackingStrategy,
    pub grid_aspect: f32, // desired width/height ratio for grid (1.0 = square)
}
impl Default for UnfoldLayout {
    fn default() -> Self {
        Self {
            padding: 0.5,
            page_width: 8.0,
            auto_scale_to_fit: true,
            target_max_extent: 4.0,
            recenter: true,
            packing: PackingStrategy::Shelf,
            grid_aspect: 1.0,
        }
    }
}

pub struct CutConfig {
    pub angle_limit: f32,
    pub max_faces_in_patch: Option<usize>,
    pub user_edge_cut: Option<Box<dyn Fn(usize, usize, (u16, u16), f32) -> bool>>,
}
impl Default for CutConfig {
    fn default() -> Self {
        Self {
            angle_limit: 1.2,
            max_faces_in_patch: None,
            user_edge_cut: None,
        }
    }
}

pub struct OrangePeelOptions {
    pub gore_count: usize,
    pub meridian_band_fraction: f32, // fraction of half distance between meridians
}
impl Default for OrangePeelOptions {
    fn default() -> Self {
        Self {
            gore_count: 8,
            meridian_band_fraction: 0.25,
        }
    }
}

struct DisjointSet {
    parent: Vec<usize>,
    rank: Vec<u8>,
    size: Vec<usize>,
}
impl DisjointSet {
    fn new(count: usize) -> Self {
        Self {
            parent: (0..count).collect(),
            rank: vec![0; count],
            size: vec![1; count],
        }
    }
    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] == x {
            x
        } else {
            let r = self.find(self.parent[x]);
            self.parent[x] = r;
            r
        }
    }
    fn union(&mut self, a: usize, b: usize) -> (usize, usize, usize) {
        let mut ra = self.find(a);
        let mut rb = self.find(b);
        if ra == rb {
            return (ra, rb, ra);
        }
        if self.rank[ra] < self.rank[rb] {
            std::mem::swap(&mut ra, &mut rb);
        }
        self.parent[rb] = ra;
        if self.rank[ra] == self.rank[rb] {
            self.rank[ra] += 1;
        }
        self.size[ra] += self.size[rb];
        (ra, rb, ra)
    }
    fn component_size(&mut self, x: usize) -> usize {
        let r = self.find(x);
        self.size[r]
    }
}

struct DualEdge {
    face_a: usize,
    face_b: usize,
    shared_va: u16,
    shared_vb: u16,
    dihedral: f32,
}

fn compute_dual_edges(indices: &[u16], face_count: usize, positions: &[f32]) -> Vec<DualEdge> {
    fn vertex3(positions: &[f32], i: usize) -> Vec3 {
        Vec3::new(positions[i * 3], positions[i * 3 + 1], positions[i * 3 + 2])
    }
    fn triangle_normal(p: &[f32], a: usize, b: usize, c: usize) -> Vec3 {
        (vertex3(p, b) - vertex3(p, a))
            .cross(vertex3(p, c) - vertex3(p, a))
            .normalize_or_zero()
    }

    let mut edge_owner: HashMap<(u16, u16), usize> = HashMap::new();
    let mut result = Vec::new();
    for face_index in 0..face_count {
        let tri = &indices[face_index * 3..face_index * 3 + 3];
        for e in 0..3 {
            let a = tri[e];
            let b = tri[(e + 1) % 3];
            let key = if a < b { (a, b) } else { (b, a) };
            if let Some(&other_face) = edge_owner.get(&key) {
                let normal_a = triangle_normal(positions, tri[0] as usize, tri[1] as usize, tri[2] as usize);
                let other_tri = &indices[other_face * 3..other_face * 3 + 3];
                let normal_b = triangle_normal(
                    positions,
                    other_tri[0] as usize,
                    other_tri[1] as usize,
                    other_tri[2] as usize,
                );
                let dihedral = normal_a.dot(normal_b).clamp(-1.0, 1.0).acos();
                result.push(DualEdge {
                    face_a: other_face,
                    face_b: face_index,
                    shared_va: key.0,
                    shared_vb: key.1,
                    dihedral,
                });
            } else {
                edge_owner.insert(key, face_index);
            }
        }
    }
    result
}

fn ensure_indices(mesh: &mut WeakMesh) {
    if mesh.indices.is_null() {
        let vertex_count = mesh.vertexCount as usize;
        unsafe {
            let buffer = MemAlloc((vertex_count * size_of::<u16>()) as u32) as *mut u16;
            for i in 0..vertex_count {
                *buffer.add(i) = i as u16;
            }
            mesh.indices = buffer;
            mesh.triangleCount = (vertex_count / 3) as i32;
        }
    }
}

pub fn sphere_orange_peel_cut_config(positions: &[f32], indices: &[u16], options: OrangePeelOptions) -> CutConfig {
    let vertex_count = positions.len() / 3;
    let mut polar = Vec::with_capacity(vertex_count);
    let mut azimuth = Vec::with_capacity(vertex_count);

    for i in 0..vertex_count {
        let x = positions[i * 3];
        let y = positions[i * 3 + 1];
        let z = positions[i * 3 + 2];
        let r = (x * x + y * y + z * z).sqrt().max(1e-9);
        let th = (y / r).clamp(-1.0, 1.0).acos();
        let mut ph = z.atan2(x);
        if ph < 0.0 {
            ph += 2.0 * PI;
        }
        polar.push(th);
        azimuth.push(ph);
    }

    let mut ring_index = vec![0usize; vertex_count];
    {
        let mut unique = polar.clone();
        unique.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
        unique.dedup_by(|a, b| (*a - *b).abs() < 1e-5);
        for (i, th) in polar.iter().enumerate() {
            let mut bucket = 0usize;
            for (k, val) in unique.iter().enumerate() {
                if (th - val).abs() < 1e-5 {
                    bucket = k;
                    break;
                }
            }
            ring_index[i] = bucket;
        }
    }

    let meridian_count = options.gore_count.max(1);
    let mut meridians = Vec::with_capacity(meridian_count);
    for k in 0..meridian_count {
        meridians.push(k as f32 * (2.0 * PI / meridian_count as f32));
    }
    let half_gap = PI / meridian_count as f32;
    let meridian_band_half_width = half_gap * options.meridian_band_fraction.clamp(0.01, 1.0);

    let max_ring = *ring_index.iter().max().unwrap_or(&0);
    let mut ring_min_phi_vertex: Vec<Option<usize>> = vec![None; max_ring + 1];
    for v in 0..vertex_count {
        let r = ring_index[v];
        if let Some(current) = ring_min_phi_vertex[r] {
            if azimuth[v] < azimuth[current] {
                ring_min_phi_vertex[r] = Some(v);
            }
        } else {
            ring_min_phi_vertex[r] = Some(v);
        }
    }

    CutConfig {
        angle_limit: f32::INFINITY,
        max_faces_in_patch: None,
        user_edge_cut: Some(Box::new(move |_fa, _fb, (va, vb), _angle| {
            let a = va as usize;
            let b = vb as usize;

            // average azimuth robust to wrap
            let da = azimuth[a];
            let db = azimuth[b];
            let mut avg_phi = (da + db) * 0.5;
            if (da - db).abs() > PI {
                if da > db {
                    avg_phi = ((da - 2.0 * PI) + db) * 0.5;
                } else {
                    avg_phi = (da + (db - 2.0 * PI)) * 0.5;
                }
                if avg_phi < 0.0 {
                    avg_phi += 2.0 * PI;
                }
            }

            let mut is_meridian_cut = false;
            for &m in &meridians {
                let mut d = (avg_phi - m).abs();
                if d > PI {
                    d = 2.0 * PI - d;
                }
                if d < meridian_band_half_width {
                    is_meridian_cut = true;
                    break;
                }
            }

            let same_ring = ring_index[a] == ring_index[b];
            let ring_break = if same_ring {
                if let Some(vmin) = ring_min_phi_vertex[ring_index[a]] {
                    vmin == a || vmin == b
                } else {
                    false
                }
            } else {
                false
            };

            is_meridian_cut || ring_break
        })),
    }
}

/* ---------------- Unfold + Pack ---------------- */

pub struct UnfoldResult {
    pub mesh: Mesh,
    pub patch_root_faces: Vec<usize>,
}

pub fn unfold_mesh_with_cuts(source: &mut WeakMesh, cut_config: &CutConfig, layout: &UnfoldLayout) -> UnfoldResult {
    ensure_indices(source);

    let face_count = source.triangleCount as usize;
    let vertex_count = source.vertexCount as usize;
    if face_count == 0 {
        return UnfoldResult {
            mesh: unsafe { zeroed() },
            patch_root_faces: Vec::new(),
        };
    }

    let positions = unsafe { std::slice::from_raw_parts(source.vertices, vertex_count * 3) };
    let indices = unsafe { std::slice::from_raw_parts(source.indices, face_count * 3) };

    fn v3(p: &[f32], i: usize) -> Vec3 {
        Vec3::new(p[i * 3], p[i * 3 + 1], p[i * 3 + 2])
    }

    let mut local_face_2d: Vec<[[f32; 2]; 3]> = Vec::with_capacity(face_count);
    for f in 0..face_count {
        let tri = &indices[f * 3..f * 3 + 3];
        let a = tri[0] as usize;
        let b = tri[1] as usize;
        let c = tri[2] as usize;
        let pa = v3(positions, a);
        let pb = v3(positions, b);
        let pc = v3(positions, c);
        let edge_ab = pb - pa;
        let axis_x = edge_ab.normalize_or_zero();
        let normal = edge_ab.cross(pc - pa).normalize_or_zero();
        let axis_y = normal.cross(axis_x);
        let la = [0.0, 0.0];
        let lb = [edge_ab.length(), 0.0];
        let rel_c = pc - pa;
        let lc = [rel_c.dot(axis_x), rel_c.dot(axis_y)];
        local_face_2d.push([la, lb, lc]);
    }

    let mut dual_edges = compute_dual_edges(indices, face_count, positions);
    dual_edges.sort_by(|a, b| a.dihedral.partial_cmp(&b.dihedral).unwrap_or(Ordering::Equal));

    let mut dsu = DisjointSet::new(face_count);
    let mut parent_face: Vec<Option<(usize, u16, u16)>> = vec![None; face_count];
    let mut is_root = vec![true; face_count];

    for edge in &dual_edges {
        let should_cut = cut_config
            .user_edge_cut
            .as_ref()
            .map(|f| {
                f(
                    edge.face_a,
                    edge.face_b,
                    (edge.shared_va, edge.shared_vb),
                    edge.dihedral,
                )
            })
            .unwrap_or(false)
            || edge.dihedral > cut_config.angle_limit;

        if should_cut {
            continue;
        }

        let comp_a = dsu.find(edge.face_a);
        let comp_b = dsu.find(edge.face_b);
        if comp_a == comp_b {
            continue;
        }

        if let Some(max_faces) = cut_config.max_faces_in_patch {
            let size_a = dsu.component_size(comp_a);
            let size_b = dsu.component_size(comp_b);
            if size_a + size_b > max_faces {
                continue;
            }
        }

        parent_face[edge.face_b] = Some((edge.face_a, edge.shared_va, edge.shared_vb));
        is_root[edge.face_b] = false;
        dsu.union(comp_a, comp_b);
    }

    let mut children: Vec<Vec<usize>> = vec![Vec::new(); face_count];
    for f in 0..face_count {
        if let Some((p, _, _)) = parent_face[f] {
            children[p].push(f);
        }
    }

    let mut placed_triangle: Vec<[[f32; 2]; 3]> = vec![[[0.0; 2]; 3]; face_count];
    let mut face_positioned = vec![false; face_count];
    let mut stack = Vec::new();
    let mut patch_roots = Vec::new();

    for f in 0..face_count {
        if is_root[f] {
            patch_roots.push(f);
            placed_triangle[f] = local_face_2d[f];
            face_positioned[f] = true;
            stack.push(f);
            while let Some(current) = stack.pop() {
                for &child in &children[current] {
                    if face_positioned[child] {
                        continue;
                    }
                    if let Some((parent, shared_va, shared_vb)) = parent_face[child] {
                        let parent_tri = &indices[parent * 3..parent * 3 + 3];
                        let child_tri = &indices[child * 3..child * 3 + 3];

                        let mut parent_a_2d = [0.0; 2];
                        let mut parent_b_2d = [0.0; 2];
                        for i in 0..3 {
                            let v = parent_tri[i];
                            if v == shared_va {
                                parent_a_2d = placed_triangle[parent][i];
                            }
                            if v == shared_vb {
                                parent_b_2d = placed_triangle[parent][i];
                            }
                        }
                        let child_local = local_face_2d[child];
                        let mut child_a_local = [0.0; 2];
                        let mut child_b_local = [0.0; 2];
                        for i in 0..3 {
                            let v = child_tri[i];
                            if v == shared_va {
                                child_a_local = child_local[i];
                            } else if v == shared_vb {
                                child_b_local = child_local[i];
                            }
                        }

                        let local_edge = [child_b_local[0] - child_a_local[0], child_b_local[1] - child_a_local[1]];
                        let global_edge = [parent_b_2d[0] - parent_a_2d[0], parent_b_2d[1] - parent_a_2d[1]];

                        let local_len = (local_edge[0] * local_edge[0] + local_edge[1] * local_edge[1])
                            .sqrt()
                            .max(1e-12);
                        let global_len = (global_edge[0] * global_edge[0] + global_edge[1] * global_edge[1])
                            .sqrt()
                            .max(1e-12);
                        let local_dir = [local_edge[0] / local_len, local_edge[1] / local_len];
                        let global_dir = [global_edge[0] / global_len, global_edge[1] / global_len];
                        let cos_t = local_dir[0] * global_dir[0] + local_dir[1] * global_dir[1];
                        let sin_t = local_dir[0] * global_dir[1] - local_dir[1] * global_dir[0];
                        let scale = global_len / local_len;

                        let mut new_positions = [[0.0; 2]; 3];
                        for i in 0..3 {
                            let cp = child_local[i];
                            let x = (cp[0] - child_a_local[0]) * scale;
                            let y = (cp[1] - child_a_local[1]) * scale;
                            let xr = x * cos_t - y * sin_t;
                            let yr = x * sin_t + y * cos_t;
                            new_positions[i] = [parent_a_2d[0] + xr, parent_a_2d[1] + yr];
                        }
                        placed_triangle[child] = new_positions;
                        face_positioned[child] = true;
                        stack.push(child);
                    }
                }
            }
        }
    }

    let mut root_of_face = vec![usize::MAX; face_count];
    for f in 0..face_count {
        let mut r = f;
        while !is_root[r] {
            r = parent_face[r].unwrap().0;
        }
        root_of_face[f] = r;
    }

    // Patch bounds
    let mut patch_bounds: HashMap<usize, ([f32; 2], [f32; 2])> = HashMap::new();
    for f in 0..face_count {
        let root = root_of_face[f];
        let tri = placed_triangle[f];
        let entry = patch_bounds.entry(root).or_insert(([f32::MAX; 2], [f32::MIN; 2]));
        for p in tri {
            if p[0] < entry.0[0] {
                entry.0[0] = p[0];
            }
            if p[1] < entry.0[1] {
                entry.0[1] = p[1];
            }
            if p[0] > entry.1[0] {
                entry.1[0] = p[0];
            }
            if p[1] > entry.1[1] {
                entry.1[1] = p[1];
            }
        }
    }

    // Packing
    // Decide packing order (largest area first)
    let mut patch_order: Vec<usize> = patch_bounds.keys().copied().collect();
    patch_order.sort_by(|a, b| {
        let ba = patch_bounds.get(a).unwrap();
        let bb = patch_bounds.get(b).unwrap();
        let area_a = (ba.1[0] - ba.0[0]) * (ba.1[1] - ba.0[1]);
        let area_b = (bb.1[0] - bb.0[0]) * (bb.1[1] - bb.0[1]);
        area_b.partial_cmp(&area_a).unwrap_or(Ordering::Equal)
    });

    let padding = layout.padding.max(0.0);
    let mut patch_offset: HashMap<usize, [f32; 2]> = HashMap::new();

    match layout.packing {
        PackingStrategy::SingleRow => {
            let mut cursor_x = 0.0;
            for root in &patch_order {
                let (min, max) = patch_bounds[root];
                patch_offset.insert(*root, [cursor_x - min[0], -min[1]]);
                let width = (max[0] - min[0]) + padding;
                cursor_x += width;
            }
        },
        PackingStrategy::Shelf => {
            // compute auto page width if requested
            let auto_page_width = if layout.page_width <= 0.0 {
                let mut total_area = 0.0;
                for r in &patch_order {
                    let (min, max) = patch_bounds[r];
                    let w = (max[0] - min[0]) + padding;
                    let h = (max[1] - min[1]) + padding;
                    total_area += w * h;
                }
                total_area.sqrt() // near-square
            } else {
                layout.page_width
            };

            let page_width = auto_page_width.max(1e-3);
            let mut cursor_x = 0.0;
            let mut cursor_y = 0.0;
            let mut current_row_height = 0.0;

            for root in &patch_order {
                let (min, max) = patch_bounds[root];
                let width = max[0] - min[0];
                let height = max[1] - min[1];
                let needed = width + padding;
                if cursor_x > 0.0 && cursor_x + needed > page_width {
                    cursor_x = 0.0;
                    cursor_y += current_row_height + padding;
                    current_row_height = 0.0;
                }
                patch_offset.insert(*root, [cursor_x - min[0], cursor_y - min[1]]);
                cursor_x += needed;
                if height > current_row_height {
                    current_row_height = height;
                }
            }
        },
        PackingStrategy::Grid => {
            let count = patch_order.len().max(1);
            let aspect = layout.grid_aspect.max(0.1);
            let cols_f = (count as f32 * aspect).sqrt();
            let mut cols = cols_f.ceil() as usize;
            cols = cols.max(1);
            let rows = ((count + cols - 1) / cols).max(1);

            // compute cell dimensions (max width/height among patches)
            let mut cell_w = 0.0;
            let mut cell_h = 0.0;
            for r in &patch_order {
                let (min, max) = patch_bounds[r];
                let w = max[0] - min[0];
                let h = max[1] - min[1];
                if w > cell_w {
                    cell_w = w;
                }
                if h > cell_h {
                    cell_h = h;
                }
            }
            cell_w += padding;
            cell_h += padding;

            for (i, root) in patch_order.iter().enumerate() {
                let col = i % cols;
                let row = i / cols;
                let (min, _) = patch_bounds[root];
                let off_x = col as f32 * cell_w - min[0];
                let off_y = row as f32 * cell_h - min[1];
                patch_offset.insert(*root, [off_x, off_y]);
            }
        },
    }

    // Emit flattened mesh
    let mut output_positions = Vec::<f32>::with_capacity(face_count * 9);
    let mut output_indices = Vec::<u16>::with_capacity(face_count * 3);
    let mut remap: HashMap<(u32, usize), u16> = HashMap::new();
    for f in 0..face_count {
        let root = root_of_face[f];
        let offset = patch_offset[&root];
        let tri_indices = &indices[f * 3..f * 3 + 3];
        for corner in 0..3 {
            let v = tri_indices[corner] as u32;
            let key = (v, root);
            let index = *remap.entry(key).or_insert_with(|| {
                let corner_pos = placed_triangle[f][corner];
                let new_index = (output_positions.len() / 3) as u16;
                output_positions.extend_from_slice(&[corner_pos[0] + offset[0], corner_pos[1] + offset[1], 0.0]);
                new_index
            });
            output_indices.push(index);
        }
    }

    // Optional overall scaling + recenter
    if layout.auto_scale_to_fit || layout.recenter {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for i in 0..(output_positions.len() / 3) {
            let x = output_positions[i * 3];
            let y = output_positions[i * 3 + 1];
            if x < min_x {
                min_x = x;
            }
            if x > max_x {
                max_x = x;
            }
            if y < min_y {
                min_y = y;
            }
            if y > max_y {
                max_y = y;
            }
        }
        let mut scale = 1.0;
        if layout.auto_scale_to_fit {
            let extent_x = (max_x - min_x).max(1e-6);
            let extent_y = (max_y - min_y).max(1e-6);
            let max_extent = extent_x.max(extent_y);
            scale = layout.target_max_extent / max_extent;
        }
        let center_x = 0.5 * (min_x + max_x);
        let center_y = 0.5 * (min_y + max_y);
        for i in 0..(output_positions.len() / 3) {
            if layout.recenter {
                output_positions[i * 3] = (output_positions[i * 3] - center_x) * scale;
                output_positions[i * 3 + 1] = (output_positions[i * 3 + 1] - center_y) * scale;
            } else if layout.auto_scale_to_fit {
                output_positions[i * 3] *= scale;
                output_positions[i * 3 + 1] *= scale;
            }
        }
    }

    // Build Mesh
    let mut flattened_mesh: Mesh = unsafe { zeroed() };
    flattened_mesh.vertexCount = (output_positions.len() / 3) as i32;
    flattened_mesh.triangleCount = (output_indices.len() / 3) as i32;
    unsafe {
        flattened_mesh.vertices = MemAlloc((output_positions.len() * size_of::<f32>()) as u32) as *mut f32;
        std::ptr::copy_nonoverlapping(
            output_positions.as_ptr(),
            flattened_mesh.vertices,
            output_positions.len(),
        );
        flattened_mesh.indices = MemAlloc((output_indices.len() * size_of::<u16>()) as u32) as *mut u16;
        std::ptr::copy_nonoverlapping(output_indices.as_ptr(), flattened_mesh.indices, output_indices.len());
        flattened_mesh.texcoords = null_mut();
        flattened_mesh.normals = null_mut();
        flattened_mesh.tangents = null_mut();
        flattened_mesh.colors = null_mut();
        flattened_mesh.upload(false);
    }

    UnfoldResult {
        mesh: flattened_mesh,
        patch_root_faces: patch_roots,
    }
}

pub fn unfold_mesh_old(src: &mut WeakMesh) -> Mesh {
    const THRESH: f32 = 1.2;
    const PAD: f32 = 1.0;
    const PAGE_W: f32 = 10.0;
    const DO_SCALE: bool = false; // set true if you want uniform scaling
    const TARGET_HALF_EXTENT: f32 = 1.0; // used only if DO_SCALE

    if src.indices.is_null() {
        let vc = src.vertexCount as usize;
        unsafe {
            let ib = MemAlloc((vc * size_of::<u16>()) as u32) as *mut u16;
            for i in 0..vc {
                *ib.add(i) = i as u16;
            }
            src.indices = ib;
            src.triangleCount = (vc / 3) as i32;
        }
    }

    let fc = src.triangleCount as usize;
    let vc = src.vertexCount as usize;
    let pos = unsafe { std::slice::from_raw_parts(src.vertices, vc * 3) };
    let idx = unsafe { std::slice::from_raw_parts(src.indices, fc * 3) };

    let mut nbr = vec![Vec::<usize>::new(); fc];
    {
        let mut edge_owner: HashMap<(u16, u16), usize> = HashMap::new();
        for f in 0..fc {
            let t = &idx[f * 3..f * 3 + 3];
            for e in 0..3 {
                let a = t[e];
                let b = t[(e + 1) % 3];
                let k = if a < b { (a, b) } else { (b, a) };
                if let Some(&o) = edge_owner.get(&k) {
                    nbr[f].push(o);
                    nbr[o].push(f);
                } else {
                    edge_owner.insert(k, f);
                }
            }
        }
    }

    #[inline]
    fn v3(p: &[f32], i: usize) -> Vec3 {
        Vec3::new(p[i * 3], p[i * 3 + 1], p[i * 3 + 2])
    }
    #[inline]
    fn n(a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
        (b - a).cross(c - a)
    }

    struct Local([[f32; 2]; 3]);
    let locals: Vec<Local> = (0..fc)
        .map(|f| {
            let t = &idx[f * 3..f * 3 + 3];
            let (ia, ib, ic) = (t[0] as usize, t[1] as usize, t[2] as usize);
            let pa = v3(pos, ia);
            let pb = v3(pos, ib);
            let pc = v3(pos, ic);
            let e0 = pb - pa;
            let x = e0.normalize_or_zero();
            let norm = e0.cross(pc - pa).normalize_or_zero();
            let y = norm.cross(x);
            let la = [0.0, 0.0];
            let lb = [e0.length(), 0.0];
            let r = pc - pa;
            let lc = [r.dot(x), r.dot(y)];
            Local([la, lb, lc])
        })
        .collect();

    let mut placed = vec![false; vc];
    let mut planar = vec![[0.0f32; 2]; vc];
    let mut face_done = vec![false; fc];
    let mut out_pos: Vec<f32> = Vec::with_capacity(fc * 9);
    let mut out_idx: Vec<u16> = Vec::with_capacity(fc * 3);
    let mut remap: HashMap<(u32, u32), u16> = HashMap::new();
    let mut island_id: u32 = 1;

    struct Shelf {
        cx: f32,
        cy: f32,
        sh: f32,
    }
    impl Shelf {
        fn place(&mut self, min: [f32; 2], max: [f32; 2], pad: f32, page_w: f32) -> [f32; 2] {
            let w = (max[0] - min[0]) + pad;
            if self.cx + w > page_w {
                self.cx = 0.0;
                self.cy += self.sh;
                self.sh = 0.0;
            }
            let h = (max[1] - min[1]) + pad;
            let off = [self.cx - min[0], self.cy - min[1]];
            self.cx += w;
            if h > self.sh {
                self.sh = h;
            }
            off
        }
    }
    let mut packer = Shelf {
        cx: 0.0,
        cy: 0.0,
        sh: 0.0,
    };

    for seed in 0..fc {
        if face_done[seed] {
            continue;
        }
        placed.fill(false);
        let mut q = VecDeque::new();
        q.push_back(seed);
        {
            let t = &idx[seed * 3..seed * 3 + 3];
            let loc = &locals[seed].0;
            for c in 0..3 {
                let v = t[c] as usize;
                planar[v] = loc[c];
                placed[v] = true;
            }
            face_done[seed] = true;
        }
        let mut patch_faces = vec![seed];

        while let Some(f) = q.pop_front() {
            for &g in &nbr[f] {
                if face_done[g] {
                    continue;
                }
                let tf = &idx[f * 3..f * 3 + 3];
                let tg = &idx[g * 3..g * 3 + 3];
                let nf = n(
                    v3(pos, tf[0] as usize),
                    v3(pos, tf[1] as usize),
                    v3(pos, tf[2] as usize),
                )
                .normalize_or_zero();
                let ng = n(
                    v3(pos, tg[0] as usize),
                    v3(pos, tg[1] as usize),
                    v3(pos, tg[2] as usize),
                )
                .normalize_or_zero();
                let ang = nf.dot(ng).clamp(-1.0, 1.0).acos();
                if ang > THRESH {
                    continue;
                }

                let tri = tg;
                let mut shared = None;
                for e in 0..3 {
                    let a = tri[e] as usize;
                    let b = tri[(e + 1) % 3] as usize;
                    if placed[a] && placed[b] {
                        shared = Some((a, b, tri[(e + 2) % 3] as usize));
                        break;
                    }
                }
                if let Some((ia, ib, ic)) = shared {
                    let loc = &locals[g].0;
                    let mut la = [0.0; 2];
                    let mut lb = [0.0; 2];
                    let mut lc = [0.0; 2];
                    for c in 0..3 {
                        let v = tri[c] as usize;
                        if v == ia {
                            la = loc[c];
                        } else if v == ib {
                            lb = loc[c];
                        } else if v == ic {
                            lc = loc[c];
                        }
                    }
                    let ga = planar[ia];
                    let gb = planar[ib];
                    let ul = [lb[0] - la[0], lb[1] - la[1]];
                    let ug = [gb[0] - ga[0], gb[1] - ga[1]];
                    let ll = (ul[0] * ul[0] + ul[1] * ul[1]).sqrt().max(1e-12);
                    let lg = (ug[0] * ug[0] + ug[1] * ug[1]).sqrt().max(1e-12);
                    let rl = [ul[0] / ll, ul[1] / ll];
                    let rg = [ug[0] / lg, ug[1] / lg];
                    let cos_t = rl[0] * rg[0] + rl[1] * rg[1];
                    let sin_t = rl[0] * rg[1] - rl[1] * rg[0];
                    let scale = lg / ll;
                    if !placed[ic] {
                        let x = (lc[0] - la[0]) * scale;
                        let y = (lc[1] - la[1]) * scale;
                        planar[ic] = [ga[0] + x * cos_t - y * sin_t, ga[1] + x * sin_t + y * cos_t];
                        placed[ic] = true;
                    }
                    face_done[g] = true;
                    patch_faces.push(g);
                    q.push_back(g);
                }
            }
        }

        let mut min = [f32::MAX; 2];
        let mut max = [f32::MIN; 2];
        let mut any = false;
        for &f in &patch_faces {
            let t = &idx[f * 3..f * 3 + 3];
            for &v in t {
                let id = v as usize;
                if placed[id] {
                    let p = planar[id];
                    if p[0] < min[0] {
                        min[0] = p[0];
                    }
                    if p[1] < min[1] {
                        min[1] = p[1];
                    }
                    if p[0] > max[0] {
                        max[0] = p[0];
                    }
                    if p[1] > max[1] {
                        max[1] = p[1];
                    }
                    any = true;
                }
            }
        }
        if !any {
            continue;
        }
        let offset = packer.place(min, max, PAD, PAGE_W);

        for &f in &patch_faces {
            let t = &idx[f * 3..f * 3 + 3];
            for &v in t {
                let key = (v as u32, island_id);
                let id = *remap.entry(key).or_insert_with(|| {
                    let p = planar[v as usize];
                    let nid = (out_pos.len() / 3) as u16;
                    out_pos.extend_from_slice(&[p[0] + offset[0], p[1] + offset[1], 0.0]);
                    nid
                });
                out_idx.push(id);
            }
        }
        island_id += 1;
    }

    // build mesh
    let mut m: Mesh = unsafe { zeroed() };
    m.vertexCount = (out_pos.len() / 3) as i32;
    m.triangleCount = (out_idx.len() / 3) as i32;
    unsafe {
        m.vertices = MemAlloc((out_pos.len() * size_of::<f32>()) as u32) as *mut f32;
        std::ptr::copy_nonoverlapping(out_pos.as_ptr(), m.vertices, out_pos.len());
        m.indices = MemAlloc((out_idx.len() * size_of::<u16>()) as u32) as *mut u16;
        std::ptr::copy_nonoverlapping(out_idx.as_ptr(), m.indices, out_idx.len());
        m.texcoords = null_mut();
        m.normals = null_mut();
        m.tangents = null_mut();
        m.colors = null_mut();
    }

    // recenter (and optional scale)
    unsafe {
        let vc_out = m.vertexCount as usize;
        if vc_out > 0 {
            let verts = std::slice::from_raw_parts_mut(m.vertices, vc_out * 3);
            let mut minx = f32::MAX;
            let mut maxx = f32::MIN;
            let mut miny = f32::MAX;
            let mut maxy = f32::MIN;
            for i in 0..vc_out {
                let x = verts[i * 3];
                let y = verts[i * 3 + 1];
                if x < minx {
                    minx = x;
                }
                if x > maxx {
                    maxx = x;
                }
                if y < miny {
                    miny = y;
                }
                if y > maxy {
                    maxy = y;
                }
            }
            let cx = 0.5 * (minx + maxx);
            let cy = 0.5 * (miny + maxy);
            let mut scale = 1.0;
            if DO_SCALE {
                let half = 0.5 * ((maxx - minx).max(maxy - miny).max(1e-6));
                scale = TARGET_HALF_EXTENT / half;
            }
            for i in 0..vc_out {
                verts[i * 3] = (verts[i * 3] - cx) * scale;
                verts[i * 3 + 1] = (verts[i * 3 + 1] - cy) * scale;
            }
        }
        m.upload(false);
    }

    m
}
