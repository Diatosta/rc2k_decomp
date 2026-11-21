//! A file packed by the "SQSL" scheme read and unpacked in place: the file begins "SQSL", two
//! dwords, and the count of dwords it unpacks to; its dwords are moved to the buffer's end, and a
//! stream of bits (each dword's highest first) is decoded from the buffer's start, each code
//! being a zero, a back reference of 8 or 12 bits, or a literal of 17, 28 (signed) or 32 bits.
//! Gives the bytes unpacked; on a failed read or a file not "SQSL", `None`. A
//! count of 0, and a stream that reads or writes past the buffer or refers back past its start,
//! is a damaged file (`Recover`).

use crate::errors::{BadInput, Refused};
use crate::files::read_file::ReadResult;

use rc2k_core::files::packed::MAGIC;

/// The bytes a read got (0 when it did not get that far), or `None` when it failed.
pub(super) fn bytes_of_read(got: &ReadResult) -> Option<u32> {
    got.ok.then(|| got.bytes_read.unwrap_or(0))
}

/// `files::read_file`'s call (the path, the buffer; the offset 0).
pub fn read(path: &[u8], buffer: &mut [u8]) -> Result<ReadResult, Refused> {
    Ok(crate::files::read_file::read_file_into(path, buffer, 0)?)
}

/// The unpacking of a file of `size` bytes read into `buffer`: the bytes written (from the
/// buffer's start). The logic is in `rc2k_core::files::packed::unpack`.
pub fn unpack(buffer: &mut [u8], size: u32) -> Result<u32, Refused> {
    Ok(rc2k_core::files::packed::unpack(buffer, size as usize)? as u32)
}

/// The file at `path` into `buffer`, whose length is the room: the bytes unpacked, or `None`
/// when the read failed or the file is not "SQSL".
pub(super) fn read_packed_file(path: &[u8], buffer: &mut [u8]) -> Result<Option<u32>, Refused> {
    let Some(size) = bytes_of_read(&read(path, buffer)?) else {
        return Ok(None);
    };
    let Some(mark) = buffer.first_chunk::<4>() else {
        return Err(BadInput("a packed file's mark past its buffer").into());
    };
    if u32::from_le_bytes(*mark) != MAGIC {
        return Ok(None);
    }
    // A stream that does not unpack is a damaged file: a trouble the game can go back from.
    let bytes = match unpack(buffer, size) {
        Err(Refused::BadInput(why)) => {
            crate::program_log::log(&format!(
                "files: the packed file {} is damaged: {}",
                String::from_utf8_lossy(path),
                why.0
            ));
            return Err(crate::errors::Recover(
                "A packed data file of the game is damaged (see rc2k.log).",
            )
            .into());
        }
        other => other?,
    };
    Ok(Some(bytes))
}
