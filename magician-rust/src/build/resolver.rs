use std::collections::{HashMap, HashSet, VecDeque};
use syn::Item;
use crate::{index::ProjectIndex, visit::IdentCollector};

pub struct ResolvedSet {
    /// Items in dependency order (deps before dependents)
    pub ordered: Vec<Item>,
}

pub fn resolve(fn_name: &String, entry_fn: &syn::ItemFn, index: &ProjectIndex) -> ResolvedSet {
    // BFS: queue of names to resolve
    let mut queue: VecDeque<String> = VecDeque::new();
    let mut visited: HashSet<String> = HashSet::new();
    // name -> item (preserves insertion = discovery order)
    let mut collected: HashMap<String, Item> = HashMap::new();

    // Seed from the entry function itself
    queue.push_back(fn_name.clone());
    visited.insert(fn_name.clone());
    collected.insert(fn_name.clone(), Item::Fn(entry_fn.clone()));

    while let Some(name) = queue.pop_front() {
        let item = match collected.get(&name).or_else(|| index.items.get(&name)) {
            Some(i) => i.clone(),
            None    => continue, // not in project (stdlib, external crate — skip)
        };

        // Collect deps referenced by this item
        let deps = deps_of_item(find_segs(&name), &item, index);

        // Pull in impl blocks for any type we're including
        pull_impls(&name, index, &mut collected, &mut visited, &mut queue);

        for dep in deps {
            if !visited.contains(&dep) {
                visited.insert(dep.clone());
                if let Some(dep_item) = index.items.get(&dep) {
                    collected.insert(dep.clone(), dep_item.clone());
                    // Also pull impl blocks for this newly discovered type
                    pull_impls(&dep, index, &mut collected, &mut visited, &mut queue);
                    queue.push_back(dep);
                }
            }
        }
    }

    // Topological sort so deps come before the things that use them
    let ordered = topo_sort(collected, index);
    ResolvedSet { ordered }
}

fn pull_impls(
    type_name: &str,
    index: &ProjectIndex,
    collected: &mut HashMap<String, Item>,
    visited: &mut HashSet<String>,
    queue: &mut VecDeque<String>,
) {
    if let Some(impl_blocks) = index.impls.get(type_name) {
        for (i, impl_item) in impl_blocks.iter().enumerate() {
            let key = format!("__impl__{}__{}", type_name, i);
            if !visited.contains(&key) {
                visited.insert(key.clone());
                collected.insert(key.clone(), impl_item.clone());
                // impl blocks themselves may reference other types
                let deps = deps_of_item(find_segs(&key), impl_item, index);
                for dep in deps {
                    if !visited.contains(&dep) {
                        queue.push_back(dep);
                    }
                }
            }
        }
    }
}

fn deps_of_item(segs: Vec<String>, item: &Item, idx: &ProjectIndex) -> HashSet<String> {
    use syn::visit::Visit;
    let mut c = IdentCollector::new(segs, idx);
    c.visit_item(item);
    c.found
}

/// Simple Kahn's algorithm over the collected items.
/// Items with no deps among the set come first.
fn topo_sort(collected: HashMap<String, Item>, idx: &ProjectIndex) -> Vec<Item> {
    // Build adjacency: name -> set of names it depends on (within collected)
    let keys: HashSet<String> = collected.keys().cloned().collect();
    let mut in_edges: HashMap<String, HashSet<String>> = HashMap::new();

    for (name, item) in &collected {
        let deps = deps_of_item(find_segs(name), item, idx)
            .into_iter()
            .filter(|d| keys.contains(d) && d != name)
            .collect();
        in_edges.insert(name.clone(), deps);
    }

    let mut result = Vec::new();
    let mut remaining = in_edges;

    while !remaining.is_empty() {
        // Find all nodes with no remaining deps
        let ready: Vec<String> = remaining
            .iter()
            .filter(|(_, deps)| deps.is_empty())
            .map(|(k, _)| k.clone())
            .collect();

        if ready.is_empty() {
            // Cycle — just dump the rest in arbitrary order
            result.extend(remaining.keys().filter_map(|k| collected.get(k)).cloned());
            break;
        }

        let mut ready = ready;
        ready.sort(); // deterministic output order
        for name in ready {
            result.push(collected[&name].clone());
            remaining.remove(&name);
            // Remove this name from other nodes' dep sets
            for deps in remaining.values_mut() {
                deps.remove(&name);
            }
        }
    }

    result
}

fn find_segs(name: &String) -> Vec<String> {
    let mut result = name.split("::").map(|a| a.to_string()).collect::<Vec<_>>();
    if !result.is_empty() { result.remove(result.len() - 1); }
    if result.len() >= 2 && result[0] == "crate" && result[1] == "crate" { result.remove(result.len() - 1); }
    return result;
}
