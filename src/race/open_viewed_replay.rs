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
//! In:  ebx the replay (`race::replay_path`'s number)
//! Out: carry set when it failed; every register kept (`pusha` / `popa`)
//!
//! Callers: `race::replay_viewer`, `race::viewer_action` (action 0x14),
//! `race::save_last_replay`, 0x444014 (`sub_443FxF`'s replay run).

use crate::errors::Refused;
use crate::files::handle;
use crate::pointers::GamePtr;
use crate::race::replay::REPLAYS;
use crate::race::replay_file_store::REPLAY_FILE;
use crate::race::replay_path::replay_path;
use crate::race::replay_work_store::REPLAY_WORK;
use crate::race::trap::file_read;
use crate::race::viewer_state_store::REPLAY_VIEW;

use rc2k_core::race::viewer_flow::{HEADER_LEN, INDEX_LEN};

/// The Rust version: whether it failed (the carry the original leaves; it keeps the
/// registers and leaves the zero flag `opened_zero(0)`, which no caller reads). `slot` the
/// replay (the original's ebx).
pub fn open_viewed_replay(slot: u32) -> Result<bool, Refused> {
    // Converted: `rc2k_core::race::viewer_flow::{last_race_missing, copied_first,
    // read_good, version_good, index_blocks, opened_marks}`.
    use rc2k_core::race::viewer_flow::{
        Blocks, CAMERA_DWORDS, CAR_HEADER_LEN, copied_first, index_blocks, last_race_missing,
        opened_marks, read_good, version_good,
    };
    let failed = || -> Result<bool, Refused> {
        crate::race::close_viewed_replay::close_viewed_replay()?;
        Ok(true)
    };
    let found = replay_path(slot)?;
    let mut path = found.path;
    if last_race_missing(
        found.last_race,
        || REPLAY_FILE.replay_closed.get(), /* dword_5BEC84: a replay was recorded */
    ) {
        return failed();
    }
    if copied_first(found.number) {
        // A saved replay is copied over the last race's first.
        let copied = crate::files::copy_file::copy_file(
            crate::pointers::host(path) as _,
            crate::pointers::host_at(crate::race::replay_path::LAST_RACE.addr()) as *const u8,
        )?;
        if !copied {
            return failed();
        }
        path = crate::race::replay_path::LAST_RACE.addr() as u32;
    }
    let Some(file) = handle::open_by_mode(crate::pointers::host(path) as _, 0)? else {
        return failed();
    };
    REPLAY_FILE.replay_handle.set(file);
    REPLAY_FILE.replay_open.set(1);
    REPLAY_VIEW.replay_block.set(0);
    REPLAY_VIEW.replay_frame.set(0);
    // A buffer read whole; whether the read is good (`read_good`).
    let read = |buffer: usize, len: u32| -> Result<bool, Refused> {
        let (n, ok) = file_read(
            REPLAY_FILE.replay_handle.get(),
            buffer as u32,
            len,
            "the replay's buffer",
        )?;
        let first = GamePtr::new(buffer as u32).read::<u32>("the replay's first word")?;
        Ok(read_good(
            !ok,
            n,
            len,
            first,
            buffer == REPLAYS.header.addr(),
        ))
    };
    if !read(REPLAYS.header.addr(), HEADER_LEN)? || !version_good(REPLAYS.header.get_u32(8)) {
        return failed();
    }
    // The index is the file's last 0x3850 bytes.
    let at = {
        handle::seek(
            REPLAY_FILE.replay_handle.get(),
            INDEX_LEN.wrapping_neg() as i32,
            2,
        )
    };
    if at.is_none() || !read(REPLAY_FILE.index_magic.addr(), INDEX_LEN)? {
        return failed();
    }
    match index_blocks(REPLAY_FILE.index_block_count.get()) {
        Blocks::Bad => return failed(),
        Blocks::Held(n) => REPLAY_FILE.index_block_count.set(n),
        Blocks::Kept => {}
    }
    let mut block = 0u32;
    loop {
        if { crate::drive::read_replay_frame_0::read_replay_frame_0(block) }? {
            return failed();
        }
        crate::race::viewer::move_dwords(
            crate::race::viewer_buffers_store::VIEWER_BUFFERS
                .block_cameras
                .addr()
                + 8 * block as usize,
            crate::race::viewer_buffers_store::VIEWER_BUFFERS.block[0].addr(),
            CAMERA_DWORDS,
        )?;
        block += 1;
        if block == REPLAY_FILE.index_block_count.get() {
            break;
        }
    }
    let (mark_in, mark_out) = opened_marks(REPLAY_FILE.index_block_count.get());
    crate::race::viewer_buffers_store::VIEWER_BUFFERS
        .mark_in
        .set(mark_in);
    crate::race::viewer_buffers_store::VIEWER_BUFFERS
        .mark_out
        .set(mark_out);
    crate::race::viewer_buffers_store::VIEWER_BUFFERS
        .block_cursor
        .set(crate::race::viewer_buffers_store::VIEWER_BUFFERS.block[0].addr() as u32);
    if { crate::drive::read_replay_frame_0::read_replay_frame_0(0) }? {
        return failed();
    }
    GamePtr::new(crate::race::replay::REPLAYS.header.addr() as u32 + 0x30) // unk_528534
        .copy_to(
            GamePtr::new(REPLAY_WORK.header_stage.addr() as u32),
            CAR_HEADER_LEN,
            "the replay header's stage",
        )?;
    crate::race::replay_patches::apply_replay_patches()?;
    Ok(false)
}
