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
    triangle_a: usize,
    triangle_b: usize,
    triangle_a_local_edge: (u8, u8),
    triangle_b_local_edge: (u8, u8),
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
    let triangle_count = welded_mesh.welded_vertices_per_triangle.len();
    let mut welded_edge_to_parent: HashMap<WeldedEdge, (usize, (u8, u8))> = HashMap::new();
    let mut dual_graph = Vec::new();
    for triangle in 0..triangle_count {
        let [welded_vertex_a, welded_vertex_b, welded_vertex_c] = welded_mesh.welded_vertices_per_triangle[triangle];
        let local_edges = [(0u8, 1u8), (1, 2), (2, 0)];
        let welded_vertices = [welded_vertex_a, welded_vertex_b, welded_vertex_c];
        for &(point_a, point_b) in &local_edges {
            let edge = WeldedEdge::new(welded_vertices[point_a as usize], welded_vertices[point_b as usize]);
            if let Some(&(parent_triangle, parent_edge_local)) = welded_edge_to_parent.get(&edge) {
                dual_graph.push(DualEdge {
                    triangle_a: parent_triangle,
                    triangle_b: triangle,
                    welded_edge: edge,
                    triangle_a_local_edge: parent_edge_local,
                    triangle_b_local_edge: (point_a, point_b),
                });
            } else {
                welded_edge_to_parent.insert(edge, (triangle, (point_a, point_b)));
            }
        }
    }
    dual_graph
}

pub fn build_parent_tree(triangle_count: usize, dual_graph: &mut [DualEdge]) -> (Vec<ParentLink>, Vec<Vec<usize>>) {
    // dual_graph.sort_by(|left, right| right.fold_weight.partial_cmp(&left.fold_weight).unwrap());
    //TODO: biggest change for the anchored triangles
    dual_graph.sort_by(|left, right| dual_edge_sorting_order(left).cmp(&dual_edge_sorting_order(right)));
    let mut dsu = DisjointSetUnion::new(triangle_count);
    let mut adjacency_list = vec![Vec::new(); triangle_count];

    for edge in dual_graph.iter().copied() {
        if dsu.union(edge.triangle_a, edge.triangle_b) {
            adjacency_list[edge.triangle_a].push((edge.triangle_b, edge));
            adjacency_list[edge.triangle_b].push((edge.triangle_a, edge));
        }
    }
    for adjacent_triangles in &mut adjacency_list {
        adjacent_triangles
            .sort_by_key(|&(triangle, edge)| (triangle, edge.welded_edge.vertex_a.id, edge.welded_edge.vertex_b.id));
    }
    let mut parent_links = vec![ParentLink::default(); triangle_count];
    let mut children = vec![Vec::new(); triangle_count];
    let mut seen = vec![false; triangle_count]; //TODO: stupid fucking parallel arrays again
    let mut triangle_queue = VecDeque::new();
    for id in 0..triangle_count {
        if seen[id] {
            continue;
        }
        seen[id] = true;
        triangle_queue.push_back(id);
        while let Some(current_triangle) = triangle_queue.pop_front() {
            for &(triangle, edge) in &adjacency_list[current_triangle] {
                if seen[triangle] {
                    continue;
                }
                seen[triangle] = true;
                let (parent_local_edge, child_local_edge) = if current_triangle == edge.triangle_a {
                    (edge.triangle_a_local_edge, edge.triangle_b_local_edge)
                } else {
                    (edge.triangle_b_local_edge, edge.triangle_a_local_edge)
                };
                parent_links[triangle] = ParentLink {
                    parent: Some(current_triangle),
                    parent_local_edge: Some(parent_local_edge),
                    child_local_edge: Some(child_local_edge),
                    welded_edge: Some(edge.welded_edge),
                };
                children[current_triangle].push(triangle);
                triangle_queue.push_back(triangle);
            }
        }
    }
    (parent_links, children)
}

pub fn collect_subtree_triangles(root_triangle: usize, children: &[Vec<usize>]) -> Vec<usize> {
    let mut subtree = Vec::new();
    let mut triangle_stack = vec![root_triangle];
    while let Some(triangle) = triangle_stack.pop() {
        subtree.push(triangle);
        for &child_triangle in &children[triangle] {
            triangle_stack.push(child_triangle);
        }
    }
    subtree
}

pub fn dual_edge_sorting_order(edge: &DualEdge) -> (usize, usize, usize, usize) {
    let welded_edge_vertex_a = edge.welded_edge.vertex_a.id;
    let welded_edge_vertex_b = edge.welded_edge.vertex_b.id;
    let lesser_triangle = edge.triangle_a.min(edge.triangle_b);
    let greater_triangle = edge.triangle_a.max(edge.triangle_b);
    (
        welded_edge_vertex_a,
        welded_edge_vertex_b,
        lesser_triangle,
        greater_triangle,
    )
}
