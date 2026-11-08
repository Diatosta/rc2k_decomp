//! Call graph of the `rc2k_decomp` crate's Rust source, for planning the race owner's
//! stage 2 (passing `&Race` down from `drive::play_race` instead of each function calling
//! `crate::race::owner::race()`).
//!
//! Usage: `cargo run --release -- <repo src dir> <out dir>`
//!
//! Every `src/**/*.rs` is parsed with `syn`. Modules come from file paths (`a/b.rs` and
//! `a/b/mod.rs` are `crate::a::b`, `bin/x.rs` is `crate::bin::x`) and inline `mod x {}`.
//! `#[cfg(test)]` modules and `#[test]` fns are skipped (only counted). Each fn (free or
//! impl method, named `module::Type::method`) records its `race()` calls (a call of a path
//! that resolves to `crate::race::owner::race`, no arguments), the groups read through it
//! (`race().physics...`, or through a variable bound by `let x = race()`), and its outgoing
//! calls. Closures and nested fns belong to the enclosing fn.
//!
//! Resolution of a call path, best effort: `crate::`, `self::`, `super::`, `Self::`, names
//! brought in by `use` (groups, `as`, `self`, glob imports, one level of `pub use`
//! re-exports), same-module fns, child modules and types. Method calls `x.f()` are only
//! counted (never guessed). Calls inside macros are read when the macro body parses as
//! comma-separated expressions, else by a token scan for `race()`.
//!
//! Outputs in the out dir: `fns.tsv` (path, file:line, race_calls, groups, n_callers,
//! n_callees), `edges.tsv` (caller, callee), `stage2.txt` (BFS from
//! `crate::drive::play_race::play_race`, race() users outside its tree, shared helpers),
//! `stats.txt` (resolution rate).
//!
//! A path naming a known fn in a value position (`car_step,` in a table, `.map(f)`) adds an
//! edge marked `ref` (a local variable with a fn's name would add a false one).
//!
//! Blind spots: method calls, trait dispatch, calls made
//! by macros that expand to fns, `#[path]` modules, fns with the same name in two impls of
//! one type (the later gets a `#2` suffix), type aliases, `use` inside fn bodies are treated
//! as module-wide.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use syn::punctuated::Punctuated;
use syn::visit::{self, Visit};
use syn::{Attribute, Expr, ImplItemFn, ItemFn, ItemImpl, ItemMod, ItemUse, UseTree};

const RACE_FN: &str = "crate::race::owner::race";
const ROOT_FN: &str = "crate::drive::play_race::play_race";
const GROUP_NAMES: [&str; 4] = ["physics", "course", "frame", "replay"];

#[derive(Default)]
struct Module {
    uses: HashMap<String, Vec<String>>,
    globs: Vec<Vec<String>>,
    children: HashSet<String>,
    fns: HashSet<String>,
    types: HashSet<String>,
}

#[derive(Default)]
struct FnRec {
    path: String,
    module: String,
    impl_ty: Option<String>,
    file: String,
    line: usize,
    binds_race: bool,
    groups: BTreeSet<String>,
    calls: Vec<Vec<String>>,
    /// paths used as values (`f` passed or stored, not called)
    refs: Vec<Vec<String>>,
    method_calls: usize,
}

struct Collector {
    file: String,
    module: Vec<String>,
    modules: HashMap<String, Module>,
    fns: Vec<FnRec>,
    cur: Option<FnRec>,
    impl_ty: Option<String>,
    skip: usize,
    test_fns: usize,
    bound: HashSet<String>,
    macro_scans: usize,
    macro_failed: usize,
}

fn is_test_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path().is_ident("test")
            || (a.path().is_ident("cfg")
                && a.meta.require_list().is_ok_and(|l| {
                    l.tokens
                        .to_string()
                        .split(|c: char| !c.is_alphanumeric() && c != '_')
                        .any(|w| w == "test")
                }))
    })
}

fn path_segs(p: &syn::Path) -> Vec<String> {
    p.segments.iter().map(|s| s.ident.to_string()).collect()
}

fn flatten_use(
    tree: &UseTree,
    prefix: &mut Vec<String>,
    out: &mut Vec<(String, Vec<String>)>,
    globs: &mut Vec<Vec<String>>,
) {
    match tree {
        UseTree::Path(p) => {
            prefix.push(p.ident.to_string());
            flatten_use(&p.tree, prefix, out, globs);
            prefix.pop();
        }
        UseTree::Name(n) => {
            let name = n.ident.to_string();
            if name == "self" {
                if let Some(last) = prefix.last() {
                    out.push((last.clone(), prefix.clone()));
                }
            } else {
                let mut t = prefix.clone();
                t.push(name.clone());
                out.push((name, t));
            }
        }
        UseTree::Rename(r) => {
            let mut t = prefix.clone();
            if r.ident != "self" {
                t.push(r.ident.to_string());
            }
            out.push((r.rename.to_string(), t));
        }
        UseTree::Glob(_) => globs.push(prefix.clone()),
        UseTree::Group(g) => {
            for t in &g.items {
                flatten_use(t, prefix, out, globs);
            }
        }
    }
}

impl Collector {
    fn mod_name(&self) -> String {
        self.module.join("::")
    }
    fn module(&mut self) -> &mut Module {
        let n = self.mod_name();
        self.modules.entry(n).or_default()
    }
    fn begin_fn(&mut self, name: &str, span: proc_macro2::Span) {
        let module = self.mod_name();
        let mut path = match &self.impl_ty {
            Some(t) => format!("{module}::{t}::{name}"),
            None => format!("{module}::{name}"),
        };
        if self.fns.iter().any(|f| f.path == path) {
            path.push_str("#2");
        }
        let impl_ty = self.impl_ty.clone();
        if impl_ty.is_none() {
            self.module().fns.insert(name.to_string());
        }
        self.bound.clear();
        self.cur = Some(FnRec {
            path,
            module,
            impl_ty,
            file: self.file.clone(),
            line: span.start().line,
            ..Default::default()
        });
    }
    fn end_fn(&mut self) {
        if let Some(f) = self.cur.take() {
            self.fns.push(f);
        }
    }
    /// A call whose callee is `path(...)`; remembers it, race() calls are resolved later.
    fn note_call(&mut self, segs: Vec<String>) {
        if let Some(f) = &mut self.cur {
            f.calls.push(segs.clone());
        }
    }
    fn visit_macro_body(&mut self, mac: &syn::Macro) {
        if self.cur.is_none() {
            return;
        }
        match mac.parse_body_with(Punctuated::<Expr, syn::Token![,]>::parse_terminated) {
            Ok(args) => {
                for e in &args {
                    self.visit_expr(e);
                }
            }
            Err(_) => {
                self.macro_failed += 1;
                let toks: Vec<proc_macro2::TokenTree> = mac.tokens.clone().into_iter().collect();
                for w in toks.windows(2) {
                    if let (proc_macro2::TokenTree::Ident(i), proc_macro2::TokenTree::Group(g)) =
                        (&w[0], &w[1])
                        && i == "race"
                        && g.stream().is_empty()
                    {
                        self.macro_scans += 1;
                        self.note_call(vec!["race".into()]);
                    }
                }
            }
        }
    }
}

impl<'a> Visit<'a> for Collector {
    fn visit_item_mod(&mut self, m: &'a ItemMod) {
        let name = m.ident.to_string();
        self.module().children.insert(name.clone());
        if is_test_attr(&m.attrs) {
            self.skip += 1;
            if let Some((_, items)) = &m.content {
                for i in items {
                    self.visit_item(i);
                }
            }
            self.skip -= 1;
            return;
        }
        if let Some((_, items)) = &m.content {
            self.module.push(name);
            let saved = self.impl_ty.take();
            for i in items {
                self.visit_item(i);
            }
            self.impl_ty = saved;
            self.module.pop();
        }
    }
    fn visit_item_use(&mut self, u: &'a ItemUse) {
        if self.skip > 0 {
            return;
        }
        let (mut out, mut globs) = (Vec::new(), Vec::new());
        flatten_use(&u.tree, &mut Vec::new(), &mut out, &mut globs);
        let m = self.module();
        for (k, v) in out {
            m.uses.insert(k, v);
        }
        m.globs.extend(globs);
    }
    fn visit_item_struct(&mut self, s: &'a syn::ItemStruct) {
        let n = s.ident.to_string();
        self.module().types.insert(n);
    }
    fn visit_item_enum(&mut self, s: &'a syn::ItemEnum) {
        let n = s.ident.to_string();
        self.module().types.insert(n);
    }
    fn visit_item_impl(&mut self, i: &'a ItemImpl) {
        let ty = match &*i.self_ty {
            syn::Type::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()),
            _ => None,
        };
        let saved = std::mem::replace(&mut self.impl_ty, ty.or(Some("_".into())));
        visit::visit_item_impl(self, i);
        self.impl_ty = saved;
    }
    fn visit_item_fn(&mut self, f: &'a ItemFn) {
        if self.cur.is_some() {
            return visit::visit_item_fn(self, f); // nested fn: part of the enclosing one
        }
        if self.skip > 0 || is_test_attr(&f.attrs) {
            self.test_fns += 1;
            return;
        }
        let saved = self.impl_ty.take();
        self.begin_fn(&f.sig.ident.to_string(), f.sig.ident.span());
        visit::visit_item_fn(self, f);
        self.end_fn();
        self.impl_ty = saved;
    }
    fn visit_impl_item_fn(&mut self, f: &'a ImplItemFn) {
        if self.skip > 0 || is_test_attr(&f.attrs) {
            self.test_fns += 1;
            return;
        }
        self.begin_fn(&f.sig.ident.to_string(), f.sig.ident.span());
        visit::visit_impl_item_fn(self, f);
        self.end_fn();
    }
    fn visit_expr_call(&mut self, c: &'a syn::ExprCall) {
        if let Expr::Path(p) = &*c.func {
            self.note_call(path_segs(&p.path));
        } else {
            self.visit_expr(&c.func);
        }
        for a in &c.args {
            self.visit_expr(a);
        }
    }
    fn visit_expr_path(&mut self, p: &'a syn::ExprPath) {
        if let Some(f) = &mut self.cur
            && p.qself.is_none()
        {
            f.refs.push(path_segs(&p.path));
        }
    }
    fn visit_expr_method_call(&mut self, c: &'a syn::ExprMethodCall) {
        if let Some(f) = &mut self.cur {
            f.method_calls += 1;
        }
        visit::visit_expr_method_call(self, c);
    }
    fn visit_local(&mut self, l: &'a syn::Local) {
        if let Some(init) = &l.init {
            let mut e = &*init.expr;
            while let Expr::Reference(r) = e {
                e = &r.expr;
            }
            let is_race = |e: &Expr| {
                matches!(e, Expr::Call(c) if c.args.is_empty()
                && matches!(&*c.func, Expr::Path(p) if p.path.segments.last().is_some_and(|s| s.ident == "race")))
            };
            if is_race(e)
                && let (Some(f), syn::Pat::Ident(pi)) = (&mut self.cur, &l.pat)
            {
                f.binds_race = true;
                self.bound.insert(pi.ident.to_string());
            }
        }
        visit::visit_local(self, l);
    }
    fn visit_expr_field(&mut self, e: &'a syn::ExprField) {
        // `race().group.x` or `bound.group.x`: the group is the first field after the owner
        if let syn::Member::Named(id) = &e.member {
            let g = id.to_string();
            if GROUP_NAMES.contains(&g.as_str()) {
                let base_is_owner = match &*e.base {
                    Expr::Call(c) => {
                        c.args.is_empty()
                            && matches!(&*c.func, Expr::Path(p) if p.path.segments.last().is_some_and(|s| s.ident == "race"))
                    }
                    Expr::Path(p) => p
                        .path
                        .get_ident()
                        .is_some_and(|i| self.bound.contains(&i.to_string())),
                    _ => false,
                };
                if base_is_owner && let Some(f) = &mut self.cur {
                    f.groups.insert(g);
                }
            }
        }
        visit::visit_expr_field(self, e);
    }
    fn visit_expr_macro(&mut self, m: &'a syn::ExprMacro) {
        self.visit_macro_body(&m.mac);
    }
    fn visit_stmt_macro(&mut self, m: &'a syn::StmtMacro) {
        self.visit_macro_body(&m.mac);
    }
}

fn module_of_file(rel: &Path) -> Vec<String> {
    let mut v = vec!["crate".to_string()];
    let comps: Vec<String> = rel
        .with_extension("")
        .iter()
        .map(|c| c.to_string_lossy().into_owned())
        .collect();
    for (i, c) in comps.iter().enumerate() {
        let last = i + 1 == comps.len();
        if (last && (c == "mod" || c == "lib" || c == "main")) && !(c == "main" && comps.len() > 1)
        {
            continue;
        }
        v.push(c.clone());
    }
    v
}

fn find_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    let mut es: Vec<_> = rd.flatten().map(|e| e.path()).collect();
    es.sort();
    for p in es {
        if p.is_dir() {
            find_rs(&p, out);
        } else if p.extension().is_some_and(|e| e == "rs") {
            out.push(p);
        }
    }
}

struct Resolver<'a> {
    modules: &'a HashMap<String, Module>,
    index: &'a HashMap<String, usize>,
}

enum Res {
    Fn(usize),
    /// looked internal (a crate path) but names no fn we know: constructor, trait fn, macro fn
    InternalMiss,
    External,
}

impl Resolver<'_> {
    /// Absolute segments of `segs` as seen from `module`; None for names outside the crate.
    fn abs(
        &self,
        module: &str,
        impl_ty: Option<&str>,
        segs: &[String],
        depth: usize,
    ) -> Option<Vec<String>> {
        if depth > 4 {
            return None;
        }
        let first = segs.first()?.as_str();
        let mp: Vec<String> = module.split("::").map(String::from).collect();
        let rest = &segs[1..];
        let join = |mut base: Vec<String>, rest: &[String]| {
            base.extend(rest.iter().cloned());
            Some(base)
        };
        match first {
            "crate" => Some(segs.to_vec()),
            "self" => join(mp, rest),
            "super" => {
                let mut m = mp;
                let mut r = segs;
                while r.first().map(String::as_str) == Some("super") {
                    m.pop();
                    r = &r[1..];
                }
                join(m, r)
            }
            "Self" => join([mp, vec![impl_ty?.to_string()]].concat(), rest),
            _ => {
                let m = self.modules.get(module);
                if let Some(t) = m.and_then(|m| m.uses.get(first)) {
                    if depth > 4 {
                        return None;
                    }
                    let base = self.abs(module, impl_ty, t, depth + 1)?;
                    return join(base, rest);
                }
                if let Some(m) = m {
                    if m.children.contains(first)
                        || m.fns.contains(first)
                        || m.types.contains(first)
                    {
                        return join(mp, segs);
                    }
                    for g in &m.globs {
                        let Some(gabs) = self.abs(module, impl_ty, g, depth + 1) else {
                            continue;
                        };
                        if let Some(gm) = self.modules.get(&gabs.join("::"))
                            && (gm.children.contains(first)
                                || gm.fns.contains(first)
                                || gm.types.contains(first))
                        {
                            return join(gabs, segs);
                        }
                    }
                }
                None
            }
        }
    }

    fn lookup(&self, full: &[String], depth: usize) -> Option<usize> {
        let key = full.join("::");
        if let Some(&i) = self.index.get(&key) {
            return Some(i);
        }
        // a `pub use` re-export in the parent module
        if depth < 4 && full.len() > 1 {
            let pm = full[..full.len() - 1].join("::");
            if let Some(t) = self
                .modules
                .get(&pm)
                .and_then(|m| m.uses.get(&full[full.len() - 1]))
            {
                let a = self.abs(&pm, None, t, 0)?;
                return self.lookup(&a, depth + 1);
            }
        }
        None
    }

    fn resolve(&self, f: &FnRec, segs: &[String]) -> Res {
        match self.abs(&f.module, f.impl_ty.as_deref(), segs, 0) {
            None => Res::External,
            Some(a) => match self.lookup(&a, 0) {
                Some(i) => Res::Fn(i),
                None => Res::InternalMiss,
            },
        }
    }
}

fn main() {
    // generated code nests deeply: parse and visit on a big stack
    std::thread::Builder::new()
        .stack_size(1 << 30)
        .spawn(run)
        .unwrap()
        .join()
        .unwrap();
}

fn run() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: callgraph <repo src dir> <out dir>");
        std::process::exit(2);
    }
    let src = PathBuf::from(&args[1]);
    let out = PathBuf::from(&args[2]);
    fs::create_dir_all(&out).unwrap();
    let mut files = Vec::new();
    find_rs(&src, &mut files);

    let mut col = Collector {
        file: String::new(),
        module: vec![],
        modules: HashMap::new(),
        fns: vec![],
        cur: None,
        impl_ty: None,
        skip: 0,
        test_fns: 0,
        bound: HashSet::new(),
        macro_scans: 0,
        macro_failed: 0,
    };
    let mut parse_errors = 0;
    for p in &files {
        let rel = p.strip_prefix(&src).unwrap();
        let text = fs::read_to_string(p).unwrap();
        let Ok(ast) = syn::parse_file(&text) else {
            parse_errors += 1;
            eprintln!("parse error: {}", p.display());
            continue;
        };
        col.file = format!("src/{}", rel.display());
        col.module = module_of_file(rel);
        // a `mod x;` declaration in this file makes x a child of this module
        col.module();
        col.visit_file(&ast);
    }

    // children implied by files (parent module -> child file module), for `x::f()` paths
    let names: Vec<String> = col.modules.keys().cloned().collect();
    for n in names {
        if let Some((parent, child)) = n.rsplit_once("::") {
            col.modules
                .entry(parent.to_string())
                .or_default()
                .children
                .insert(child.to_string());
        }
    }

    let index: HashMap<String, usize> = col
        .fns
        .iter()
        .enumerate()
        .map(|(i, f)| (f.path.clone(), i))
        .collect();
    let res = Resolver {
        modules: &col.modules,
        index: &index,
    };
    let race_idx = index.get(RACE_FN).copied();

    let n = col.fns.len();
    let mut race_calls = vec![0usize; n];
    let mut edges: BTreeSet<(usize, usize)> = BTreeSet::new();
    let (mut n_resolved, mut n_miss, mut n_ext, mut n_method) = (0usize, 0usize, 0usize, 0usize);
    for (i, f) in col.fns.iter().enumerate() {
        n_method += f.method_calls;
        for c in &f.calls {
            match res.resolve(f, c) {
                Res::Fn(j) => {
                    n_resolved += 1;
                    if Some(j) == race_idx {
                        race_calls[i] += 1;
                    } else {
                        edges.insert((i, j));
                    }
                }
                Res::InternalMiss => n_miss += 1,
                Res::External => n_ext += 1,
            }
        }
    }
    let mut n_ref_edges = 0usize;
    let mut ref_only: BTreeSet<(usize, usize)> = BTreeSet::new();
    for (i, f) in col.fns.iter().enumerate() {
        for r in &f.refs {
            if let Res::Fn(j) = res.resolve(f, r)
                && Some(j) != race_idx
                && j != i
                && edges.insert((i, j))
            {
                n_ref_edges += 1;
                ref_only.insert((i, j));
            }
        }
    }
    let mut callees = vec![Vec::new(); n];
    let mut callers = vec![Vec::new(); n];
    for &(a, b) in &edges {
        callees[a].push(b);
        callers[b].push(a);
    }

    let mut s = String::new();
    for (i, f) in col.fns.iter().enumerate() {
        let g = f.groups.iter().cloned().collect::<Vec<_>>().join(",");
        let _ = writeln!(
            s,
            "{}\t{}:{}\t{}\t{}\t{}\t{}\t{}",
            f.path,
            f.file,
            f.line,
            race_calls[i],
            g,
            callers[i].len(),
            callees[i].len(),
            f.binds_race
        );
    }
    fs::write(out.join("fns.tsv"), s).unwrap();
    let mut s = String::new();
    for &(a, b) in &edges {
        let kind = if ref_only.contains(&(a, b)) {
            "ref"
        } else {
            "call"
        };
        let _ = writeln!(s, "{}\t{}\t{kind}", col.fns[a].path, col.fns[b].path);
    }
    fs::write(out.join("edges.tsv"), s).unwrap();

    // stage 2: the tree under play_race
    let mut st = String::new();
    let root = index.get(ROOT_FN).copied();
    let users: Vec<usize> = (0..n).filter(|&i| race_calls[i] > 0).collect();
    // fns that reach (or are) a race() user
    let mut reaches = vec![false; n];
    let mut q: VecDeque<usize> = users.iter().copied().collect();
    for &u in &users {
        reaches[u] = true;
    }
    while let Some(x) = q.pop_front() {
        for &c in &callers[x] {
            if !reaches[c] {
                reaches[c] = true;
                q.push_back(c);
            }
        }
    }
    let mut depth = vec![usize::MAX; n];
    if let Some(r) = root {
        depth[r] = 0;
        let mut q = VecDeque::from([r]);
        while let Some(x) = q.pop_front() {
            for &c in &callees[x] {
                if depth[c] == usize::MAX {
                    depth[c] = depth[x] + 1;
                    q.push_back(c);
                }
            }
        }
    } else {
        let _ = writeln!(st, "root {ROOT_FN} not found");
    }
    let in_tree = |i: usize| depth[i] != usize::MAX;
    let _ = writeln!(
        st,
        "fns: {n}, race() users: {}, race() calls: {}",
        users.len(),
        race_calls.iter().sum::<usize>()
    );
    let _ = writeln!(
        st,
        "\nBFS from {ROOT_FN}\ndepth\tfns\tuse race()\treach a user (incl. self)"
    );
    let maxd = depth
        .iter()
        .filter(|&&d| d != usize::MAX)
        .max()
        .copied()
        .unwrap_or(0);
    let (mut tf, mut tu, mut tr) = (0, 0, 0);
    for d in 0..=maxd {
        let at: Vec<usize> = (0..n).filter(|&i| depth[i] == d).collect();
        let u = at.iter().filter(|&&i| race_calls[i] > 0).count();
        let r = at.iter().filter(|&&i| reaches[i]).count();
        let _ = writeln!(st, "{d}\t{}\t{u}\t{r}", at.len());
        tf += at.len();
        tu += u;
        tr += r;
    }
    let _ = writeln!(st, "total\t{tf}\t{tu}\t{tr}");

    let mut outside: Vec<usize> = users.iter().copied().filter(|&i| !in_tree(i)).collect();
    outside.sort_by_key(|&i| (std::cmp::Reverse(race_calls[i]), col.fns[i].path.clone()));
    let _ = writeln!(
        st,
        "\nrace() users NOT reachable from play_race: {} (calls: {})",
        outside.len(),
        outside.iter().map(|&i| race_calls[i]).sum::<usize>()
    );
    let _ = writeln!(
        st,
        "path\tfile:line\trace_calls\tgroups\tcallers\troots (callers with no callers)"
    );
    for &i in &outside {
        let roots = roots_of(i, &callers, &col.fns);
        let shown: Vec<&str> = roots
            .iter()
            .take(4)
            .map(|&r| col.fns[r].path.as_str())
            .collect();
        let more = if roots.len() > 4 {
            format!(" (+{})", roots.len() - 4)
        } else {
            String::new()
        };
        let f = &col.fns[i];
        let _ = writeln!(
            st,
            "{}\t{}:{}\t{}\t{}\t{}\t{}{more}",
            f.path,
            f.file,
            f.line,
            race_calls[i],
            f.groups.iter().cloned().collect::<Vec<_>>().join(","),
            callers[i].len(),
            if roots.is_empty() {
                "(itself a root)".to_string()
            } else {
                shown.join(" | ")
            }
        );
    }

    let mut shared: Vec<(usize, usize)> = users
        .iter()
        .copied()
        .filter(|&i| in_tree(i))
        .map(|i| (i, callers[i].iter().filter(|&&c| !in_tree(c)).count()))
        .filter(|&(_, o)| o > 0)
        .collect();
    shared.sort_by_key(|&(i, o)| (std::cmp::Reverse(o), col.fns[i].path.clone()));
    let _ = writeln!(
        st,
        "\nshared helpers (use race(), in the play_race tree, also called from outside it): {}",
        shared.len()
    );
    let _ = writeln!(
        st,
        "path\tfile:line\trace_calls\tcallers inside\tcallers outside"
    );
    for &(i, o) in &shared {
        let f = &col.fns[i];
        let _ = writeln!(
            st,
            "{}\t{}:{}\t{}\t{}\t{o}",
            f.path,
            f.file,
            f.line,
            race_calls[i],
            callers[i].len() - o
        );
    }
    fs::write(out.join("stage2.txt"), st).unwrap();

    let known_methods: HashSet<&str> = col
        .fns
        .iter()
        .filter(|f| f.impl_ty.is_some())
        .map(|f| f.path.rsplit("::").next().unwrap())
        .collect();
    let mut s = String::new();
    let total_path = n_resolved + n_miss + n_ext;
    let _ = writeln!(
        s,
        "files {} (parse errors {parse_errors}), modules {}, fns {n}, test fns skipped {}",
        files.len(),
        col.modules.len(),
        col.test_fns
    );
    let _ = writeln!(
        s,
        "path calls {total_path}: resolved to a crate fn {n_resolved}, crate path with no known fn {n_miss}, outside the crate {n_ext}"
    );
    let _ = writeln!(
        s,
        "resolved / (resolved + crate path with no known fn) = {:.1}%",
        100.0 * n_resolved as f64 / (n_resolved + n_miss).max(1) as f64
    );
    let _ = writeln!(
        s,
        "method calls (not resolved): {n_method}; distinct impl method names known: {}",
        known_methods.len()
    );
    let _ = writeln!(
        s,
        "macro bodies not parsed as expressions: {} (race() found by token scan: {})",
        col.macro_failed, col.macro_scans
    );
    let _ = writeln!(
        s,
        "edges {} (of which {n_ref_edges} only by a fn used as a value, `ref` in edges.tsv), fns with no caller {}",
        edges.len(),
        callers.iter().filter(|c| c.is_empty()).count()
    );
    fs::write(out.join("stats.txt"), &s).unwrap();
    print!("{s}");
    let mut by_group: BTreeMap<String, usize> = BTreeMap::new();
    for f in &col.fns {
        for g in &f.groups {
            *by_group.entry(g.clone()).or_default() += 1;
        }
    }
    println!("fns reading each group: {by_group:?}");
}

/// Callers (transitively) that have no callers themselves.
fn roots_of(i: usize, callers: &[Vec<usize>], fns: &[FnRec]) -> Vec<usize> {
    let mut seen = HashSet::from([i]);
    let mut q = VecDeque::from([i]);
    let mut roots = Vec::new();
    while let Some(x) = q.pop_front() {
        for &c in &callers[x] {
            if seen.insert(c) {
                if callers[c].is_empty() {
                    roots.push(c);
                }
                q.push_back(c);
            }
        }
    }
    roots.sort_by_key(|&r| fns[r].path.clone());
    roots
}
