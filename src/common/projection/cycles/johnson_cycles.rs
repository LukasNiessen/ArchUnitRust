use std::collections::{BTreeMap, BTreeSet};

use super::{Adjacency, tarjan_scc::tarjan_scc};

/// Enumerates every elementary directed cycle exactly once with Johnson's algorithm.
pub(crate) fn johnson_cycles(adjacency: &Adjacency) -> Vec<Vec<usize>> {
    let adjacency = normalize_adjacency(adjacency);
    let vertices = adjacency.keys().copied().collect::<Vec<_>>();
    let mut cycles = Vec::new();
    let mut lower_bound = vertices.first().copied();

    while let Some(bound) = lower_bound {
        let remaining = vertices.iter().copied().filter(|vertex| *vertex >= bound);
        let component = tarjan_scc(&adjacency, remaining)
            .into_iter()
            .filter(|component| component.len() > 1)
            .min_by_key(|component| component.first().copied());

        let Some(component) = component else {
            break;
        };
        let Some(start) = component.first().copied() else {
            break;
        };

        let mut search = CircuitSearch::new(&adjacency, component, start);
        search.circuit(start);
        cycles.extend(search.cycles);
        lower_bound = vertices.iter().copied().find(|vertex| *vertex > start);
    }

    cycles.sort();
    cycles
}

fn normalize_adjacency(adjacency: &Adjacency) -> Adjacency {
    let vertices = adjacency
        .iter()
        .flat_map(|(source, targets)| std::iter::once(*source).chain(targets.iter().copied()))
        .collect::<BTreeSet<_>>();

    vertices
        .into_iter()
        .map(|vertex| {
            let neighbours = adjacency
                .get(&vertex)
                .into_iter()
                .flatten()
                .copied()
                .filter(|neighbour| *neighbour != vertex)
                .collect();
            (vertex, neighbours)
        })
        .collect()
}

struct CircuitSearch<'a> {
    adjacency: &'a Adjacency,
    component: BTreeSet<usize>,
    start: usize,
    stack: Vec<usize>,
    blocked: BTreeSet<usize>,
    blocked_by: BTreeMap<usize, BTreeSet<usize>>,
    cycles: Vec<Vec<usize>>,
}

impl<'a> CircuitSearch<'a> {
    fn new(adjacency: &'a Adjacency, component: Vec<usize>, start: usize) -> Self {
        Self {
            adjacency,
            component: component.into_iter().collect(),
            start,
            stack: Vec::new(),
            blocked: BTreeSet::new(),
            blocked_by: BTreeMap::new(),
            cycles: Vec::new(),
        }
    }

    fn circuit(&mut self, vertex: usize) -> bool {
        self.stack.push(vertex);
        self.blocked.insert(vertex);
        let neighbours = self.component_neighbours(vertex);
        let mut found_cycle = false;

        for neighbour in neighbours.iter().copied() {
            if neighbour == self.start {
                self.cycles.push(self.stack.clone());
                found_cycle = true;
            } else if !self.blocked.contains(&neighbour) && self.circuit(neighbour) {
                found_cycle = true;
            }
        }

        if found_cycle {
            self.unblock(vertex);
        } else {
            for neighbour in neighbours {
                self.blocked_by.entry(neighbour).or_default().insert(vertex);
            }
        }
        self.stack.pop();
        found_cycle
    }

    fn unblock(&mut self, vertex: usize) {
        if !self.blocked.remove(&vertex) {
            return;
        }

        if let Some(dependants) = self.blocked_by.remove(&vertex) {
            for dependant in dependants {
                self.unblock(dependant);
            }
        }
    }

    fn component_neighbours(&self, vertex: usize) -> Vec<usize> {
        self.adjacency
            .get(&vertex)
            .into_iter()
            .flatten()
            .filter(|neighbour| self.component.contains(neighbour))
            .copied()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::johnson_cycles;

    type Adjacency = BTreeMap<usize, BTreeSet<usize>>;

    fn graph(entries: &[(usize, &[usize])]) -> Adjacency {
        entries
            .iter()
            .map(|(source, targets)| (*source, targets.iter().copied().collect()))
            .collect()
    }

    #[test]
    fn returns_no_cycles_for_acyclic_or_self_edge_only_graphs() {
        assert!(johnson_cycles(&graph(&[(0, &[1]), (1, &[2]), (2, &[])])).is_empty());
        assert!(johnson_cycles(&graph(&[(0, &[0]), (1, &[1])])).is_empty());
    }

    #[test]
    fn finds_disconnected_and_overlapping_cycles_exactly_once() {
        let adjacency = graph(&[
            (0, &[1, 2]),
            (1, &[0, 2]),
            (2, &[0, 1]),
            (3, &[4]),
            (4, &[3]),
        ]);

        assert_eq!(
            johnson_cycles(&adjacency),
            [
                vec![0, 1],
                vec![0, 1, 2],
                vec![0, 2],
                vec![0, 2, 1],
                vec![1, 2],
                vec![3, 4],
            ]
        );
    }

    #[test]
    fn enumerates_all_twenty_cycles_in_a_dense_four_vertex_graph() {
        let adjacency = (0..4)
            .map(|source| {
                let targets = (0..4).filter(|target| *target != source).collect();
                (source, targets)
            })
            .collect();

        let cycles = johnson_cycles(&adjacency);

        assert_eq!(cycles.len(), 20);
        assert!(cycles.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(
            cycles.iter().all(|cycle| {
                cycle.iter().copied().collect::<BTreeSet<_>>().len() == cycle.len()
            })
        );
    }

    #[test]
    fn matches_an_exhaustive_oracle_for_every_four_vertex_directed_graph() {
        let directed_pairs = (0..4)
            .flat_map(|source| {
                (0..4)
                    .filter(move |target| *target != source)
                    .map(move |target| (source, target))
            })
            .collect::<Vec<_>>();

        for mask in 0..(1_u16 << directed_pairs.len()) {
            let mut adjacency = (0..4)
                .map(|vertex| (vertex, BTreeSet::new()))
                .collect::<Adjacency>();
            for (bit, (source, target)) in directed_pairs.iter().copied().enumerate() {
                if mask & (1_u16 << bit) != 0 {
                    if let Some(targets) = adjacency.get_mut(&source) {
                        targets.insert(target);
                    }
                }
            }

            let actual = johnson_cycles(&adjacency)
                .into_iter()
                .collect::<BTreeSet<_>>();
            assert_eq!(
                actual,
                exhaustive_cycles(&adjacency),
                "edge mask {mask:#05x}"
            );
        }
    }

    fn exhaustive_cycles(adjacency: &Adjacency) -> BTreeSet<Vec<usize>> {
        let mut cycles = BTreeSet::new();
        for length in 2..=adjacency.len() {
            append_paths(
                adjacency,
                length,
                &mut Vec::new(),
                &mut BTreeSet::new(),
                &mut cycles,
            );
        }
        cycles
    }

    fn append_paths(
        adjacency: &Adjacency,
        length: usize,
        path: &mut Vec<usize>,
        used: &mut BTreeSet<usize>,
        cycles: &mut BTreeSet<Vec<usize>>,
    ) {
        if path.len() == length {
            if path.first() == path.iter().min() && is_connected_cycle(path, adjacency) {
                cycles.insert(path.clone());
            }
            return;
        }

        for vertex in adjacency.keys().copied().collect::<Vec<_>>() {
            if used.insert(vertex) {
                path.push(vertex);
                append_paths(adjacency, length, path, used, cycles);
                path.pop();
                used.remove(&vertex);
            }
        }
    }

    fn is_connected_cycle(path: &[usize], adjacency: &Adjacency) -> bool {
        path.iter().enumerate().all(|(index, source)| {
            let target = path[(index + 1) % path.len()];
            adjacency
                .get(source)
                .is_some_and(|targets| targets.contains(&target))
        })
    }
}
