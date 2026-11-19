//! Reads bytes from a file into a buffer: a count of 0 or more is a plain read (`files::read`);
//! below 0, minus the count of packed bytes are read and unpacked into the buffer
//! (`drive::unpack_rle`), returning the bytes unpacked. What cannot be read or unpacked is a
//! failed read (nothing written then but the packed bytes).

use crate::errors::{BadInput, Refused};
use crate::files::handle::read_slice;
use crate::race::owner::Race;
use crate::system::owner::System;

/// Into the replay block. A plain read of more than the block holds, a packed one past the
/// scratch and a packing that does not unpack into the block are a damaged replay file (cut
/// short, or written over): the read fails (`true` in the second place) and nothing more is
/// done, so a damaged recording never ends the program.
pub fn read_packed(
    race: &Race,
    system: &System,
    file: u32,
    count: u32,
) -> Result<(u32, bool, Option<bool>), Refused> {
    match read(race, system, file, count) {
        Err(Refused::BadInput(why)) => {
            crate::program_log::log(&format!(
                "replay: a block of {count:#x} bytes cannot be read ({why:?}); the file is damaged"
            ));
            Ok((0, true, None))
        }
        other => other,
    }
}

fn read(
    race: &Race,
    system: &System,
    file: u32,
    count: u32,
) -> Result<(u32, bool, Option<bool>), Refused> {
    let work = &system.work();
    let Some(packed) = rc2k_core::drive::race_steps::packed_bytes(count) else {
        if count as usize > race.viewer_buffers().block_len() {
            return Err(BadInput("a replay block read past the block").into());
        }
        let mut bytes = vec![0u8; count as usize];
        let (n, ok) = read_slice(file, &mut bytes);
        race.viewer_buffers()
            .put_in_block(0, &bytes[..n as usize], "a replay block read")?;
        return Ok((n, !ok, None));
    };
    let (n, ok) = work
        .files
        .read_from(file, packed, "a packed replay block")?;
    if !ok {
        return Ok((n, true, None));
    }
    let out =
        crate::drive::unpack_rle::unpack_rle(race.viewer_buffers(), &work.files.buffer.read())?;
    Ok((out, false, Some(out == 0)))
}
