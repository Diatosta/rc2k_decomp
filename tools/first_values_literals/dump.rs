//! Temporary test (copy to src/test_support/literals.rs, add `mod literals;` to test_support/mod.rs):
//! writes, for each store named in RC2K_DUMP_STORES (comma list of golden keys, `store:...`),
//! the primitive cells that are not all zero: `key<TAB>path<TAB>type<TAB>hex` per line.
use super::first_values::store_keys;

fn size_of_type(t: &str) -> Option<usize> {
    let short = t.rsplit("::").next().unwrap_or(t);
    let short = short.trim_start_matches("Plain");
    match short {
        "Dword" | "Float" | "Flag" | "Loose" => Some(4),
        "Word" => Some(2),
        "Byte" => Some(1),
        _ => None,
    }
}

#[test]
fn dump_first_value_cells() {
    let (Ok(keys), Ok(out)) = (
        std::env::var("RC2K_DUMP_STORES"),
        std::env::var("RC2K_DUMP_OUT"),
    ) else {
        return;
    };
    let mut text = String::new();
    for want in keys.split(',') {
        let (key, e) = store_keys()
            .into_iter()
            .find(|(k, _)| k == want)
            .unwrap_or_else(|| panic!("no store {want}"));
        let store = e.store;
        let size = store.size();
        let mut at = 0;
        while at < size {
            let mut path = String::new();
            let ty = store.field_path_at(at, &mut path).expect("typed store");
            let (start, run) = store.cell_run_at(at).expect("a run");
            let len = match size_of_type(ty) {
                Some(n) => n,
                None => start + run.len() - at,
            };
            let mut bytes = vec![0; len];
            store.load(at, &mut bytes).expect("load");
            if bytes.iter().any(|&b| b != 0) {
                let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
                text.push_str(&format!("{key}\t{path}\t{ty}\t{at:x}\t{hex}\n"));
            }
            at += len;
        }
    }
    std::fs::write(out, text).unwrap();
}
