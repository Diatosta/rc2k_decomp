//! The game's first values, frozen in `data/first_values.golden`: for every store of the
//! `stores` registry a hash over its fields' names and values, and for the constants and
//! fields the tests read, the bytes themselves under a name.
//!
//! Each line is `<key> <size> <hash>` with, for a named read, the bytes in hex after it.
//! A store's key is `store:` and its registry name (`#2`, `#3` ... after the name when the
//! registry holds it more than once, in the registry's order). The hash is FNV-1a, 64 bits.
//!
//! A store's line does not depend on the layout. The store is cut into its runs of like
//! cells (`Cells::cell_run`), each named by its field path (`Cells::field_path`: `.a.b[2]`);
//! the line's hash is the sum of one hash per run over its path, length and bytes, so the
//! order of the fields and the bytes between them do not count, and its size is the bytes
//! the runs hold. Gap fields (`_gap_*`, `_lead*`, `_pad*`, `_tail*`: bytes no code names) are
//! left out, but for a gap that starts non-zero: it has a line of its own,
//! `gap:<store key>:<field path>` with its bytes, so dropping one changes the file in plain
//! sight. A rename of a gap that starts at zero changes nothing.

use std::collections::HashMap;

static GOLDEN: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/data/first_values.golden"
));

/// One line of the golden file.
pub struct Golden {
    pub size: usize,
    pub hash: u64,
    pub bytes: Option<Vec<u8>>,
}

/// FNV-1a over `bytes`, 64 bits: the golden file's hash.
pub fn hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xCBF2_9CE4_8422_2325, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01B3)
    })
}

fn parse_hex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|n| u8::from_str_radix(&text[n..n + 2], 16).expect("hex bytes"))
        .collect()
}

/// The golden file's lines by key.
pub fn table() -> &'static HashMap<&'static str, Golden> {
    static TABLE: std::sync::OnceLock<HashMap<&'static str, Golden>> = std::sync::OnceLock::new();
    TABLE.get_or_init(|| {
        GOLDEN
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| {
                let mut words = line.split(' ');
                let mut next = || words.next().expect("a golden line's words");
                let key = next();
                let size = next().parse().expect("a golden line's size");
                let hash = u64::from_str_radix(next(), 16).expect("a golden line's hash");
                let bytes = words.next().map(parse_hex);
                if let Some(b) = &bytes {
                    assert_eq!(
                        (b.len(), super::first_values::hash(b)),
                        (size, hash),
                        "{key}"
                    );
                }
                (key, Golden { size, hash, bytes })
            })
            .collect()
    })
}

/// The line under `key`.
pub fn golden(key: &str) -> &'static Golden {
    table()
        .get(key)
        .unwrap_or_else(|| panic!("no first value named {key} in data/first_values.golden"))
}

/// The bytes frozen under `key`.
pub fn bytes(key: &str) -> &'static [u8] {
    golden(key)
        .bytes
        .as_deref()
        .unwrap_or_else(|| panic!("the first value {key} is held as a hash only"))
}

/// `mine` equals the bytes under `key` (compared by size and hash where only those are held).
pub fn holds(key: &str, mine: &[u8]) -> bool {
    let g = golden(key);
    match &g.bytes {
        Some(b) => mine == &b[..],
        None => mine.len() == g.size && hash(mine) == g.hash,
    }
}

/// Dword `n` (from 0) of the bytes frozen under `key`.
pub fn dword(key: &str, n: usize) -> u32 {
    let b = bytes(key);
    u32::from_le_bytes(b[4 * n..4 * n + 4].try_into().expect("four bytes"))
}

/// `mine` is what the golden file holds under `key`: its size and hash, and its bytes
/// where they are held (the first byte that differs is named).
pub fn assert_matches(key: &str, mine: &[u8]) {
    let g = golden(key);
    if let Some(b) = &g.bytes {
        let first = mine.iter().zip(b).position(|(a, b)| a != b);
        assert!(
            mine == &b[..],
            "{key}: {} bytes where the game has {}; first difference at {first:x?}",
            mine.len(),
            b.len()
        );
    } else {
        assert_eq!(mine.len(), g.size, "{key}: the size");
        assert!(
            hash(mine) == g.hash,
            "{key}: the bytes differ from the game's"
        );
    }
}

/// The registry's stores with their golden keys (`store:<name>`, numbered after the first
/// when a name repeats), in the registry's order.
pub fn store_keys() -> Vec<(String, &'static crate::stores::Entry)> {
    let mut seen: HashMap<&str, usize> = HashMap::new();
    crate::stores::listed()
        .iter()
        .map(|e| {
            let n = seen.entry(e.name).or_default();
            *n += 1;
            let key = match *n {
                1 => format!("store:{}", e.name),
                n => format!("store:{}#{n}", e.name),
            };
            (key, e)
        })
        .collect()
}

/// The prefixes of the names of gap fields: bytes the game has and no code names.
const GAP_PREFIXES: [&str; 4] = ["_gap_", "_lead", "_pad", "_tail"];

/// The field path holds a gap field (`.a._gap_935630`, `.a[3]._tail`).
fn is_gap(path: &str) -> bool {
    path.split('.').skip(1).any(|part| {
        let name = part.split('[').next().unwrap_or(part);
        GAP_PREFIXES.iter().any(|p| name.starts_with(p))
    })
}

/// One run of like cells of a store: where it starts, how long it is, and the field path of
/// its first byte (`@<offset>` where the store names no fields).
pub struct Run {
    pub offset: usize,
    pub len: usize,
    pub path: String,
}

/// The runs a store is cut into, in order.
pub fn runs(store: &dyn crate::cells::Store) -> Vec<Run> {
    let size = store.size();
    let mut runs = Vec::new();
    let mut at = 0;
    while at < size {
        let end = match store.cell_run_at(at) {
            Some((start, run)) => (start + run.len()).min(size),
            None => size,
        };
        assert!(end > at, "a run that does not advance, at byte {at}");
        let mut path = String::new();
        store.field_path_at(at, &mut path);
        if path.is_empty() {
            path = format!("@{at}");
        }
        runs.push(Run {
            offset: at,
            len: end - at,
            path,
        });
        at = end;
    }
    runs
}

/// What a store holds as the golden file keeps it: the bytes of its fields that are not
/// gaps, the hash over their paths and values, and the gaps that hold a non-zero byte, each
/// with its path and bytes.
pub struct Summary {
    pub size: usize,
    pub hash: u64,
    pub gaps: Vec<(String, Vec<u8>)>,
}

/// The `Summary` of a store as it is now.
pub fn summary(store: &dyn crate::cells::Store) -> Summary {
    let mut summary = Summary {
        size: 0,
        hash: 0,
        gaps: Vec::new(),
    };
    for run in runs(store) {
        let mut bytes = vec![0; run.len];
        store
            .load(run.offset, &mut bytes)
            .expect("a store's own bytes");
        if is_gap(&run.path) {
            if bytes.iter().any(|&b| b != 0) {
                summary.gaps.push((run.path, bytes));
            }
            continue;
        }
        let mut entry = run.path.into_bytes();
        entry.push(0);
        entry.extend((run.len as u64).to_le_bytes());
        entry.extend(&bytes);
        summary.size += run.len;
        summary.hash = summary.hash.wrapping_add(hash(&entry));
    }
    summary
}

/// The golden key of the gap `path` of the store `key`.
pub fn gap_key(key: &str, path: &str) -> String {
    format!("gap:{key}:{path}")
}

/// The bytes of the game's range of the registry's store `key`, as they are laid out now: for
/// the tests that compare a store with a named read of the game's image by offset.
pub fn store_bytes(key: &str) -> Vec<u8> {
    let (_, e) = store_keys()
        .into_iter()
        .find(|(k, _)| k == key)
        .unwrap_or_else(|| panic!("no store {key} in the registry"));
    let mut mine = vec![0; e.len];
    e.store.load(0, &mut mine).expect("a store's own bytes");
    mine
}

/// The `Summary` of the registry's store `key` now.
pub fn store_summary(key: &str) -> Summary {
    let (_, e) = store_keys()
        .into_iter()
        .find(|(k, _)| k == key)
        .unwrap_or_else(|| panic!("no store {key} in the registry"));
    summary(e.store)
}

/// Why the store `key` does not hold the game's first values, or none.
pub fn store_unlike_the_games(key: &str) -> Option<String> {
    let g = golden(key);
    let mine = store_summary(key);
    if mine.size != g.size {
        return Some(format!(
            "{key}: {} bytes in the fields, the game's {}",
            mine.size, g.size
        ));
    }
    if mine.hash != g.hash {
        return Some(format!(
            "{key}: the fields' names or values differ from the game's"
        ));
    }
    let prefix = gap_key(key, "");
    let mut frozen: Vec<&str> = table()
        .keys()
        .copied()
        .filter(|k| k.starts_with(&prefix))
        .collect();
    frozen.sort_unstable();
    let mut now: Vec<String> = mine.gaps.iter().map(|(p, _)| gap_key(key, p)).collect();
    now.sort_unstable();
    if now != frozen {
        return Some(format!(
            "{key}: the gaps that start non-zero are {now:?}, the golden file's {frozen:?}"
        ));
    }
    for (path, bytes) in &mine.gaps {
        let gap = gap_key(key, path);
        if !holds(&gap, bytes) {
            return Some(format!("{gap}: the gap's bytes differ"));
        }
    }
    None
}

/// The store `key` (`store:<name>`) holds the game's first values. Hold `test_support::test_lock`.
pub fn assert_store_starts_as_the_games(key: &str) {
    if let Some(why) = store_unlike_the_games(key) {
        panic!("{why}");
    }
}

/// The bytes the registry's store `key` has.
fn store_len(key: &str) -> usize {
    store_keys()
        .into_iter()
        .find(|(k, _)| k == key)
        .map_or(0, |(_, e)| e.store.size())
}

/// The key of the registry store that starts at the typed block `block`.
fn key_of_block<T>(block: &'static T) -> String {
    let first = std::ptr::from_ref(block).addr();
    let (key, _) = store_keys()
        .into_iter()
        .find(|(_, e)| e.store.first().addr() == first)
        .unwrap_or_else(|| panic!("no registry store starts at the block"));
    let size = size_of::<T>();
    let len = store_len(&key);
    assert!(
        len <= size && size <= len.next_multiple_of(4),
        "{key}: the struct's size"
    );
    key
}

/// The typed block `block` (a registry store's record) holds the game's first values: the
/// store that starts at its first byte, compared with the golden file. Takes
/// `test_support::test_lock`.
pub fn assert_block_starts_as_the_games<T>(block: &'static T) {
    let _g = crate::test_support::test_lock();
    assert_store_starts_as_the_games(&key_of_block(block));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hash_is_fnv1a() {
        assert_eq!(hash(b""), 0xCBF2_9CE4_8422_2325);
        assert_eq!(hash(b"a"), 0xAF63_DC4C_8601_EC8C);
    }

    /// The stores other tests change while they run without the lock (the allocator's
    /// counts, the file search, the files the game-file tests open, the picture being
    /// decoded).
    const CHANGED_BY_TESTS: [&str; 4] = [
        "store:memory.state.",
        "store:files.state.",
        "store:system.owner.SYSTEM.work.files",
        "store:gfx.owner.GFX.pictures.picture",
    ];

    /// Every store starts as the game's: the one test of the stores' first values. (The
    /// stores that keep a handle where the game has an address are frozen with the handle:
    /// their own tests name the address.)
    #[test]
    fn every_store_starts_as_the_games() {
        let _g = crate::test_support::test_lock();
        let unlike: Vec<String> = store_keys()
            .into_iter()
            .map(|(key, _)| key)
            .filter(|key| !CHANGED_BY_TESTS.iter().any(|p| key.starts_with(p)))
            .filter_map(|key| store_unlike_the_games(&key))
            .collect();
        assert!(
            unlike.is_empty(),
            "unlike the game's first values: {unlike:?}"
        );
    }

    #[test]
    fn a_block_is_checked_by_its_store() {
        assert_block_starts_as_the_games(&crate::math::next_random::STATE);
    }

    #[test]
    fn every_store_has_a_first_value() {
        for (key, e) in store_keys() {
            let g = golden(&key);
            assert!(g.size <= e.store.size(), "{key}: the store is shorter");
            assert!(e.len <= e.store.size(), "{key}: the game's range is longer");
        }
    }
}
