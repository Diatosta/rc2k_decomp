//! The install script (from `install::script_name` 0x41370C and
//! `install::run_script` 0x41365D): a byte program that names the folders and files to
//! copy, as nested folders. Opcodes: 1 a folder (its name follows) entered, 2 back to the
//! parent, 3 files matching a pattern, 4 a folder and everything under it, 8 n what follows
//! up to the matching 9 only for an installation of kind n or more (8 and 9 nest), 6 and 9
//! nothing; anything else ends it. The folders and files it names go into two tables
//! (`install::list_folder` 0x413876 and `install::list_files` 0x4138B2,
//! day): names with NULs, past whose room is a fatal error.

use crate::Invalid;
use crate::strings::bytes::Bytes;

/// The longest name looked for before a name is taken to have no end.
pub const NAME_LIMIT: usize = 0x1000;
/// The bytes a script may take (its skips included) before it is taken to have no end.
pub const LIMIT: usize = 0x10000;

/// A name in the script from its start to the first byte below a space (a control byte or
/// the end): the name and that byte. Refused: none in `NAME_LIMIT` bytes.
pub fn script_name(script: &(impl Bytes + ?Sized)) -> Result<(Vec<u8>, u8), Invalid> {
    let mut name = Vec::new();
    for n in 0..NAME_LIMIT {
        let c = script
            .byte(n)
            .ok_or(Invalid("a script name past the script"))?;
        if c < 0x20 {
            return Ok((name, c));
        }
        name.push(c);
    }
    Err(Invalid(
        "a script name with no control byte in 0x1000 bytes",
    ))
}

/// What an opcode does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    /// 1: a folder entered.
    Folder,
    /// 2: back to the parent.
    Up,
    /// 3: the files matching a pattern.
    Files,
    /// 4: a folder and everything under it.
    Tree,
    /// 8 n: up to the matching 9 only for an installation of kind n or more.
    Media,
    /// 6, 9: nothing.
    Nothing,
    /// Anything else: the end (with folders still entered: fatal error 0x80AF3011).
    End,
}

/// The fatal error of a script that ends with folders still entered.
pub const UNBALANCED: u32 = 0x80AF_3011;

impl Op {
    pub fn of(byte: u8) -> Op {
        match byte {
            1 => Op::Folder,
            2 => Op::Up,
            3 => Op::Files,
            4 => Op::Tree,
            8 => Op::Media,
            6 | 9 => Op::Nothing,
            _ => Op::End,
        }
    }
}

/// Whether the part after an 8 n is passed over for installation kind `media`.
pub fn skipped(n: u8, media: u8) -> bool {
    n > media
}

/// From `at` (past the 8 n) to past the matching 9, byte by byte as the original (a name's
/// bytes 8 and 9 count too); `steps` counts the bytes taken against `LIMIT`. Refused: no
/// 9 in time.
pub fn skip(
    script: &(impl Bytes + ?Sized),
    at: usize,
    steps: &mut usize,
) -> Result<usize, Invalid> {
    let mut at = at;
    let mut depth: i8 = 0;
    loop {
        *steps += 1;
        if *steps > LIMIT {
            return Err(Invalid("a skip with no 9 in 0x10000 bytes"));
        }
        let b = script.byte(at).ok_or(Invalid("a skip past the script"))?;
        at += 1;
        if b == 8 {
            depth = depth.wrapping_add(1);
        }
        if b == 9 {
            depth = depth.wrapping_sub(1);
            if depth < 0 {
                return Ok(at);
            }
        }
    }
}

/// The folders' table's room and the fatal error past it.
pub const FOLDERS_ROOM: u32 = 0x10000;
pub const FOLDERS_FULL: u32 = 0x80AF_2205;
/// The files' table's room and the fatal error past it.
pub const FILES_ROOM: u32 = 0x40000;
pub const FILES_FULL: u32 = 0x80AF_2206;

/// A table's bytes used (the cursor less the table, unsigned) checked against its room:
/// the fatal error when not below it.
pub fn room_left(used: u32, room: u32, full: u32) -> Result<(), u32> {
    if used >= room { Err(full) } else { Ok(()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rooms() {
        assert_eq!(room_left(0xFFFF, FOLDERS_ROOM, FOLDERS_FULL), Ok(()));
        assert_eq!(
            room_left(0x10000, FOLDERS_ROOM, FOLDERS_FULL),
            Err(0x80AF_2205)
        );
        assert_eq!(room_left(0x40000, FILES_ROOM, FILES_FULL), Err(0x80AF_2206));
    }

    #[test]
    fn names_and_skips() {
        assert_eq!(script_name(b"abc\x01x"), Ok((b"abc".to_vec(), 1)));
        assert_eq!(script_name(b"\0"), Ok((Vec::new(), 0)));
        assert!(script_name(&[b'a'; NAME_LIMIT]).is_err());
        assert_eq!(Op::of(1), Op::Folder);
        assert_eq!(Op::of(9), Op::Nothing);
        assert_eq!(Op::of(0), Op::End);
        assert!(skipped(2, 1));
        assert!(!skipped(1, 1));
        let mut steps = 0;
        assert_eq!(skip(b"\x01a\x08\x02\x09b\x09c", 0, &mut steps), Ok(7));
        assert_eq!(steps, 7);
        let mut steps = LIMIT;
        assert!(skip(b"\x09", 0, &mut steps).is_err());
    }
}
