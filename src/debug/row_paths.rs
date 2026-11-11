//! Writes, for the rows read through their IDA address, the field each address is in (run by
//! `tools/debug_pages/typed_rows.py`: `RC2K_ROW_PATHS=<file> cargo test dump_row_paths`).

#[test]
fn dump_row_paths() {
    let Some(out) = std::env::var_os("RC2K_ROW_PATHS") else {
        return;
    };
    let mut lines = String::new();
    let groups = crate::debug::display_rows::GROUPS.iter();
    for group in groups.chain(crate::debug::watch_rows::GROUPS) {
        for row in group.rows.iter().filter(|r| !r.is_typed()) {
            let at = row.at();
            let entry = crate::moved::MOVED
                .iter()
                .position(|m| (m.ida..m.ida + m.len).contains(&at));
            let found = crate::moved::named(at)
                .ok()
                .and_then(|p| crate::moved::store_of(p.addr() as usize));
            let (Some(entry), Some((store, offset))) = (entry, found) else {
                lines += &format!("{at:#X}\t-\t-\t-\t-\n");
                continue;
            };
            let mut path = String::new();
            let leaf = store.field_path_at(offset, &mut path);
            lines += &format!(
                "{at:#X}\t{entry}\t{}\t{path}\t{}\n",
                store.type_name(),
                leaf.unwrap_or("-")
            );
        }
    }
    std::fs::write(out, lines).unwrap();
}

/// Every row read through its field has that field where the row's IDA address is.
#[test]
fn typed_rows_are_where_their_addresses_say() {
    let groups = crate::debug::display_rows::GROUPS.iter();
    for group in groups.chain(crate::debug::watch_rows::GROUPS) {
        for row in group.rows {
            let Some(cell) = row.cell_address() else {
                continue;
            };
            let named = crate::moved::named(row.at()).unwrap().addr();
            assert_eq!(cell, named, "{:#x}", row.at());
        }
    }
}
