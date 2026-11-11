//! `files::read_packed_file` (0x41BA71): a file packed by the game's "SQSL" scheme read and
//! unpacked in place.
//!
//! Takes the path, the buffer and its size. Returns the bytes unpacked; on a failed read or a
//! file not "SQSL", 0 and a failure.
//!
//! The file (up to the given bytes, `files::read_file`) begins "SQSL", two dwords, and the count of
//! dwords it unpacks to. Its dwords are moved to the buffer's end (backwards, so a file filling the
//! buffer stays in place); from +0x10 of them a stream of bits (each dword's highest first) is read
//! and the dwords written from the buffer's start (both + the size's odd bytes: the move shifts the
//! file by them), each by a code: 0 a zero; 10 and 8 bits n, the dword n + 1 back; 1100 and 12
//! bits, likewise; 1101 and 17 bits, that number; 1110, a sign bit and 27 bits, that number (28
//! bits, signed); 1111 and 32 bits, that number. Read from the course data
//! (`files::read_level_file`). Refuses a count of 0 (the original counts round), and a stream that
//! reads or writes past the buffer or refers back past its start (the original would carry on in
//! other memory).

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
    /// What `files::read_file` leaves on a read: `eax` the bytes read (the original leaves
    /// the path's address when it did not get that far; here 0, nobody reads it then),
    /// carry a failure, zero success.
    pub fn of_read(got: &ReadResult) -> Self {
        Self {
            eax: got.bytes_read.unwrap_or(0),
            carry: !got.ok,
            zero: got.ok,
        }
    }
}

/// `files::read_file`'s call (the path, the buffer, its bytes; the offset 0).
pub fn read(path: &[u8], buffer: GamePtr, bytes: u32) -> Result<ReadResult, Refused> {
    Ok(crate::files::read_file::read_file(path, buffer, bytes, 0)?)
}

/// The unpacking of a file of `size` bytes read into the buffer of `room`: the bytes
/// written (from the buffer's start). Converted: `rc2k_core::files::packed::unpack`.
pub fn unpack(buffer: GamePtr, room: u32, size: u32) -> Result<u32, Refused> {
    let at = buffer;
    let mut bytes = at.read_bytes(room as usize, "a packed file's buffer")?;
    let written = rc2k_core::files::packed::unpack(&mut bytes, size as usize)? as u32;
    at.write_bytes(&bytes, "a packed file's buffer")?;
    Ok(written)
}

/// `files::read_packed_file` (see the header): the file at `path` into `buffer` of `room`
/// bytes.
pub fn read_packed_file(path: &[u8], buffer: GamePtr, room: u32) -> Result<Outcome, Refused> {
    let got = Outcome::of_read(&read(path, buffer, room)?);
    // A failed read leaves zero clear (`Outcome::of_read`), as a file not "SQSL" does.
    let failed = Outcome {
        eax: 0,
        carry: true,
        zero: false,
    };
    if got.carry {
        return Ok(failed);
    }
    if buffer.read::<u32>("a packed file's mark")? != MAGIC {
        return Ok(failed);
    }
    let bytes = unpack(buffer, room, got.eax)?;
    Ok(Outcome {
        eax: bytes,
        carry: false,
        zero: bytes == 0,
    })
}
