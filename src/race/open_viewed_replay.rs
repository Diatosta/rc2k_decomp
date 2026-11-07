//! `race::open_viewed_replay` (0x41E9FF): a replay opened for the viewer: its file
//! (`race::replay_path`; the last race's only when one was recorded, `dword_5BEC84`; a
//! saved one, below 100 after the path's, copied over `var\tmprpl` first,
//! `files::copy_file`) opened (mode 0) into `h`, `dword_5BEC88` 1, block and frame 0;
//! its header (0x430 bytes into `dword_528504`: "REPL", version 0x50400) and its index
//! (the last 0x3850 bytes into "RCIB": its name, the block count held to 0x708, more
//! than 0) read; each block read (`drive::read_replay_frame_0`) for its first 8 bytes,
//! the block's cameras (`unk_963BB4`); the marks round the whole; the first block read
//! again, the cars' header (0x400 bytes from `unk_528534`) into `dword_719B14`, and the
//! replay's physics patches in force (`race::apply_replay_patches`). Anything failing:
//! the replay closed (`race::close_viewed_replay`) and carry.
//!
//! Takes: the replay (`race::replay_path`'s number)
//! Out: carry set when it failed
//!
//! Callers: `race::replay_viewer`, `race::viewer_action` (action 0x14),
//! `race::save_last_replay`, 0x444014 (`sub_443FxF`'s replay run).

use crate::errors::Refused;
use crate::files::handle;
use crate::pointers::GamePtr;
use crate::race::owner::Race;
use crate::race::replay_path::replay_path;
use crate::race::replay_work_store::REPLAY_WORK;

use rc2k_core::race::viewer_flow::{HEADER_LEN, INDEX_LEN};

/// The Rust version: whether it failed (the carry the original leaves; it keeps the
/// registers and leaves the zero flag `opened_zero(0)`, which no caller reads). `slot` the
/// replay (the original's ebx).
pub fn open_viewed_replay(race: &Race, slot: u32) -> Result<bool, Refused> {
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
        || shared.replay_file.replay_closed.get(), /* dword_5BEC84: a replay was recorded */
    ) {
        return failed();
    }
    if copied_first(found.number) {
        // A saved replay is copied over the last race's first.
        let copied = crate::files::copy_file::copy_file(
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
    shared.replay_file.replay_handle.set(file);
    shared.replay_file.replay_open.set(1);
    replay.replay_view.replay_block.set(0);
    replay.replay_view.replay_frame.set(0);
    // Whether a buffer's read is good (`read_good`), given its first word after.
    let good = |(n, ok): (u32, bool), len: u32, first: &dyn Fn() -> u32, header: bool| {
        read_good(!ok, n, len, first(), header)
    };
    let mut header = shared.replays.header.read();
    let got = handle::read_slice(shared.replay_file.replay_handle.get(), &mut header);
    shared.replays.header.write(&header);
    if !good(
        got,
        HEADER_LEN,
        &|| race.shared.replays.header.get_u32(0),
        true,
    ) || !version_good(shared.replays.header.get_u32(8))
    {
        return failed();
    }
    // The index is the file's last 0x3850 bytes.
    let at = {
        handle::seek(
            shared.replay_file.replay_handle.get(),
            INDEX_LEN.wrapping_neg() as i32,
            2,
        )
    };
    if at.is_none()
        || !good(
            handle::read_cells(
                shared.replay_file.replay_handle.get(),
                shared.replay_file.index(),
            ),
            INDEX_LEN,
            &|| race.shared.replay_file.index_magic.get(),
            false,
        )
    {
        return failed();
    }
    match index_blocks(shared.replay_file.index_block_count.get()) {
        Blocks::Bad => return failed(),
        Blocks::Held(n) => shared.replay_file.index_block_count.set(n),
        Blocks::Kept => {}
    }
    let mut block = 0u32;
    loop {
        if { crate::drive::read_replay_frame_0::read_replay_frame_0(race, block) }? {
            return failed();
        }
        crate::race::viewer::move_cells(
            &crate::race::viewer_buffers_store::VIEWER_BUFFERS.block_cameras,
            2 * block as usize,
            &crate::race::viewer_buffers_store::VIEWER_BUFFERS.block,
            0,
            CAMERA_DWORDS,
        )?;
        block += 1;
        if block == shared.replay_file.index_block_count.get() {
            break;
        }
    }
    let (mark_in, mark_out) = opened_marks(shared.replay_file.index_block_count.get());
    crate::race::viewer_buffers_store::VIEWER_BUFFERS
        .mark_in
        .set(mark_in);
    crate::race::viewer_buffers_store::VIEWER_BUFFERS
        .mark_out
        .set(mark_out);
    crate::race::viewer_buffers_store::VIEWER_BUFFERS
        .block_used
        .set(0);
    if { crate::drive::read_replay_frame_0::read_replay_frame_0(race, 0) }? {
        return failed();
    }
    // The file's header is bytes (a byte-level copy of its stage part).
    // unk_528534
    let stage = shared
        .replays
        .header
        .read_at(0x30, CAR_HEADER_LEN, "the replay header's stage")?;
    GamePtr::to(&REPLAY_WORK.header_stage)
        .unbounded()
        .write_bytes(&stage, "the replay header's stage")?;
    crate::race::replay_patches::apply_replay_patches(race)?;
    Ok(false)
}
