//! `files::read_whole_file` (0x421B54): reads a file's start into a buffer and says how
//! much it got.
//!
//! Takes the path, the buffer and how many bytes at most. Sets the last-read count to the bytes read
//! (0 after a failed open or close). Fails if the file could not be opened, read or closed; the
//! original also leaves what `CloseHandle` gave (-1 if the file did not open) and 1 after a
//! good read, 0 after a failed one, in registers.
//!
//! `files::open_by_mode` with mode 0, `files::read`, `files::close`. 29 callers: the models, the
//! stage's files. Refuses what `files::resolve_path` refuses.

use crate::errors::BadInput;
use crate::files::state::BYTES_READ;
use crate::pointers::GamePtr;

/// Reads the whole file at `path` into `buffer` (`count` bytes at most). The bytes read
/// go to `BYTES_READ` (0 after any failure but a failed read's own count).
pub fn read_whole_file(path: &[u8], to: GamePtr, count: u32) -> Result<(), BadInput> {
    let mut into = to.read_bytes(count as usize, "a file's buffer")?;
    read_whole_file_into(path, &mut into)?;
    to.write_bytes(&into, "a file's buffer")
}

/// `read_whole_file` into `cells` (a typed store's dwords, their bytes in order: `4 *
/// cells.len()` bytes at most; what the file does not reach is kept).
pub fn read_whole_file_cells(path: &[u8], cells: &[crate::session::Dword]) -> Result<(), BadInput> {
    let mut into: Vec<u8> = cells.iter().flat_map(|c| c.get().to_le_bytes()).collect();
    read_whole_file_into(path, &mut into)?;
    for (c, b) in cells.iter().zip(into.as_chunks::<4>().0) {
        c.set(u32::from_le_bytes(*b));
    }
    Ok(())
}

/// `read_whole_file` into `bytes` (all of them at most; what the file does not reach is
/// kept).
pub fn read_whole_file_bytes<const N: usize>(
    path: &[u8],
    bytes: &crate::session::Bytes<N>,
) -> Result<(), BadInput> {
    let mut into = bytes.read();
    read_whole_file_into(path, &mut into)?;
    bytes.write(&into);
    Ok(())
}

/// `read_whole_file` into Rust's memory (`into.len()` bytes at most).
pub fn read_whole_file_into(path: &[u8], into: &mut [u8]) -> Result<(), BadInput> {
    // Converted: `rc2k_core::files::io::read_whole_file`.
    {
        let w = rc2k_core::files::io::read_whole_file(
            &mut crate::files::handle::GameFiles::new(),
            path,
            into,
        )?;
        if !w.opened {
            BYTES_READ.set(0);
            return Ok(());
        }
        BYTES_READ.set(if w.closed { w.bytes } else { 0 });
        Ok(())
    }
}
