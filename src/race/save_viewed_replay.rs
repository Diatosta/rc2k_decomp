//! `race::save_viewed_replay` (0x41ED1C): the viewed (and edited) replay saved as replay ebx: with
//! blocks to save, the header made ("REPL", 0, version 0x50400, the date
//! `system::packed_local_time`, the slot's name: 32 bytes of `byte_528934`); the last race's file
//! (`var\tmprpl`) opened to read (mode 0, `dword_975508`) and the slot's (`race::replay_path`) made
//! (mode 1, `dword_97550C`); the undo emptied (`dword_9673F4` 0, its places zeroed: they become the
//! new index); the header written, then each block in the viewer's order: read from its place
//! (`files::seek`, `drive::read_packed` into `dword_9613A4`, neither checked), its first 8 bytes
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

/// Whether it was saved. `slot` the slot to save into.
pub fn save_viewed_replay(race: &Race, slot: u32) -> Result<bool, Refused> {
    let done = save(race, slot)?;
    crate::race::viewer_buffers_store::VIEWER_BUFFERS
        .undo_blocks
        .set(0);
    Ok(done)
}

/// Whether it was saved (`edx` 1).
fn save(race: &Race, slot: u32) -> Result<bool, Refused> {
    let replay = &race.replay;
    let shared = &race.shared;
    // Converted: `rc2k_core::race::viewer_flow::{has_blocks, SAVED_HEADER, name_offset,
    // next_offset}`.
    use rc2k_core::race::viewer_flow::{
        CAMERA_DWORDS, HEADER_LEN, INDEX_HEAD_LEN, NAME_LEN, SAVED_HEADER, has_blocks, name_offset,
        next_offset,
    };
    if !has_blocks(shared.replay_file.index_block_count.get()) {
        return Ok(false);
    }
    let header = &shared.replays.header;
    for (k, word) in SAVED_HEADER.into_iter().enumerate() {
        header.set_u32(4 * k, word);
    }
    header.set_u32(0xC, crate::system::packed_local_time::packed_local_time());
    let name =
        shared
            .replays
            .names
            .read_at(name_offset(slot) as usize, NAME_LEN, "a replay's name")?;
    header.write_at(0x10, &name, "a replay's name")?;
    // The last race's file opened to read from, the slot's to write to.
    let Some(from) = handle::open_by_mode(&last_race(replay), 0)? else {
        return Ok(false);
    };
    crate::race::viewer_buffers_store::VIEWER_BUFFERS
        .from_file
        .set(from);
    let path = replay_path(race, slot)?.path;
    let Some(to) = handle::open_by_mode(&path, 1)? else {
        handle::close(
            crate::race::viewer_buffers_store::VIEWER_BUFFERS
                .from_file
                .get(),
        );
        return Ok(false);
    };
    crate::race::viewer_buffers_store::VIEWER_BUFFERS
        .to_file
        .set(to);
    crate::race::viewer_buffers_store::VIEWER_BUFFERS
        .undo_blocks
        .set(0);
    for place in &crate::race::viewer_buffers_store::VIEWER_BUFFERS.undo_places[..TABLE_LEN / 4] {
        place.set(0);
    }
    // Both files closed on the way out (the new one only when it was made).
    let close = || {
        handle::close(
            crate::race::viewer_buffers_store::VIEWER_BUFFERS
                .to_file
                .get(),
        );
        handle::close(
            crate::race::viewer_buffers_store::VIEWER_BUFFERS
                .from_file
                .get(),
        );
    };
    replay.replay_view.file_offset.set(HEADER_LEN); // the new file's length so far
    if !handle::write_slice(
        crate::race::viewer_buffers_store::VIEWER_BUFFERS
            .to_file
            .get(),
        &shared.replays.header.read(),
    )
    .1
    {
        close();
        return Ok(false);
    }
    let mut block = 0u32;
    loop {
        handle::seek(
            crate::race::viewer_buffers_store::VIEWER_BUFFERS
                .from_file
                .get(),
            crate::race::replay::block_place(race, 2 * block as usize)? as i32,
            0,
        );
        crate::drive::read_packed::read_packed(
            crate::race::viewer_buffers_store::VIEWER_BUFFERS
                .from_file
                .get(),
            crate::race::replay::block_place(race, 2 * block as usize + 1)?,
        )?;
        move_cells(
            &crate::race::viewer_buffers_store::VIEWER_BUFFERS.block,
            0,
            &crate::race::viewer_buffers_store::VIEWER_BUFFERS.block_cameras,
            2 * block as usize,
            CAMERA_DWORDS,
        )?;
        crate::race::checkpoints_store::cell_in(
            &crate::race::viewer_buffers_store::VIEWER_BUFFERS.undo_places,
            2 * block as usize,
        )
        .set(replay.replay_view.file_offset.get());
        let (written, failed) = crate::race::replay::write_replay_chunk(
            &race.shared,
            crate::race::viewer_buffers_store::VIEWER_BUFFERS
                .to_file
                .get(),
        )?;
        if failed {
            close();
            return Ok(false);
        }
        crate::race::checkpoints_store::cell_in(
            &crate::race::viewer_buffers_store::VIEWER_BUFFERS.undo_places,
            2 * block as usize + 1,
        )
        .set(written);
        replay
            .replay_view
            .file_offset
            .set(next_offset(replay.replay_view.file_offset.get(), written));
        block += 1;
        if block == shared.replay_file.index_block_count.get() {
            break;
        }
    }
    let done = handle::write_cells(
        crate::race::viewer_buffers_store::VIEWER_BUFFERS
            .to_file
            .get(),
        &shared.replay_file.index()[..INDEX_HEAD_LEN as usize / 4],
    )
    .1 && handle::write_cells(
        crate::race::viewer_buffers_store::VIEWER_BUFFERS
            .to_file
            .get(),
        &crate::race::viewer_buffers_store::VIEWER_BUFFERS.undo_places[..TABLE_LEN / 4],
    )
    .1;
    close();
    Ok(done)
}
