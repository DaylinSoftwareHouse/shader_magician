use std::collections::HashSet;
use syn::{visit::Visit, ItemFn, Path};

pub struct IdentCollector {
    pub found: HashSet<String>,
    // These are never user-defined items; skip them
    builtins: HashSet<&'static str>,
}

impl IdentCollector {
    pub fn new() -> Self {
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
        Self { found: HashSet::new(), builtins }
    }

    pub fn collect_from_fn(f: &ItemFn) -> HashSet<String> {
        let mut c = Self::new();
        c.visit_item_fn(f);
        c.found
    }
}

impl<'ast> Visit<'ast> for IdentCollector {
    // Catch type paths: MyStruct, some_fn(), MyEnum::Variant
    fn visit_path(&mut self, p: &'ast Path) {
        // Only take the last segment to get the bare name;
        // the resolver looks it up in the index regardless of module path.
        if let Some(last) = p.segments.last() {
            let name = last.ident.to_string();
            if !self.builtins.contains(name.as_str()) {
                self.found.insert(name);
            }
        }
        syn::visit::visit_path(self, p);
    }

    // Catch macro invocations: my_macro!(...)
    fn visit_macro(&mut self, m: &'ast syn::Macro) {
        if let Some(last) = m.path.segments.last() {
            self.found.insert(last.ident.to_string());
        }
        syn::visit::visit_macro(self, m);
    }
}
