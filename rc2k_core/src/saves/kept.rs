//! The two files the game keeps between runs (from `saves::load_options`
//! 0x4AAB32, `save_options` 0x4AAE57, `load_best_times` 0x4AB155, `save_best_times`
//! 0x4AB236): each begins with a head of 0x10 bytes, its four letters, 0, the version
//! 0x50004 and the time it was written. `var\ral.dat` ("R5CI") then has the 69 pieces of
//! the options one after the other and "####"; `var\hst.dat` ("R5HS") the six best times'
//! tables. A car setup, `savedata\carsetup\carsetNN` ("CRSU", version 0x50000, from
//! `service::write_setup_slot` 0x4AB378 / `read_setup_slot` 0x4AB31E), is 0x48 bytes: the
//! head, the slot's name (0x20 bytes) and six values. A saved game ("R5SG", 0x51122,
//! `saves::write_savegame_slot` 0x4AB558 / `read_savegame_slot` 0x4AB4FE) and a replay's
//! head ("REPL", 0x50400, `saves::read_replay_slot` 0x41ECAD) have the slot's name at +0x10
//! too.

use crate::Invalid;

/// The options' file.
pub const OPTIONS: [u8; 4] = *b"R5CI";
/// The best times' file.
pub const BEST_TIMES: [u8; 4] = *b"R5HS";
pub const VERSION: u32 = 0x50004;
pub const HEAD_LEN: usize = 0x10;
/// What ends the options' file.
pub const END_MARK: [u8; 4] = *b"####";

/// A file's head.
pub fn head(letters: [u8; 4], time: u32) -> [u8; HEAD_LEN] {
    let mut h = [0u8; HEAD_LEN];
    h[..4].copy_from_slice(&letters);
    h[8..12].copy_from_slice(&VERSION.to_le_bytes());
    h[12..].copy_from_slice(&time.to_le_bytes());
    h
}

/// Whether `block` begins with a head of these letters and the version (the 0 and the
/// time not looked at).
pub fn head_ok(block: &[u8], letters: [u8; 4]) -> bool {
    block.len() >= HEAD_LEN && block[..4] == letters && block[8..12] == VERSION.to_le_bytes()
}

/// The options' pieces as `block` (what was read, and what the block held past it) has
/// them after the head, of the sizes given; and whether "####" follows them (else the
/// game's fatal error 0x80ACFF01, "ral.dat out of date", once they are copied). Refused: a
/// block too short for them.
pub fn pieces<'a>(block: &'a [u8], sizes: &[usize]) -> Result<(Vec<&'a [u8]>, bool), Invalid> {
    let total: usize = sizes.iter().sum();
    if block.len() < HEAD_LEN + total + 4 {
        return Err(Invalid("an options block too short for its pieces"));
    }
    let mut at = HEAD_LEN;
    let mut out = Vec::with_capacity(sizes.len());
    for &len in sizes {
        out.push(&block[at..at + len]);
        at += len;
    }
    Ok((out, block[at..at + 4] == END_MARK))
}

/// The options' file: its head, the pieces, "####".
pub fn options_file(time: u32, pieces: &[&[u8]]) -> Vec<u8> {
    let mut file = head(OPTIONS, time).to_vec();
    for p in pieces {
        file.extend_from_slice(p);
    }
    file.extend_from_slice(&END_MARK);
    file
}

/// A car setup's file.
pub const SETUP: [u8; 4] = *b"CRSU";
pub const SETUP_VERSION: u32 = 0x5_0000;
pub const SETUP_LEN: usize = 0x48;
pub const SETUP_NAME_LEN: usize = 0x20;

/// A head of these letters and version.
pub fn head_with(letters: [u8; 4], version: u32, time: u32) -> [u8; HEAD_LEN] {
    let mut h = head(letters, time);
    h[8..12].copy_from_slice(&version.to_le_bytes());
    h
}

/// Whether `block` begins with a head of these letters and version.
pub fn head_ok_with(block: &[u8], letters: [u8; 4], version: u32) -> bool {
    block.len() >= HEAD_LEN && block[..4] == letters && block[8..12] == version.to_le_bytes()
}

/// A car setup's file: the head, the slot's name, the tyre, brake balance, ride height,
/// suspension stiffness, steering sensitivity and gear ratio.
pub fn setup_file(time: u32, name: &[u8; SETUP_NAME_LEN], values: [u32; 6]) -> [u8; SETUP_LEN] {
    let mut f = [0u8; SETUP_LEN];
    f[..0x30].copy_from_slice(&slot_head(SETUP, SETUP_VERSION, time, name));
    for (n, v) in values.into_iter().enumerate() {
        f[0x30 + 4 * n..0x34 + 4 * n].copy_from_slice(&v.to_le_bytes());
    }
    f
}

/// A saved game's file.
pub const SAVEGAME: [u8; 4] = *b"R5SG";
pub const SAVEGAME_VERSION: u32 = 0x5_1122;
/// A replay's file.
pub const REPLAY: [u8; 4] = *b"REPL";
pub const REPLAY_VERSION: u32 = 0x5_0400;

/// The name of a slot's file (at +0x10), if its head has these letters and version.
pub fn slot_name(file: &[u8], letters: [u8; 4], version: u32) -> Option<[u8; SETUP_NAME_LEN]> {
    (file.len() >= 0x30 && head_ok_with(file, letters, version))
        .then(|| file[0x10..0x30].try_into().unwrap())
}

/// A slot's head and name, the first 0x30 bytes of its file.
pub fn slot_head(
    letters: [u8; 4],
    version: u32,
    time: u32,
    name: &[u8; SETUP_NAME_LEN],
) -> [u8; 0x30] {
    let mut h = [0u8; 0x30];
    h[..HEAD_LEN].copy_from_slice(&head_with(letters, version, time));
    h[0x10..].copy_from_slice(name);
    h
}

/// The name of a car setup's file, if its head is right.
pub fn setup_name(file: &[u8; SETUP_LEN]) -> Option<[u8; SETUP_NAME_LEN]> {
    slot_name(file, SETUP, SETUP_VERSION)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn files() {
        let file = options_file(7, &[b"ab", b"cde"]);
        assert!(head_ok(&file, OPTIONS) && !head_ok(&file, BEST_TIMES));
        assert_eq!(file.len(), HEAD_LEN + 5 + 4);
        let (p, ended) = pieces(&file, &[2, 3]).unwrap();
        assert_eq!((p, ended), (vec![&b"ab"[..], &b"cde"[..]], true));
        assert!(!pieces(&file, &[2, 2]).unwrap().1);
        assert!(pieces(&file, &[2, 9]).is_err());
        assert_eq!(&head(BEST_TIMES, 9)[12..], &9u32.to_le_bytes());
        let f = setup_file(1, &[b'n'; 0x20], [1, 2, 3, 4, 5, 6]);
        assert_eq!((setup_name(&f), f[0x44]), (Some([b'n'; 0x20]), 6));
    }
}
