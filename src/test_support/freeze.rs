//! Writes `data/first_values.golden` from the stores' bytes as the program starts (the values
//! the game starts with), run by hand (`tools/m7_tests/freeze.sh`):
//! `RC2K_FREEZE_FIRST_VALUES=<golden> RC2K_FIRST_VALUES_MANIFEST=<manifest>`.
//!
//! Every registry store gets its line (`first_values::summary`: the bytes in its fields and
//! the hash over their paths and values) and a line for each gap that starts non-zero; each
//! manifest line its named read, the bytes of its pieces one after another
//! (`<key> <store key>@<offset>+<length> ...`, hex). A line `<key> held` (a read whose bytes
//! no store holds as the game has them: a Rust item, a handle where the game has an address)
//! keeps what the golden file already holds under the key.

use super::first_values::{gap_key, hash, store_keys, summary};

/// A named read longer than this many bytes is frozen as size and hash only.
const LONGEST_HELD: usize = 0x4000;

fn hex(n: &str) -> usize {
    usize::from_str_radix(n.trim_start_matches("0x").replace('_', "").as_str(), 16)
        .unwrap_or_else(|_| panic!("a hex number: {n}"))
}

/// The golden line for `key`: its size, hash and, for a named read short enough, its bytes.
fn line_of(key: &str, bytes: &[u8], named: bool) -> String {
    let line = format!("{key} {} {:016x}", bytes.len(), hash(bytes));
    if !named || bytes.len() > LONGEST_HELD {
        return line;
    }
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!("{line} {hex}")
}

/// The bytes of a manifest line's pieces (`<store key>@<offset>+<length>`), one after another.
fn read_pieces(pieces: &[&str]) -> Vec<u8> {
    let stores = store_keys();
    let mut bytes = Vec::new();
    for piece in pieces {
        let (key, place) = piece
            .split_once('@')
            .unwrap_or_else(|| panic!("a piece is `store@offset+length`: {piece}"));
        let (offset, len) = place
            .split_once('+')
            .unwrap_or_else(|| panic!("a piece is `store@offset+length`: {piece}"));
        let (_, e) = stores
            .iter()
            .find(|(k, _)| k == key)
            .unwrap_or_else(|| panic!("no store {key} in the registry"));
        let mut part = vec![0; hex(len)];
        e.store
            .load(hex(offset), &mut part)
            .unwrap_or_else(|_| panic!("{piece}: past the store's end"));
        bytes.extend(part);
    }
    bytes
}

#[test]
fn freeze_first_values() {
    let (Ok(out), Ok(manifest)) = (
        std::env::var("RC2K_FREEZE_FIRST_VALUES"),
        std::env::var("RC2K_FIRST_VALUES_MANIFEST"),
    ) else {
        return;
    };
    // The lines frozen before, for the keys held.
    let before = std::fs::read_to_string(&out).unwrap_or_default();
    let kept = |key: &str| {
        before
            .lines()
            .find(|l| l.split(' ').next() == Some(key))
            .unwrap_or_else(|| panic!("{key} is held but the golden file has no line for it"))
            .to_string()
    };
    let manifest = std::fs::read_to_string(manifest).expect("the manifest");
    let mut lines = Vec::new();
    for line in manifest.lines() {
        // Only whole lines are comments: a repeated store's key holds a `#`.
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let w: Vec<&str> = line.split_whitespace().collect();
        match w[..] {
            [key, "held"] => {
                assert!(!key.starts_with("store:"), "{key}: a store is never held");
                lines.push(kept(key));
            }
            [key, ref pieces @ ..] if !pieces.is_empty() => {
                lines.push(line_of(key, &read_pieces(pieces), true));
            }
            _ => panic!("a manifest line is `key held` or `key piece ...`: {line}"),
        }
    }
    for (key, e) in store_keys() {
        let s = summary(e.store);
        lines.push(format!("{key} {} {:016x}", s.size, s.hash));
        for (path, bytes) in &s.gaps {
            lines.push(line_of(&gap_key(&key, path), bytes, true));
        }
    }
    lines.sort();
    let n = lines.len();
    lines.dedup();
    assert_eq!(n, lines.len(), "a key twice");
    let mut text = String::from(
        "# The game's first values (test_support::first_values). Made by\n\
         # tools/m7_tests/freeze.sh; a change of a first value or of a gap that starts non-zero updates it in the same commit.\n",
    );
    for l in lines {
        text.push_str(&l);
        text.push('\n');
    }
    std::fs::write(out, text).expect("the golden file");
}
