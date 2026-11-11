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

use crate::errors::{BadInput, Refused};
use crate::files::read_file::ReadResult;

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

/// `files::read_file`'s call (the path, the buffer; the offset 0).
pub fn read(path: &[u8], buffer: &mut [u8]) -> Result<ReadResult, Refused> {
    Ok(crate::files::read_file::read_file_into(path, buffer, 0)?)
}

/// The unpacking of a file of `size` bytes read into `buffer`: the bytes written (from the
/// buffer's start). Converted: `rc2k_core::files::packed::unpack`.
pub fn unpack(buffer: &mut [u8], size: u32) -> Result<u32, Refused> {
    Ok(rc2k_core::files::packed::unpack(buffer, size as usize)? as u32)
}

/// `files::read_packed_file` (see the header): the file at `path` into `buffer`, whose length
/// is the room.
pub fn read_packed_file(path: &[u8], buffer: &mut [u8]) -> Result<Outcome, Refused> {
    let got = Outcome::of_read(&read(path, buffer)?);
    // A failed read leaves zero clear (`Outcome::of_read`), as a file not "SQSL" does.
    let failed = Outcome {
        eax: 0,
        carry: true,
        zero: false,
    };
    if got.carry {
        return Ok(failed);
    }
    let Some(mark) = buffer.first_chunk::<4>() else {
        return Err(BadInput("a packed file's mark past its buffer").into());
    };
    if u32::from_le_bytes(*mark) != MAGIC {
        return Ok(failed);
    }
    let bytes = unpack(buffer, got.eax)?;
    Ok(Outcome {
        eax: bytes,
        carry: false,
        zero: bytes == 0,
    })
}
