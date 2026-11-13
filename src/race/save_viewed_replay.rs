//! `race::save_viewed_replay` (0x41ED1C): the viewed (and edited) replay saved as a replay slot: with
//! blocks to save, the header made ("REPL", 0, version 0x50400, the date
//! `system::packed_local_time`, the slot's name: 32 bytes of the slot-name table); the last race's file
//! (`var\tmprpl`) opened to read (mode 0, `from_file`) and the slot's (`race::replay_path`) made
//! (mode 1, `to_file`); the undo emptied (`undo_blocks` 0, its places zeroed: they become the
//! new index); the header written, then each block in the viewer's order: read from its place
//! (`files::seek`, `drive::read_packed` into `block`, neither checked), its first 8 bytes
//! the block's cameras, written packed (`race::write_replay_chunk`), its offset and size (negative
//! when packed) into the new index; then "RCIB" and the count (16 bytes) and the new index (0x3840
//! bytes). Both closed; the undo emptied again.
//!
//! Takes: the slot (`race::replay_path`'s number)
//!
//! Callers: `race::viewer_action` (0x15 back into a saved slot, 0x16 into a chosen one),
//! `race::save_last_replay`. Saving into slot 222 would make the file it reads from anew (the
//! original's way, kept).

use crate::errors::Refused;
use crate::files::handle;
use crate::race::owner::Race;
use crate::race::replay_path::{last_race, replay_path};
use crate::race::viewer::{TABLE_LEN, move_cells};
use crate::race::viewer_buffers_store::ViewerBuffers;
use crate::system::owner::System;

/// Whether it was saved. `slot` the slot to save into.
pub fn save_viewed_replay(race: &Race, system: &System, slot: u32) -> Result<bool, Refused> {
    let done = save(race, system, slot)?;
    race.viewer_buffers().undo_blocks.set(0);
    Ok(done)
}

/// Whether it was saved.
fn save(race: &Race, system: &System, slot: u32) -> Result<bool, Refused> {
    let replay = &race.replay;
    let shared = &race.shared;
    // Converted: `rc2k_core::race::viewer_flow::{has_blocks, SAVED_HEADER, name_offset,
    // next_offset}`.
    use rc2k_core::race::viewer_flow::{
        CAMERA_DWORDS, HEADER_LEN, INDEX_HEAD_LEN, NAME_LEN, SAVED_HEADER, has_blocks, name_offset,
        next_offset,
    };
    if !has_blocks(shared.replay_file().index.block_count.get()) {
        return Ok(false);
    }
    let header = &shared.replays().header;
    for (k, word) in SAVED_HEADER.into_iter().enumerate() {
        header.set_u32(4 * k, word);
    }
    header.set_u32(0xC, crate::system::packed_local_time::packed_local_time());
    let name =
        shared
            .replays()
            .names
            .read_at(name_offset(slot) as usize, NAME_LEN, "a replay's name")?;
    header.write_at(0x10, &name, "a replay's name")?;
    // The last race's file opened to read from, the slot's to write to.
    let Some(from) = handle::open_by_mode(&last_race(replay), 0)? else {
        return Ok(false);
    };
    race.viewer_buffers().from_file.set(from);
    let path = replay_path(race, slot)?.path;
    let Some(to) = handle::open_by_mode(&path, 1)? else {
        handle::close(race.viewer_buffers().from_file.get());
        return Ok(false);
    };
    race.viewer_buffers().to_file.set(to);
    race.viewer_buffers().undo_blocks.set(0);
    for place in &race.viewer_buffers().undo_places[..TABLE_LEN / 4] {
        place.set(0);
    }
    // Both files closed on the way out (the new one only when it was made).
    let close = || {
        handle::close(race.viewer_buffers().to_file.get());
        handle::close(race.viewer_buffers().from_file.get());
    };
    replay.replay_view().file_offset.set(HEADER_LEN); // the new file's length so far
    if !handle::write_slice(
        race.viewer_buffers().to_file.get(),
        &shared.replays().header.read(),
    )
    .1
    {
        close();
        return Ok(false);
    }
    let mut block = 0u32;
    loop {
        handle::seek(
            race.viewer_buffers().from_file.get(),
            crate::race::replay::block_place(race, 2 * block as usize)? as i32,
            0,
        );
        crate::drive::read_packed::read_packed(
            race,
            system,
            race.viewer_buffers().from_file.get(),
            crate::race::replay::block_place(race, 2 * block as usize + 1)?,
        )?;
        move_cells(
            &race.viewer_buffers().block,
            0,
            &race.viewer_buffers().block_cameras,
            2 * block as usize,
            CAMERA_DWORDS,
        )?;
        undo_place(race.viewer_buffers(), 2 * block as usize)?
            .set(replay.replay_view().file_offset.get());
        let (written, failed) = crate::race::replay::write_replay_chunk(
            race,
            system,
            race.viewer_buffers().to_file.get(),
        )?;
        if failed {
            close();
            return Ok(false);
        }
        undo_place(race.viewer_buffers(), 2 * block as usize + 1)?.set(written);
        replay
            .replay_view()
            .file_offset
            .set(next_offset(replay.replay_view().file_offset.get(), written));
        block += 1;
        if block == shared.replay_file().index.block_count.get() {
            break;
        }
    }
    let done = handle::write_cells(
        race.viewer_buffers().to_file.get(),
        &shared.replay_file().index.dwords()[..INDEX_HEAD_LEN as usize / 4],
    )
    .1 && handle::write_cells(
        race.viewer_buffers().to_file.get(),
        &race.viewer_buffers().undo_places[..TABLE_LEN / 4],
    )
    .1;
    close();
    Ok(done)
}

/// The undo's block place `n` (`g_undoPlaces`, 0x3840 bytes). The viewer holds a replay's
/// block count to 0x708 (`open_viewed_replay` clamps what the file says, the recorder stops at
/// the full index, a paste is refused past it, a cut only lowers it), so `n`, 2 a block and one
/// more, stays inside the 0xE10 places.
fn undo_place(
    buffers: &ViewerBuffers,
    n: usize,
) -> Result<&crate::session::PlainDword, crate::errors::BadInput> {
    buffers
        .undo_places
        .get(n)
        .ok_or(crate::errors::BadInput("an undo place past the table"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every place the viewer's block counts (up to 0x708 blocks, two a block) ask for is a
    /// cell of the table, and the first past the 0xE10 places is refused.
    #[test]
    fn places_cover_the_blocks() {
        let buffers = crate::race::owner::race().viewer_buffers();
        for n in 0..2 * rc2k_core::race::replay::MOST_BLOCKS as usize {
            let new = undo_place(buffers, n).ok().map(std::ptr::from_ref);
            assert_eq!(
                new,
                buffers.undo_places.get(n).map(std::ptr::from_ref),
                "{n}"
            );
        }
        assert!(undo_place(buffers, 0xE10).is_err());
    }
}
