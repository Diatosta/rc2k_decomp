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

use crate::cells::Cells;
use crate::race::blocks_store::CarBlocks;

/// The prefixes of the names of gap fields.
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
pub fn car_blocks_groups(blocks: &CarBlocks) -> Vec<(String, u64)> {
    let size = size_of::<CarBlocks>();
    let mut groups: Vec<(String, u64)> = Vec::new();
    let mut at = 0;
    while at < size {
        let (start, run) = blocks.cell_run(at);
        let end = (start + run.len()).min(size);
        let mut path = String::new();
        blocks.field_path(at, &mut path);
        if !is_gap(&path) {
            let mut bytes = vec![0; end - at];
            run.load(at - start, &mut bytes);
            let group = path_group(&path);
            let mut entry = path.into_bytes();
            entry.push(0);
            entry.extend(((end - at) as u64).to_le_bytes());
            entry.extend(&bytes);
            let hash = fnv1a(&entry);
            match groups.iter_mut().find(|g| g.0 == group) {
                Some(g) => g.1 = g.1.wrapping_add(hash),
                None => groups.push((group, hash)),
            }
        }
        at = end;
    }
    groups
}

/// The replay staging's group: its bytes (the frame's file image) as one run named `.staging`.
pub fn staging_group(bytes: &[u8]) -> (String, u64) {
    let mut entry = b".staging\0".to_vec();
    entry.extend((bytes.len() as u64).to_le_bytes());
    entry.extend(bytes);
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

    #[test]
    fn a_field_changes_the_hash_and_a_gap_does_not() {
        let blocks = &CarBlocks::new();
        let first = total(&car_blocks_groups(blocks));
        assert_eq!(first, total(&car_blocks_groups(blocks)));
        let car = 2 * size_of::<crate::physics::car_store::Car>();
        let gap = car + core::mem::offset_of!(crate::physics::car_store::Car, _gap_f4) + 3;
        crate::cells::store_cells(blocks, gap, &[0x55]).unwrap();
        assert_eq!(first, total(&car_blocks_groups(blocks)));
        let gear = car + core::mem::offset_of!(crate::physics::car_store::Car, gear);
        crate::cells::store_cells(blocks, gear, &[0x7B]).unwrap();
        assert_ne!(first, total(&car_blocks_groups(blocks)));
        let groups = car_blocks_groups(blocks);
        assert_eq!(groups.len(), 8);
        assert_eq!(groups[2].0, ".cars[2]");
    }
}
