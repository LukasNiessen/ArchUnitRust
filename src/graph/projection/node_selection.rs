use std::collections::{BTreeSet, VecDeque};

use crate::{Edge, Filter};

use super::GraphQueryOptions;

pub(crate) fn select_graph_nodes(edges: &[Edge], options: &GraphQueryOptions) -> BTreeSet<String> {
    if options.focus().is_none()
        && options.reachable_from().is_none()
        && options.dependents_of().is_none()
    {
        return all_nodes(edges);
    }

    let mut selected = BTreeSet::new();
    if let Some(focus) = options.focus() {
        selected.extend(expand_focus(edges, focus, options.focus_depth()));
    }
    if let Some(reachable_from) = options.reachable_from() {
        selected.extend(walk(edges, reachable_from, Direction::Outgoing));
    }
    if let Some(dependents_of) = options.dependents_of() {
        selected.extend(walk(edges, dependents_of, Direction::Incoming));
    }
    selected
}

fn all_nodes(edges: &[Edge]) -> BTreeSet<String> {
    edges
        .iter()
        .flat_map(|edge| [&edge.source, &edge.target])
        .cloned()
        .collect()
}

fn matching_nodes(edges: &[Edge], filter: &Filter) -> BTreeSet<String> {
    all_nodes(edges)
        .into_iter()
        .filter(|node| filter.matches(node))
        .collect()
}

fn expand_focus(edges: &[Edge], filter: &Filter, depth: usize) -> BTreeSet<String> {
    let mut selected = matching_nodes(edges, filter);
    let mut queue = selected
        .iter()
        .cloned()
        .map(|node| (node, 0))
        .collect::<VecDeque<_>>();

    while let Some((current, current_depth)) = queue.pop_front() {
        if current_depth >= depth {
            continue;
        }

        for neighbor in neighbors_of(edges, &current) {
            if selected.insert(neighbor.clone()) {
                queue.push_back((neighbor, current_depth + 1));
            }
        }
    }
    selected
}

fn neighbors_of(edges: &[Edge], node: &str) -> BTreeSet<String> {
    edges
        .iter()
        .filter_map(|edge| {
            if edge.source == node && edge.target != node {
                Some(edge.target.clone())
            } else if edge.target == node && edge.source != node {
                Some(edge.source.clone())
            } else {
                None
            }
        })
        .collect()
}

#[derive(Clone, Copy)]
enum Direction {
    Outgoing,
    Incoming,
}

fn walk(edges: &[Edge], filter: &Filter, direction: Direction) -> BTreeSet<String> {
    let mut selected = matching_nodes(edges, filter);
    let mut queue = selected.iter().cloned().collect::<VecDeque<_>>();

    while let Some(current) = queue.pop_front() {
        for next in next_nodes(edges, &current, direction) {
            if selected.insert(next.clone()) {
                queue.push_back(next);
            }
        }
    }
    selected
}

fn next_nodes(edges: &[Edge], node: &str, direction: Direction) -> BTreeSet<String> {
    edges
        .iter()
        .filter_map(|edge| match direction {
            Direction::Outgoing if edge.source == node => Some(edge.target.clone()),
            Direction::Incoming if edge.target == node => Some(edge.source.clone()),
            Direction::Outgoing | Direction::Incoming => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{Edge, GraphQueryOptions, RegexFactory};

    use super::select_graph_nodes;

    fn edges() -> Vec<Edge> {
        vec![
            Edge::self_edge("src/api.rs"),
            Edge::self_edge("src/service.rs"),
            Edge::self_edge("src/database.rs"),
            Edge::self_edge("src/orphan.rs"),
            Edge::new("src/api.rs", "src/service.rs", false, []),
            Edge::new("src/service.rs", "src/database.rs", false, []),
        ]
    }

    fn filter(path: &str) -> crate::Filter {
        RegexFactory::default()
            .path_matcher(path)
            .expect("fixture selector should compile")
    }

    #[test]
    fn no_query_selects_every_endpoint_including_isolated_self_nodes() {
        let selected = select_graph_nodes(&edges(), &GraphQueryOptions::new());

        assert_eq!(
            selected.into_iter().collect::<Vec<_>>(),
            [
                "src/api.rs",
                "src/database.rs",
                "src/orphan.rs",
                "src/service.rs"
            ]
        );
    }

    #[test]
    fn focus_expands_undirected_neighbors_to_the_exact_depth() {
        let exact = GraphQueryOptions::new().with_focus(filter("src/service.rs"), 0);
        let neighbors = GraphQueryOptions::new().with_focus(filter("src/service.rs"), 1);

        assert_eq!(
            select_graph_nodes(&edges(), &exact)
                .into_iter()
                .collect::<Vec<_>>(),
            ["src/service.rs"]
        );
        assert_eq!(
            select_graph_nodes(&edges(), &neighbors)
                .into_iter()
                .collect::<Vec<_>>(),
            ["src/api.rs", "src/database.rs", "src/service.rs"]
        );
    }

    #[test]
    fn directed_walks_select_dependencies_or_reverse_dependents_transitively() {
        let reachable = GraphQueryOptions::new().with_reachable_from(filter("src/service.rs"));
        let dependents = GraphQueryOptions::new().with_dependents_of(filter("src/database.rs"));

        assert_eq!(
            select_graph_nodes(&edges(), &reachable)
                .into_iter()
                .collect::<Vec<_>>(),
            ["src/database.rs", "src/service.rs"]
        );
        assert_eq!(
            select_graph_nodes(&edges(), &dependents)
                .into_iter()
                .collect::<Vec<_>>(),
            ["src/api.rs", "src/database.rs", "src/service.rs"]
        );
    }

    #[test]
    fn independent_query_modifiers_combine_as_a_union() {
        let options = GraphQueryOptions::new()
            .with_focus(filter("src/orphan.rs"), 0)
            .with_reachable_from(filter("src/service.rs"));

        assert_eq!(
            select_graph_nodes(&edges(), &options)
                .into_iter()
                .collect::<Vec<_>>(),
            ["src/database.rs", "src/orphan.rs", "src/service.rs"]
        );
    }
}
