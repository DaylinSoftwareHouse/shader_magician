use std::collections::HashSet;
use syn::{visit::Visit, ItemFn, Path};

pub struct IdentCollector {
    pub found: HashSet<String>,
    /// The module path of the file being visited, e.g. ["a", "b"]
    /// so we can qualify single-segment paths as crate::a::b::foo
    module_segments: Vec<String>,
    builtins: HashSet<&'static str>,
}

impl IdentCollector {
    pub fn new(module_segments: Vec<String>) -> Self {
        let builtins = [
            "i8","i16","i32","i64","i128","isize",
            "u8","u16","u32","u64","u128","usize",
            "f32","f64","bool","char","str","String",
            "Vec","Option","Result","Box","Pin","Arc","Rc",
            "HashMap","HashSet","BTreeMap","BTreeSet",
            "Some","None","Ok","Err","true","false",
            "Self","self","super","crate",
            "println","eprintln","panic","assert",
            "Default","Clone","Copy","Debug","Display",
            "Iterator","Into","From","AsRef","AsMut",
            "Send","Sync","Sized","Unpin",
        ].into_iter().collect();
        Self { found: HashSet::new(), module_segments, builtins }
    }

    pub fn collect_from_fn(f: &ItemFn, module_segments: Vec<String>) -> HashSet<String> {
        let mut c = Self::new(module_segments);
        c.visit_item_fn(f);
        c.found
    }

    /// Normalise a syn Path into a `crate::a::b::Foo` string, or None if
    /// it's purely a builtin / keyword path we should skip.
    fn normalise_path(&self, p: &Path) -> Option<String> {
        let segments: Vec<String> = p.segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect();

        // Skip pure-keyword paths
        if segments.iter().all(|s| {
            matches!(s.as_str(), "self" | "super" | "crate" | "Self")
        }) {
            return None;
        }

        // Skip if the leaf is a builtin
        let last = segments.last()?;
        if self.builtins.contains(last.as_str()) {
            return None;
        }

        let qualified = match segments.first().map(String::as_str) {
            // Already absolute: `crate::a::b::Foo`
            Some("crate") => segments.join("::"),

            // External crate: `serde::Deserialize` — keep as-is, your
            // resolver can decide whether it cares about external paths
            Some(first) if is_external_crate(first, &segments) => {
                segments.join("::")
            }

            // Relative single-segment `Foo` — qualify with current module
            // so the resolver gets `crate::a::b::Foo`
            _ if segments.len() == 1 => {
                if self.module_segments.is_empty() {
                    format!("{}", segments[0])
                } else {
                    format!("{}::{}", self.module_segments.join("::"), segments[0])
                }
            }

            // Relative multi-segment `a::b::Foo` — prepend `crate::`
            _ => format!("{}", segments.join("::")),
        };

        Some(qualified)
    }
}

/// Heuristic: if the first segment is lowercase and not a Rust keyword it's
/// likely an external crate name rather than a local module reference.
/// Your resolver will ultimately confirm whether the path exists locally.
fn is_external_crate(first: &str, segments: &[String]) -> bool {
    // paths starting with an uppercase are types, not crate names
    let starts_lowercase = first.chars().next().map_or(false, |c| c.is_lowercase());
    // single-segment lowercase is just a local ident, not an external crate
    let is_multi = segments.len() > 1;
    starts_lowercase && is_multi
}

impl<'ast> Visit<'ast> for IdentCollector {
    fn visit_path(&mut self, p: &'ast Path) {
        if let Some(qualified) = self.normalise_path(p) {
            self.found.insert(qualified);
        }
        syn::visit::visit_path(self, p);
    }

    fn visit_macro(&mut self, m: &'ast syn::Macro) {
        if let Some(qualified) = self.normalise_path(&m.path) {
            self.found.insert(qualified);
        }
        syn::visit::visit_macro(self, m);
    }
}
