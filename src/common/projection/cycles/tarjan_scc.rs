use std::collections::{BTreeMap, BTreeSet};

use super::Adjacency;

/// Finds strongly connected components in one induced directed subgraph.
pub(super) fn tarjan_scc(
    adjacency: &Adjacency,
    vertices: impl IntoIterator<Item = usize>,
) -> Vec<Vec<usize>> {
    let vertices = vertices.into_iter().collect::<BTreeSet<_>>();
    let mut search = TarjanSearch::new(adjacency, vertices);
    search.run()
}

struct TarjanSearch<'a> {
    adjacency: &'a Adjacency,
    vertices: BTreeSet<usize>,
    next_index: usize,
    indices: BTreeMap<usize, usize>,
    lowlinks: BTreeMap<usize, usize>,
    stack: Vec<usize>,
    on_stack: BTreeSet<usize>,
    components: Vec<Vec<usize>>,
}

impl<'a> TarjanSearch<'a> {
    fn new(adjacency: &'a Adjacency, vertices: BTreeSet<usize>) -> Self {
        Self {
            adjacency,
            vertices,
            next_index: 0,
            indices: BTreeMap::new(),
            lowlinks: BTreeMap::new(),
            stack: Vec::new(),
            on_stack: BTreeSet::new(),
            components: Vec::new(),
        }
    }

    fn run(&mut self) -> Vec<Vec<usize>> {
        for vertex in self.vertices.iter().copied().collect::<Vec<_>>() {
            if !self.indices.contains_key(&vertex) {
                self.visit(vertex);
            }
        }

        std::mem::take(&mut self.components)
    }

    fn visit(&mut self, vertex: usize) {
        self.index_vertex(vertex);

        for neighbour in self.neighbours(vertex) {
            self.inspect_neighbour(vertex, neighbour);
        }

        if self.lowlinks.get(&vertex) == self.indices.get(&vertex) {
            self.extract_component(vertex);
        }
    }

    fn index_vertex(&mut self, vertex: usize) {
        self.indices.insert(vertex, self.next_index);
        self.lowlinks.insert(vertex, self.next_index);
        self.next_index += 1;
        self.stack.push(vertex);
        self.on_stack.insert(vertex);
    }

    fn inspect_neighbour(&mut self, vertex: usize, neighbour: usize) {
        if !self.indices.contains_key(&neighbour) {
            self.visit(neighbour);
            if let Some(neighbour_lowlink) = self.lowlinks.get(&neighbour).copied() {
                self.lower_lowlink(vertex, neighbour_lowlink);
            }
        } else if self.on_stack.contains(&neighbour) {
            if let Some(neighbour_index) = self.indices.get(&neighbour).copied() {
                self.lower_lowlink(vertex, neighbour_index);
            }
        }
    }

    fn lower_lowlink(&mut self, vertex: usize, candidate: usize) {
        if let Some(lowlink) = self.lowlinks.get_mut(&vertex) {
            *lowlink = (*lowlink).min(candidate);
        }
    }

    fn extract_component(&mut self, root: usize) {
        let mut component = Vec::new();

        while let Some(vertex) = self.stack.pop() {
            self.on_stack.remove(&vertex);
            component.push(vertex);
            if vertex == root {
                break;
            }
        }

        component.sort_unstable();
        self.components.push(component);
    }

    fn neighbours(&self, vertex: usize) -> Vec<usize> {
        self.adjacency
            .get(&vertex)
            .into_iter()
            .flatten()
            .filter(|neighbour| self.vertices.contains(neighbour))
            .copied()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::tarjan_scc;

    fn graph(entries: &[(usize, &[usize])]) -> BTreeMap<usize, BTreeSet<usize>> {
        entries
            .iter()
            .map(|(source, targets)| (*source, targets.iter().copied().collect()))
            .collect()
    }

    #[test]
    fn separates_components_from_acyclic_vertices() {
        let adjacency = graph(&[
            (0, &[1]),
            (1, &[2]),
            (2, &[0, 3]),
            (3, &[4]),
            (4, &[3]),
            (5, &[]),
        ]);

        let mut components = tarjan_scc(&adjacency, adjacency.keys().copied());
        components.sort();

        assert_eq!(components, [vec![0, 1, 2], vec![3, 4], vec![5]]);
    }

    #[test]
    fn honors_the_requested_induced_subgraph() {
        let adjacency = graph(&[(0, &[1]), (1, &[0, 2]), (2, &[3]), (3, &[2])]);

        let mut components = tarjan_scc(&adjacency, [1, 2, 3]);
        components.sort();

        assert_eq!(components, [vec![1], vec![2, 3]]);
    }

    #[test]
    fn returns_no_components_for_an_empty_vertex_set() {
        assert!(tarjan_scc(&BTreeMap::new(), []).is_empty());
    }
}
