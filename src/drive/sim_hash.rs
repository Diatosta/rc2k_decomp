//! The hash of the car blocks that `RC2K_SIM_HASH` logs after each tick of a race (see
//! `drive::race_step`). It is taken by field path and value, as the first-values golden is:
//! the blocks are cut into runs of like cells, each named by its field path (`.cars[3].gear`),
//! and the hash is the sum of one hash per run over its path, length and bytes. The order of
//! the fields and the bytes between them do not count, so a change to the records' layout
//! leaves the hash of the same state as it was. Gap fields (`_gap_*`, `_lead*`, `_pad*`,
//! `_tail*`: bytes no code names) are left out.
//!
//! Only the eight cars are taken by field: the replay staging (the frame's file image, which
//! the 1999 layout kept in car blocks 8 and 9, the tail and the replay tape) is hashed as one
//! group of its bytes, `.staging`, so that it hashes the same wherever it is kept.

use crate::memory::cells::{CellVisitor, Cells};
use crate::race::blocks_store::CarBlocks;

const GAP_PREFIXES: [&str; 4] = ["_gap_", "_lead", "_pad", "_tail"];

/// The field path holds a gap field (`.cars[0]._gap_f4`).
fn is_gap(path: &str) -> bool {
    path.split('.').skip(1).any(|part| {
        let name = part.split('[').next().unwrap_or(part);
        GAP_PREFIXES.iter().any(|p| name.starts_with(p))
    })
}

/// FNV-1a over `bytes`, 64 bits.
fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3)
    })
}

/// The hash of the eight cars' fields in groups, one for each car (`.cars[3]`): each is the sum of one hash per run of like cells over its path, length and
/// bytes. The groups added up are the whole blocks' hash (`total`).
pub fn car_blocks_groups(
    blocks: &CarBlocks,
    parts: &crate::race::car_parts_store::CarParts,
) -> Vec<(String, u64)> {
    let mut hasher = GroupHasher::default();
    blocks.visit(&mut hasher);
    // The drawing's own fields of each car were once in its block: the same paths.
    crate::memory::plain_record::PlainRecord::visit(parts, &mut hasher);
    #[cfg(any(test, feature = "remote"))]
    hasher.add_removed_fields();
    hasher.groups
}

/// A check: every field of the cars' blocks, one line each, `sim field <tick> <path> <hash>`.
#[cfg(feature = "remote")]
pub fn log_fields(blocks: &CarBlocks, parts: &crate::race::car_parts_store::CarParts, tick: u32) {
    struct Each {
        path: String,
        marks: Vec<usize>,
        tick: u32,
    }
    impl CellVisitor for Each {
        fn enter(&mut self, name: &'static str) {
            self.marks.push(self.path.len());
            self.path.push('.');
            self.path.push_str(name);
        }
        fn index(&mut self, i: usize) {
            self.marks.push(self.path.len());
            self.path.push_str(&format!("[{i}]"));
        }
        fn leave(&mut self) {
            let mark = self.marks.pop().expect("a part entered");
            self.path.truncate(mark);
        }
        fn leaf(&mut self, bytes: &[u8]) {
            if !is_gap(&self.path) {
                crate::program_log::log(&format!(
                    "sim field {} {} {:016x}",
                    self.tick,
                    self.path,
                    fnv1a(bytes)
                ));
            }
        }
    }
    let mut each = Each {
        path: String::new(),
        marks: Vec::new(),
        tick,
    };
    blocks.visit(&mut each);
    crate::memory::plain_record::PlainRecord::visit(parts, &mut each);
}

/// The hash of one whole record by field path and value (the cars' hash for any other store):
/// its layout and its gap fields do not count. The tests' reference for the hash's scheme.
#[cfg(test)]
pub fn record_hash<T: Cells>(record: &T) -> u64 {
    let mut hasher = GroupHasher::default();
    record.visit(&mut hasher);
    total(&hasher.groups)
}

/// The hash `record_hash` adds for one run of dwords at `path` (`.counters[0]`, or `.kind_slot_bytes`
/// for a lone field): a store of plain fields hashes as the cell store it replaced by adding these.
#[cfg(feature = "remote")]
pub fn dword_run_hash(path: &str, dwords: &[u32]) -> u64 {
    let bytes: Vec<u8> = dwords.iter().flat_map(|d| d.to_le_bytes()).collect();
    byte_run_hash(path, &bytes)
}

/// The hash `record_hash` adds for one run of bytes at `path`.
#[cfg(any(test, feature = "remote"))]
pub fn byte_run_hash(path: &str, bytes: &[u8]) -> u64 {
    let mut entry = path.as_bytes().to_vec();
    entry.push(0);
    entry.extend((bytes.len() as u64).to_le_bytes());
    entry.extend(bytes);
    fnv1a(&entry)
}

/// FNV-1a of `bytes` and their count, for a store that is a table of bytes.
#[cfg(any(test, feature = "remote"))]
pub fn bytes_hash(bytes: &[u8]) -> u64 {
    let mut entry = (bytes.len() as u64).to_le_bytes().to_vec();
    entry.extend(bytes);
    fnv1a(&entry)
}

/// The groups built from `Cells::visit`: the field path kept as the visit goes in and out.
#[derive(Default)]
struct GroupHasher {
    path: String,
    /// The path's length before each part entered.
    marks: Vec<usize>,
    groups: Vec<(String, u64)>,
}

impl GroupHasher {
    /// Adds the runs of the cars' fields that were removed, which always held 0 (`state_bits`),
    /// so that the hash of a state stays the one it always was.
    #[cfg(any(test, feature = "remote"))]
    fn add_removed_fields(&mut self) {
        for car in 0..crate::race::blocks_store::BLOCK_COUNT {
            let path = format!(".cars[{car}].state_bits");
            let hash = byte_run_hash(&path, &[0; 4]);
            let group = path_group(&path);
            match self.groups.iter_mut().find(|g| g.0 == group) {
                Some(g) => g.1 = g.1.wrapping_add(hash),
                None => self.groups.push((group, hash)),
            }
        }
    }
}

impl CellVisitor for GroupHasher {
    fn enter(&mut self, name: &'static str) {
        self.marks.push(self.path.len());
        self.path.push('.');
        self.path.push_str(name);
    }

    fn index(&mut self, i: usize) {
        self.marks.push(self.path.len());
        self.path.push_str(&format!("[{i}]"));
    }

    fn leave(&mut self) {
        let mark = self.marks.pop().expect("a part entered");
        self.path.truncate(mark);
    }

    fn leaf(&mut self, bytes: &[u8]) {
        if is_gap(&self.path) {
            return;
        }
        let mut entry = self.path.clone().into_bytes();
        entry.push(0);
        entry.extend((bytes.len() as u64).to_le_bytes());
        entry.extend(bytes);
        let hash = fnv1a(&entry);
        let group = path_group(&self.path);
        match self.groups.iter_mut().find(|g| g.0 == group) {
            Some(g) => g.1 = g.1.wrapping_add(hash),
            None => self.groups.push((group, hash)),
        }
    }
}

/// The replay staging's group: its bytes (the frame's file image) as one run named `.staging`,
/// but for the hat fields of the tape's input words (`HAT_FIELDS`): a device reading the input
/// thread makes on its own clock, 0 until its first poll and the centred hat (0xE00) after, so
/// the same tick recorded 0 in one run and 0xE00 in another. The cars never see those bits (the
/// computer drives them), only the replay file keeps them, and the file is not changed.
#[cfg(feature = "remote")]
pub fn staging_group(bytes: &[u8]) -> (String, u64) {
    use rc2k_core::drive::replay_frames::START_DWORDS;
    /// The hat's angle (the top byte) and direction (the second byte's low nibble) in the
    /// device's bits word, which the tape keeps as a car's first input word.
    const HAT_FIELDS: u32 = 0xFF00_0F00;
    let mut masked = bytes.to_vec();
    // The tape's entries are two dwords each (the input word, the packed rest).
    for entry in masked
        .get_mut(4 * START_DWORDS..)
        .unwrap_or_default()
        .as_chunks_mut::<8>()
        .0
    {
        let input = u32::from_le_bytes([entry[0], entry[1], entry[2], entry[3]]) & !HAT_FIELDS;
        entry[..4].copy_from_slice(&input.to_le_bytes());
    }
    let mut entry = b".staging\0".to_vec();
    entry.extend((masked.len() as u64).to_le_bytes());
    entry.extend(masked);
    (".staging".to_string(), fnv1a(&entry))
}

/// The whole blocks' hash: the groups' hashes added up.
pub fn total(groups: &[(String, u64)]) -> u64 {
    groups.iter().fold(0, |sum, g| sum.wrapping_add(g.1))
}

/// The first two parts of a field path: `.cars[3]` of `.cars[3].gear`.
fn path_group(path: &str) -> String {
    let second = path[1..].find('.').map_or(path.len(), |i| i + 1);
    path[..second].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::race::car_parts_store::CarParts;

    /// Varied bytes through every run of the blocks (the gap fields too).
    fn filled(seed: u32) -> CarBlocks {
        struct Fill(u32);
        impl CellVisitor for Fill {
            fn enter(&mut self, _: &'static str) {}
            fn index(&mut self, _: usize) {}
            fn leave(&mut self) {}
            fn leaf(&mut self, _: &[u8]) {}
            fn run(&mut self, run: crate::memory::cells::CellRun<'_>) {
                let bytes: Vec<u8> = (0..run.len())
                    .map(|_| {
                        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                        (self.0 >> 24) as u8
                    })
                    .collect();
                run.store(0, &bytes);
            }
        }
        let blocks = CarBlocks::new();
        blocks.visit(&mut Fill(seed));
        blocks
    }

    #[test]
    fn the_groups_are_one_per_car() {
        for blocks in [CarBlocks::new(), filled(1), filled(0xDEAD_BEEF)] {
            let groups = car_blocks_groups(&blocks, &CarParts::new());
            assert_eq!(groups.len(), 8);
        }
    }

    #[test]
    fn a_field_changes_the_hash() {
        let (blocks, mut parts) = (&CarBlocks::new(), CarParts::new());
        let parts = &mut parts;
        let first = total(&car_blocks_groups(blocks, parts));
        assert_eq!(first, total(&car_blocks_groups(blocks, parts)));
        use crate::test_support::paths::set_u32;
        assert!(set_u32(blocks, ".cars[2].gear", 0x7B));
        assert_ne!(first, total(&car_blocks_groups(blocks, parts)));
        // A field of the drawing's state counts as one of the car's.
        let before = total(&car_blocks_groups(blocks, parts));
        parts.cars[5].mode_on = 1;
        assert_ne!(before, total(&car_blocks_groups(blocks, parts)));
        let groups = car_blocks_groups(blocks, parts);
        assert_eq!(groups.len(), 8);
        assert_eq!(groups[2].0, ".cars[2]");
    }

    /// A gap field (`_pad_*`) is left out of the hash.
    #[test]
    fn a_gap_does_not_change_the_hash() {
        #[derive(rc2k_derive::Cells)]
        struct Padded {
            value: crate::memory::session::PlainDword,
            _pad_value: crate::memory::session::PlainDword,
        }
        let record: Padded = crate::memory::cells::zeroed_cells();
        let first = record_hash(&record);
        record._pad_value.set(0x55);
        assert_eq!(first, record_hash(&record));
        record.value.set(1);
        assert_ne!(first, record_hash(&record));
    }
}
