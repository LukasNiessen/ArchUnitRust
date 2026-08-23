//! Deterministic directed-cycle algorithms over integer vertex identifiers.

mod johnson_cycles;
mod tarjan_scc;

use std::collections::{BTreeMap, BTreeSet};

pub(super) use johnson_cycles::johnson_cycles;

pub(super) type Adjacency = BTreeMap<usize, BTreeSet<usize>>;
