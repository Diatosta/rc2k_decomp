//! `files::read_packed_file` (0x41BA71): a file packed by the game's "SQSL" scheme read and
//! unpacked in place.
//!
//! In:  eax = path, ebx = buffer, ecx = its bytes
//! Out: eax = the bytes unpacked, carry clear, zero flag = none; on a failed read or a
//!      file not "SQSL", eax = 0, carry set, zero clear; everything else preserved
//!
//! The file (up to ecx bytes, `files::read_file`) begins "SQSL", two dwords, and the count
//! of dwords it unpacks to. Its dwords are moved to the buffer's end (backwards, so a file
//! filling the buffer stays in place); from +0x10 of them a stream of bits (each dword's
//! highest first) is read and the dwords written from the buffer's start (both + the
//! size's odd bytes: the move shifts the file by them), each by a code: 0 a zero; 10 and 8 bits n, the
//! dword n + 1 back; 1100 and 12 bits, likewise; 1101 and 17 bits, that number; 1110, a
//! sign bit and 27 bits, that number (28 bits, signed); 1111 and 32 bits, that number. Read
//! from the course data (`files::read_level_file`). Refuses a count of 0 (the original
//! counts round), and a stream that reads or writes past the buffer or refers back past its
//! start (the original would carry on in other memory).
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::Refused;
use crate::files::read_file::ReadResult;
use crate::pointers::GamePtr;

use rc2k_core::files::packed::MAGIC;

/// What the original leaves: eax and two flags.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub eax: u32,
    pub carry: bool,
    pub zero: bool,
}

impl Outcome {
    /// What `files::read_file` leaves on a read of `path`: eax the bytes read (`path` when
    /// it did not get that far), carry a failure, zero success.
    pub fn of_read(path: u32, got: &ReadResult) -> Self {
        Self {
            eax: got.bytes_read.unwrap_or(path),
            carry: !got.ok,
            zero: got.ok,
        }
    }
}

/// `files::read_file`'s call (the path, the buffer, its bytes; the offset 0).
pub type Read<'a> = &'a mut dyn FnMut(u32, u32, u32) -> Result<ReadResult, Refused>;

/// In the game: `files::read_file` in Rust.
pub fn game_read(path: u32, buffer: u32, bytes: u32) -> Result<ReadResult, Refused> {
    Ok(crate::files::read_file::read_file(
        crate::pointers::GamePtr::new(path),
        crate::pointers::host(buffer) as _,
        bytes,
        0,
    )?)
}

fn get(at: u32) -> Result<u32, crate::errors::BadInput> {
    GamePtr::new(at).read::<u32>("a packed file's mark")
}

/// The unpacking of a file of `size` bytes read into the buffer of `room`: the bytes
/// written (from the buffer's start). Converted: `rc2k_core::files::packed::unpack`.
pub fn unpack(buffer: u32, room: u32, size: u32) -> Result<u32, Refused> {
    let at = GamePtr::new(buffer);
    let mut bytes = at.read_bytes(room as usize, "a packed file's buffer")?;
    let written = rc2k_core::files::packed::unpack(&mut bytes, size as usize)? as u32;
    at.write_bytes(&bytes, "a packed file's buffer")?;
    Ok(written)
}

/// `files::read_packed_file` (see the header): the file at `path` into `buffer` of `room`
/// bytes.
pub fn read_packed_file(path: u32, buffer: u32, room: u32, read: Read) -> Result<Outcome, Refused> {
    let got = Outcome::of_read(path, &read(path, buffer, room)?);
    // A failed read leaves zero clear (`Outcome::of_read`), as a file not "SQSL" does.
    let failed = Outcome {
        eax: 0,
        carry: true,
        zero: false,
    };
    if got.carry {
        return Ok(failed);
    }
    if get(buffer)? != MAGIC {
        return Ok(failed);
    }
    let bytes = unpack(buffer, room, got.eax)?;
    Ok(Outcome {
        eax: bytes,
        carry: false,
        zero: bytes == 0,
    })
}
