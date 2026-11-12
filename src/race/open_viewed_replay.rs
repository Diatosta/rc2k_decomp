//! `race::open_viewed_replay` (0x41E9FF): a replay opened for the viewer: its file
//! (`race::replay_path`; the last race's only when one was recorded, `replay_closed`; a saved one,
//! below 100 after the path's, copied over `var\tmprpl` first, `files::copy_file`) opened (mode 0)
//! into `h`, `replay_open` 1, block and frame 0; its header (0x430 bytes into `Replays.header`:
//! "REPL", version 0x50400) and its index (the last 0x3850 bytes into "RCIB": its name, the block
//! count held to 0x708, more than 0) read; each block read (`drive::read_replay_frame_0`) for its
//! first 8 bytes, the block's cameras (`block_cameras`); the marks round the whole; the first block
//! read again, the cars' header (0x400 bytes from `Replays.header`) into `header_stage`, and the
//! replay's physics patches in force (`race::apply_replay_patches`). Anything failing: the replay
//! closed (`race::close_viewed_replay`) and carry.
//!
//! Takes: the replay (`race::replay_path`'s number)
//!
//! Callers: `race::replay_viewer`, `race::viewer_action` (action 0x14),
//! `race::save_last_replay`, 0x444014 (`sub_443FxF`'s replay run).

use crate::errors::Refused;
use crate::files::handle;
use crate::race::owner::Race;
use crate::race::replay_path::replay_path;

use rc2k_core::race::viewer_flow::{HEADER_LEN, INDEX_LEN};

/// Whether it failed. `slot` the replay.
pub fn open_viewed_replay(
    race: &Race,
    system: &crate::system::owner::System,
    slot: u32,
) -> Result<bool, Refused> {
    let replay = &race.replay;
    let shared = &race.shared;
    // Converted: `rc2k_core::race::viewer_flow::{last_race_missing, copied_first,
    // read_good, version_good, index_blocks, opened_marks}`.
    use rc2k_core::race::viewer_flow::{
        Blocks, CAMERA_DWORDS, CAR_HEADER_LEN, copied_first, index_blocks, last_race_missing,
        opened_marks, read_good, version_good,
    };
    let failed = || -> Result<bool, Refused> {
        crate::race::close_viewed_replay::close_viewed_replay(race)?;
        Ok(true)
    };
    let found = replay_path(race, slot)?;
    let mut path = found.path;
    if last_race_missing(
        found.last_race,
        || shared.replay_file().replay_closed.get(), /* dword_5BEC84: a replay was recorded */
    ) {
        return failed();
    }
    if copied_first(found.number) {
        // A saved replay is copied over the last race's first.
        let copied = crate::files::copy_file::copy_file(
            system,
            &path,
            &crate::race::replay_path::last_race(replay),
        )?;
        if !copied {
            return failed();
        }
        path = crate::race::replay_path::last_race(replay);
    }
    let Some(file) = handle::open_by_mode(&path, 0)? else {
        return failed();
    };
    shared.replay_file().replay_handle.set(file);
    shared.replay_file().replay_open.set(true);
    replay.replay_view().replay_block.set(0);
    replay.replay_view().replay_frame.set(0);
    // Whether a buffer's read is good (`read_good`), given its first word after.
    let good = |(n, ok): (u32, bool), len: u32, first: &dyn Fn() -> u32, header: bool| {
        read_good(!ok, n, len, first(), header)
    };
    let mut header = shared.replays().header.read();
    let got = handle::read_slice(shared.replay_file().replay_handle.get(), &mut header);
    shared.replays().header.write(&header);
    if !good(
        got,
        HEADER_LEN,
        &|| race.shared.replays().header.get_u32(0),
        true,
    ) || !version_good(shared.replays().header.get_u32(8))
    {
        return failed();
    }
    // The index is the file's last 0x3850 bytes.
    let at = {
        handle::seek(
            shared.replay_file().replay_handle.get(),
            INDEX_LEN.wrapping_neg() as i32,
            2,
        )
    };
    if at.is_none()
        || !good(
            handle::read_cells(
                shared.replay_file().replay_handle.get(),
                shared.replay_file().index(),
            ),
            INDEX_LEN,
            &|| race.shared.replay_file().index_magic.get(),
            false,
        )
    {
        return failed();
    }
    match index_blocks(shared.replay_file().index_block_count.get()) {
        Blocks::Bad => return failed(),
        Blocks::Held(n) => shared.replay_file().index_block_count.set(n),
        Blocks::Kept => {}
    }
    let mut block = 0u32;
    loop {
        if { crate::drive::read_replay_frame_0::read_replay_frame_0(race, system, block) }? {
            return failed();
        }
        crate::race::viewer::move_cells(
            &race.viewer_buffers().block_cameras,
            2 * block as usize,
            &race.viewer_buffers().block,
            0,
            CAMERA_DWORDS,
        )?;
        block += 1;
        if block == shared.replay_file().index_block_count.get() {
            break;
        }
    }
    let (mark_in, mark_out) = opened_marks(shared.replay_file().index_block_count.get());
    race.viewer_buffers().mark_in.set(mark_in);
    race.viewer_buffers().mark_out.set(mark_out);
    race.viewer_buffers().block_used.set(0);
    if { crate::drive::read_replay_frame_0::read_replay_frame_0(race, system, 0) }? {
        return failed();
    }
    // The file's header is bytes (a byte-level copy of its stage part).
    // Replays.header
    let stage =
        shared
            .replays()
            .header
            .read_at(0x30, CAR_HEADER_LEN, "the replay header's stage")?;
    put_header_stage(race, &stage);
    crate::race::replay_patches::apply_replay_patches(race)?;
    Ok(false)
}

/// The cars' header (`CAR_HEADER_LEN` bytes) into the replay work's fields from `header_stage` on:
/// the header's settings, the sealed part, the patch switches and the unread rest, a dword each
/// with no gap, up to the camera wishes.
fn put_header_stage(race: &Race, header: &[u8]) {
    let w = race.replay_work();
    let singles = [
        &w.header_stage,
        &w.header_mode,
        &w.header_players,
        &w.header_cars,
        &w.header_kind_0,
        &w.header_kind_1,
        &w.header_kind_2,
        &w.header_kind_3,
        &w.header_one_player,
        &w.header_codriver_sex_0,
        &w.header_codriver_sex_1,
        &w.header_start_place,
        &w.header_arcade_cars,
        &w.header_seed,
        &w.header_game_over_0,
        &w.header_game_over_1,
    ];
    let cells = singles
        .into_iter()
        .chain(&w.sealed_header)
        .chain(std::iter::once(&w.patches_in_header))
        .chain(&w.header_patches)
        .chain(&w.unknown_7d0);
    for (dword, bytes) in cells.zip(header.as_chunks::<4>().0) {
        dword.set(u32::from_le_bytes(*bytes));
    }
}
