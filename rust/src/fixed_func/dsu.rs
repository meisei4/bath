use crate::fixed_func::topology::{WeldedEdge, WeldedMesh};
use std::collections::{HashMap, VecDeque};
use std::mem::swap;

pub struct DisjointSetUnion {
    disjoint_sets: Vec<usize>,
    rank: Vec<u8>, //TODO: rank is how many nodes in already exist in a given disjoint set...? I DONT LIKE PARALLEL ARRAYS!!
}
impl DisjointSetUnion {
    fn new(nodes: usize) -> Self {
        Self {
            disjoint_sets: (0..nodes).collect(),
            rank: vec![0; nodes],
        }
    }
    fn find(&mut self, node: usize) -> usize {
        if self.disjoint_sets[node] != node {
            self.disjoint_sets[node] = self.find(self.disjoint_sets[node]);
        }
        self.disjoint_sets[node]
    }
    fn union(&mut self, node_a: usize, node_b: usize) -> bool {
        let (mut a_representative, mut b_representative) = (self.find(node_a), self.find(node_b));
        if a_representative == b_representative {
            return false;
        }
        if self.rank[a_representative] < self.rank[b_representative] {
            swap(&mut a_representative, &mut b_representative);
        }
        self.disjoint_sets[b_representative] = a_representative;
        if self.rank[a_representative] == self.rank[b_representative] {
            self.rank[a_representative] += 1;
        }
        true
    }
}

#[derive(Copy, Clone, Debug)]
pub struct DualEdge {
    face_a: usize,
    face_b: usize,
    face_a_local_edge: (u8, u8),
    face_b_local_edge: (u8, u8),
    welded_edge: WeldedEdge,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct ParentLink {
    pub parent: Option<usize>,
    pub parent_local_edge: Option<(u8, u8)>,
    pub child_local_edge: Option<(u8, u8)>,
    pub welded_edge: Option<WeldedEdge>,
}

pub fn build_dual_graph(welded_mesh: &WeldedMesh) -> Vec<DualEdge> {
    let face_count = welded_mesh.welded_faces.len();
    let mut welded_edge_to_parent: HashMap<WeldedEdge, (usize, (u8, u8))> = HashMap::new();
    let mut dual_graph = Vec::new();
    for face in 0..face_count {
        let [welded_vertex_a, welded_vertex_b, welded_vertex_c] = welded_mesh.welded_faces[face];
        let local_edges = [(0u8, 1u8), (1, 2), (2, 0)];
        let welded_vertices = [welded_vertex_a, welded_vertex_b, welded_vertex_c];
        for &(point_a, point_b) in &local_edges {
            let edge = WeldedEdge::new(welded_vertices[point_a as usize], welded_vertices[point_b as usize]);
            if let Some(&(parent_face, parent_edge_local)) = welded_edge_to_parent.get(&edge) {
                dual_graph.push(DualEdge {
                    face_a: parent_face,
                    face_b: face,
                    welded_edge: edge,
                    face_a_local_edge: parent_edge_local,
                    face_b_local_edge: (point_a, point_b),
                });
            } else {
                welded_edge_to_parent.insert(edge, (face, (point_a, point_b)));
            }
        }
    }
    dual_graph
}

pub fn build_parent_tree(face_count: usize, dual_graph: &mut [DualEdge]) -> (Vec<ParentLink>, Vec<Vec<usize>>) {
    // dual_graph.sort_by(|left, right| right.fold_weight.partial_cmp(&left.fold_weight).unwrap());
    //TODO: biggest change for the anchored faces
    dual_graph.sort_by(|left, right| dual_edge_sorting_order(left).cmp(&dual_edge_sorting_order(right)));
    let mut dsu = DisjointSetUnion::new(face_count);
    let mut adjacency_list = vec![Vec::new(); face_count];

    for edge in dual_graph.iter().copied() {
        if dsu.union(edge.face_a, edge.face_b) {
            adjacency_list[edge.face_a].push((edge.face_b, edge));
            adjacency_list[edge.face_b].push((edge.face_a, edge));
        }
    }
    for adjacent_faces in &mut adjacency_list {
        adjacent_faces.sort_by_key(|&(face, edge)| (face, edge.welded_edge.vertex_a.id, edge.welded_edge.vertex_b.id));
    }
    let mut parent_links = vec![ParentLink::default(); face_count];
    let mut children = vec![Vec::new(); face_count];
    let mut seen = vec![false; face_count]; //TODO: stupid fucking parallel arrays again
    let mut face_queue = VecDeque::new();
    for id in 0..face_count {
        if seen[id] {
            continue;
        }
        seen[id] = true;
        face_queue.push_back(id);
        while let Some(current_face) = face_queue.pop_front() {
            for &(face, edge) in &adjacency_list[current_face] {
                if seen[face] {
                    continue;
                }
                seen[face] = true;
                let (parent_local_edge, child_local_edge) = if current_face == edge.face_a {
                    (edge.face_a_local_edge, edge.face_b_local_edge)
                } else {
                    (edge.face_b_local_edge, edge.face_a_local_edge)
                };
                parent_links[face] = ParentLink {
                    parent: Some(current_face),
                    parent_local_edge: Some(parent_local_edge),
                    child_local_edge: Some(child_local_edge),
                    welded_edge: Some(edge.welded_edge),
                };
                children[current_face].push(face);
                face_queue.push_back(face);
            }
        }
    }
    (parent_links, children)
}

pub fn collect_subtree_faces(root_face: usize, children: &[Vec<usize>]) -> Vec<usize> {
    let mut subtree = Vec::new();
    let mut face_stack = vec![root_face];
    while let Some(face) = face_stack.pop() {
        subtree.push(face);
        for &child_face in &children[face] {
            face_stack.push(child_face);
        }
    }
    subtree
}

pub fn dual_edge_sorting_order(edge: &DualEdge) -> (u32, u32, usize, usize) {
    let welded_edge_vertex_a = edge.welded_edge.vertex_a.id;
    let welded_edge_vertex_b = edge.welded_edge.vertex_b.id;
    let lesser_face = edge.face_a.min(edge.face_b);
    let greater_face = edge.face_a.max(edge.face_b);
    (welded_edge_vertex_a, welded_edge_vertex_b, lesser_face, greater_face)
}
