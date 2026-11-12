//! The game's first values, frozen in `data/first_values.golden`: for every store of the
//! `stores` registry its size and a hash of the bytes it starts with, and for the constants
//! and fields the tests read, the bytes themselves under a name.
//!
//! Each line is `<key> <size> <hash>` with, for a named read, the bytes in hex after it.
//! A store's key is `store:` and its registry name (`#2`, `#3` ... after the name when the
//! registry holds it more than once, in the registry's order). The hash is FNV-1a, 64 bits.

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
    crate::stores::all()
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

/// The bytes the registry's store `key` holds now, as many as the golden file names.
pub fn store_bytes(key: &str) -> Vec<u8> {
    let (_, e) = store_keys()
        .into_iter()
        .find(|(k, _)| k == key)
        .unwrap_or_else(|| panic!("no store {key} in the registry"));
    let mut mine = vec![0; golden(key).size];
    e.store.load(0, &mut mine).expect("a store's own bytes");
    mine
}

/// The store `key` (`store:<name>`) holds the game's first values. Hold `store::test_lock`.
pub fn assert_store_starts_as_the_games(key: &str) {
    assert_matches(key, &store_bytes(key));
}

/// The typed block `block` (a registry store's record) holds the game's first values: the
/// store that starts at its first byte, compared with the golden file. Takes
/// `store::test_lock`.
pub fn assert_block_starts_as_the_games<T>(block: &'static T) {
    let _g = crate::store::test_lock();
    let first = std::ptr::from_ref(block).addr();
    let (key, _) = store_keys()
        .into_iter()
        .find(|(_, e)| e.store.first().addr() == first)
        .unwrap_or_else(|| panic!("no registry store starts at the block"));
    let size = size_of::<T>();
    let len = golden(&key).size;
    assert!(
        len <= size && size <= len.next_multiple_of(4),
        "{key}: the struct's size"
    );
    assert_store_starts_as_the_games(&key);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hash_is_fnv1a() {
        assert_eq!(hash(b""), 0xCBF2_9CE4_8422_2325);
        assert_eq!(hash(b"a"), 0xAF63_DC4C_8601_EC8C);
    }

    /// The stores whose first values differ from the game's on purpose (a handle or a Rust
    /// value where the game has an address): their own tests patch the game's bytes first.
    const NOT_THE_GAMES: [&str; 18] = [
        "store:audio.owner.AUDIO_OWNER.sample_records",
        "store:gfx.owner.GFX.card.texture_tables",
        "store:gfx.owner.GFX.fonts.font_block",
        "store:input.owner.INPUT.devices",
        "store:install.owner.INSTALL.pages",
        "store:install.owner.INSTALL.vars",
        "store:menu.owner.MENUS.cars.table",
        "store:menu.owner.MENUS.play.players",
        "store:menu.owner.MENUS.tables.lists",
        "store:menu.owner.MENUS.tables.menus.labels",
        "store:menu.owner.MENUS.tables.texts#2",
        "store:race.owner.RACE.frame.view_and_records",
        "store:race.owner.RACE.shared.view",
        "store:race.owner.RACE.shared.work",
        "store:sound.owner.SOUND.race.cue_lists",
        "store:sound.owner.SOUND.race.kinds",
        "store:sound.owner.SOUND.race.sets",
        "store:sound.owner.SOUND.race.surface_sets",
    ];

    /// The stores other tests change while they run without the lock (the allocator's
    /// counts, the file search, the files the game-file tests open, the picture being
    /// decoded).
    const CHANGED_BY_TESTS: [&str; 4] = [
        "store:memory.state.",
        "store:files.state.",
        "store:system.owner.SYSTEM.work.files",
        "store:gfx.owner.GFX.pictures.picture",
    ];

    /// Every other store starts as the game's: the one test of the stores' first values.
    #[test]
    fn every_store_starts_as_the_games() {
        let _g = crate::store::test_lock();
        let unlike: Vec<String> = store_keys()
            .into_iter()
            .map(|(key, _)| key)
            .filter(|key| !NOT_THE_GAMES.contains(&key.as_str()))
            .filter(|key| !CHANGED_BY_TESTS.iter().any(|p| key.starts_with(p)))
            .filter(|key| hash(&store_bytes(key)) != golden(key).hash)
            .collect();
        assert!(
            unlike.is_empty(),
            "unlike the game's first values: {unlike:?}"
        );
    }

    /// With `RC2K_FIRST_VALUE_DIFFERENCES` set, lists the stores whose bytes differ from the
    /// game's first values (a survey: some hold handles where the game holds addresses).
    #[test]
    fn list_stores_unlike_the_game() {
        let Ok(out) = std::env::var("RC2K_FIRST_VALUE_DIFFERENCES") else {
            return;
        };
        let _g = crate::store::test_lock();
        let mut text = String::new();
        for (key, _) in store_keys() {
            let mine = store_bytes(&key);
            if hash(&mine) != golden(&key).hash {
                text.push_str(&format!("{key} {}\n", mine.len()));
            }
        }
        std::fs::write(out, text).unwrap();
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
        }
    }
}
