//! Reads a file's start into a buffer and says how much it got: `files::open_by_mode` with mode
//! 0, `files::read`, `files::close`. Fails if the file could not be opened, read or closed. Used
//! for the models and the stage's files. Refuses what `files::resolve_path` refuses.

use crate::errors::BadInput;

/// `read_whole_file` into `cells` (a typed store's dwords, their bytes in order: `4 *
/// cells.len()` bytes at most; what the file does not reach is kept). Gives the bytes read (0
/// when the file could not be opened or closed).
pub fn read_whole_file_cells<C: crate::session::DwordLike>(
    path: &[u8],
    cells: &[C],
) -> Result<u32, BadInput> {
    let mut into: Vec<u8> = cells.iter().flat_map(|c| c.get().to_le_bytes()).collect();
    let (_, bytes_read) = read_whole_file_counted(path, &mut into)?;
    for (c, b) in cells.iter().zip(into.as_chunks::<4>().0) {
        c.set(u32::from_le_bytes(*b));
    }
    Ok(bytes_read)
}

/// `read_whole_file` into a plain buffer (all of its bytes at most; what the file does not
/// reach is kept). Gives the bytes read (0 when the file could not be opened or closed).
pub fn read_whole_file_plain_bytes<const N: usize>(
    path: &[u8],
    bytes: &crate::session::PlainBytes<N>,
) -> Result<u32, BadInput> {
    let mut into = bytes.read();
    let (_, bytes_read) = read_whole_file_counted(path, &mut into)?;
    bytes.write(&into);
    Ok(bytes_read)
}

/// `read_whole_file` into Rust's memory (`into.len()` bytes at most).
pub fn read_whole_file_into(path: &[u8], into: &mut [u8]) -> Result<(), BadInput> {
    read_whole_file_len(path, into).map(|_| ())
}

/// `read_whole_file_into`, giving how many bytes of `into` the file replaced (0 when it could
/// not be opened or read).
pub fn read_whole_file_len(path: &[u8], into: &mut [u8]) -> Result<usize, BadInput> {
    read_whole_file_counted(path, into).map(|(len, _)| len)
}

/// `read_whole_file_len` and the bytes counted as read (0 when the file could not be opened or
/// closed).
fn read_whole_file_counted(path: &[u8], into: &mut [u8]) -> Result<(usize, u32), BadInput> {
    let w = rc2k_core::files::io::read_whole_file(
        &mut crate::files::handle::GameFiles::new(),
        path,
        into,
    )?;
    if !w.opened {
        return Ok((0, 0));
    }
    Ok((
        if w.read_ok { w.bytes as usize } else { 0 },
        if w.closed { w.bytes } else { 0 },
    ))
}
