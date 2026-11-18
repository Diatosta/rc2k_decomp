//! `race::save_viewed_replay`: the viewed (and edited) replay saved as a replay slot: with blocks
//! to save, the header made ("REPL", 0, version 0x50400 (0x50401 for a replay with the extra tape), the date `system::packed_local_time`, the
//! slot's name: 32 bytes of the slot-name table); the last race's file (`var\tmprpl`) opened to
//! read (mode 0, `from_file`) and the slot's (`race::replay_path`) made (mode 1, `to_file`); the
//! undo emptied (`undo_blocks` 0, its places zeroed: they become the new index); the header
//! written, then each block in the viewer's order: read from its place (`files::seek`,
//! `drive::read_packed` into `block`, neither checked), its first 8 bytes the block's cameras,
//! written packed (`race::write_replay_chunk`), its offset and size (negative when packed) into the
//! new index; then "RCIB" and the count (16 bytes) and the new index (0x3840 bytes). Both closed;
//! the undo emptied again.
//!
//! Takes the slot (`race::replay_path`'s number).
//!
//! Saving into slot 222 would make the file it reads from anew.

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
    // The replays list tags the slot by what it holds now.
    if done && (slot as usize) < crate::race::replay::SLOTS {
        let kind = if race.viewer_buffers().v2.is_set() {
            crate::race::replay::SLOT_NEW
        } else {
            crate::race::replay::SLOT_OLD
        };
        race.shared.replays().kinds.set(slot as usize, kind);
    }
    Ok(done)
}

/// The viewed replay, as it is now (edits and all), written to the file at `path` in the format
/// it is viewed in, for a program that renders it; whether it was written. Nothing of the
/// viewer changes: the undo's table, which the writing uses as scratch, is put back.
pub fn save_to_file(race: &Race, system: &System, path: &[u8]) -> Result<bool, Refused> {
    let buffers = race.viewer_buffers();
    let places: Vec<u32> = buffers
        .undo_places
        .iter()
        .map(crate::memory::session::PlainDword::get)
        .collect();
    let undo_blocks = buffers.undo_blocks.get();
    let format = if buffers.v2.is_set() {
        Format::Own
    } else {
        Format::Old {
            extra_tape: buffers.extra_tape.is_set(),
        }
    };
    let saved = save_file(race, system, 0, path, format);
    for (place, value) in buffers.undo_places.iter().zip(places) {
        place.set(value);
    }
    buffers.undo_blocks.set(undo_blocks);
    saved
}

/// The format a file is saved in.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Format {
    /// The own format (`rc2k_core::race::replay_v2`): the replay as it is viewed.
    Own,
    /// The original's: the block's first bytes; `extra_tape` for the version that has the tape.
    Old { extra_tape: bool },
}

/// Whether it was saved. A replay of the own format is saved in it and, when the original's can
/// hold the race (`Extension::old_format_holds`), also in the original's beside it, each in the
/// slot's file of its kind (`race::replay_path`); the other kind of file the slot may have is
/// taken away, so that the slot holds what was saved. A replay of the original's format is saved
/// as it was.
fn save(race: &Race, system: &System, slot: u32) -> Result<bool, Refused> {
    use crate::race::replay_path::{own_format, remove};
    let path = replay_path(race, slot)?.path;
    if !race.viewer_buffers().v2.is_set() {
        let extra_tape = race.viewer_buffers().extra_tape.is_set();
        let saved = save_file(race, system, slot, &path, Format::Old { extra_tape })?;
        if saved {
            remove(&own_format(&path));
        }
        return Ok(saved);
    }
    let holds = race
        .replay
        .v2()
        .lock()
        .ext
        .as_ref()
        .is_some_and(rc2k_core::race::replay_v2::Extension::old_format_holds);
    if !save_file(race, system, slot, &own_format(&path), Format::Own)? {
        return Ok(false);
    }
    if holds {
        save_file(race, system, slot, &path, Format::Old { extra_tape: false })?;
    } else {
        remove(&path);
    }
    Ok(true)
}

/// Whether it was saved, to the file at `path` in `format`.
fn save_file(
    race: &Race,
    system: &System,
    slot: u32,
    path: &[u8],
    format: Format,
) -> Result<bool, Refused> {
    let replay = &race.replay;
    let shared = &race.shared;
    // Logic in `rc2k_core::race::viewer_flow::{has_blocks, name_offset, next_offset}`.
    use rc2k_core::race::replay_v2::{EXT_LEN_AT, MAGIC, VERSION};
    use rc2k_core::race::viewer_flow::{
        CAMERA_DWORDS, HEADER_LEN, INDEX_HEAD_LEN, NAME_LEN, has_blocks, name_offset, next_offset,
    };
    if !has_blocks(shared.replay_file().index.block_count.get()) {
        return Ok(false);
    }
    let header = &shared.replays().header;
    // The saved file has the version of the blocks it is made of.
    let words = match format {
        Format::Own => [MAGIC, 0, VERSION],
        Format::Old { extra_tape } => rc2k_core::race::viewer_flow::saved_header(extra_tape),
    };
    for (k, word) in words.into_iter().enumerate() {
        header.set_u32(4 * k, word);
    }
    let extension = match format {
        Format::Own => replay
            .v2()
            .lock()
            .ext
            .as_ref()
            .map(rc2k_core::race::replay_v2::Extension::to_bytes)
            .unwrap_or_default(),
        Format::Old { .. } => Vec::new(),
    };
    header.set_u32(EXT_LEN_AT, extension.len() as u32);
    header.set_u32(0xC, crate::system::packed_local_time::packed_local_time());
    let name =
        shared
            .replays()
            .names
            .read_at(name_offset(slot) as usize, NAME_LEN, "a replay's name")?;
    header.write_at(0x10, &name, "a replay's name")?;
    // The last race's file opened to read from, the file to write to.
    let Some(from) = handle::open_by_mode(&last_race(replay), 0)? else {
        return Ok(false);
    };
    race.viewer_buffers().from_file.set(from);
    let Some(to) = handle::open_by_mode(path, 1)? else {
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
    // the new file's length so far
    replay
        .replay_view()
        .file_offset
        .set(HEADER_LEN + extension.len() as u32);
    let mut head = header.read().to_vec();
    head.extend_from_slice(&extension);
    if !handle::write_slice(race.viewer_buffers().to_file.get(), &head).1 {
        close();
        return Ok(false);
    }
    // The original's index holds 1800 blocks: a longer replay is written cut there, as the
    // original's recording would have ended; the own format's holds `MOST_BLOCKS`.
    let (most, table_dwords) = match format {
        Format::Own => (rc2k_core::race::replay_v2::MOST_BLOCKS, TABLE_LEN / 4),
        Format::Old { .. } => (rc2k_core::race::replay::MOST_BLOCKS, 0x3840 / 4),
    };
    let blocks = shared.replay_file().index.block_count.get().min(most);
    let mut block = 0u32;
    loop {
        handle::seek(
            race.viewer_buffers().from_file.get(),
            crate::race::replay::block_place(race, 2 * block as usize)? as i32,
            0,
        );
        // A block that cannot be read or unpacked (a recording cut short, or written over by
        // another copy of the game on the same folder) stops the saving: nothing here ends the game.
        let read = crate::drive::read_packed::read_packed(
            race,
            system,
            race.viewer_buffers().from_file.get(),
            crate::race::replay::block_place(race, 2 * block as usize + 1)?,
        );
        if !matches!(read, Ok((_, false, _))) {
            close();
            return Ok(false);
        }
        move_cells(
            &race.viewer_buffers().block,
            0,
            &race.viewer_buffers().block_cameras,
            2 * block as usize,
            CAMERA_DWORDS,
        )?;
        undo_place(race.viewer_buffers(), 2 * block as usize)?
            .set(replay.replay_view().file_offset.get());
        let bytes = race.viewer_buffers().block_bytes();
        let kept = match format {
            Format::Own => &bytes[..],
            // The original's block: its frame, and the extra tape when its version has it.
            Format::Old { extra_tape } => {
                let len = rc2k_core::drive::replay_frames::block_bytes(extra_tape);
                &bytes[..len.min(bytes.len())]
            }
        };
        let (written, failed) =
            crate::race::replay::write_chunk_of(race, race.viewer_buffers().to_file.get(), kept)?;
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
        if block == blocks {
            break;
        }
    }
    let index = &shared.replay_file().index;
    let steps = if blocks < index.block_count.get() {
        100 * blocks
    } else {
        index.frame_count.get()
    };
    let head: Vec<u8> = [index.magic.get(), index.seal.get(), blocks, steps]
        .iter()
        .flat_map(|w| w.to_le_bytes())
        .collect();
    debug_assert_eq!(head.len(), INDEX_HEAD_LEN as usize);
    let done = handle::write_slice(race.viewer_buffers().to_file.get(), &head).1
        && handle::write_cells(
            race.viewer_buffers().to_file.get(),
            &race.viewer_buffers().undo_places[..table_dwords],
        )
        .1;
    close();
    Ok(done)
}

/// The undo's block place `n` (`undo_places`, 0x3840 bytes). The viewer holds a replay's
/// block count to 0x708 (`open_viewed_replay` clamps what the file says, the recorder stops at
/// the full index, a paste is refused past it, a cut only lowers it), so `n`, 2 a block and one
/// more, stays inside the 0xE10 places.
fn undo_place(
    buffers: &ViewerBuffers,
    n: usize,
) -> Result<&crate::memory::session::PlainDword, crate::errors::BadInput> {
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
        let buffers = crate::test_support::own::race().viewer_buffers();
        for n in 0..2 * rc2k_core::race::replay_v2::MOST_BLOCKS as usize {
            let new = undo_place(buffers, n).ok().map(std::ptr::from_ref);
            assert_eq!(
                new,
                buffers.undo_places.get(n).map(std::ptr::from_ref),
                "{n}"
            );
        }
        assert!(
            undo_place(
                buffers,
                2 * rc2k_core::race::replay_v2::MOST_BLOCKS as usize
            )
            .is_err()
        );
    }
}
