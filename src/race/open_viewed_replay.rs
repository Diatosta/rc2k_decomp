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
//! Callers: `race::replay_viewer`, `race::viewer_action` (action 0x14), `race::save_last_replay`,
//! 0x444014 (`sub_443FxF`'s replay run). Its calls through `part`: 1 `files::copy_file`
//! (eax from, ebx to), 2 `files::open_by_mode` (eax path, ebx mode -> eax handle, carry),
//! 3 `files::read` (eax handle, ebx buffer, ecx bytes -> eax bytes read, carry), 4
//! `files::seek` (eax handle, ecx distance, ebx method -> carry), 5
//! `drive::read_replay_frame_0` (eax block -> carry), 6 `race::close_viewed_replay`, 7
//! `race::apply_replay_patches`; in the game their Rust versions (`replay_path` is called
//! as it is). Shadow mode runs the original alone (a saved replay is copied over the
//! last race's). Self-test: a copy whose calls go to stand-ins that play a file system
//! (headers and indexes good and bad, reads failing at each place), against the Rust
//! version; the variables, the calls (eax, ebx, ecx, edx) and carry compared.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::flow::a;
use crate::race::replay_path::{LAST_RACE_PATH, replay_path};
use crate::race::viewer::{
    BLOCK, BLOCK_CURSOR, BLOCK_DATA, BLOCKS, CAMERAS, CAR_HEADER, FRAME, HANDLE, HEADER, INDEX,
    MARK_IN, MARK_OUT, OPEN, Part, get, put, regs_entry,
};

const RECORDED: usize = 0x5BEC84; // dword_5BEC84: a replay was recorded
const CARS_FROM: usize = 0x528534; // unk_528534
use rc2k_core::race::replay::HEADER_VERSION as VERSION;
use rc2k_core::race::viewer_flow::{HEADER_LEN, INDEX_LEN};

/// The Rust version: the registers and flags it leaves.
pub fn open_viewed_replay(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::race::viewer_flow::{last_race_missing, copied_first,
    // read_good, version_good, index_blocks, opened_marks, opened_zero}`.
    use rc2k_core::race::viewer_flow::{
        Blocks, CAMERA_DWORDS, CAR_HEADER_LEN, copied_first, index_blocks, last_race_missing,
        opened_marks, opened_zero, read_good, version_good,
    };
    let mut out = *entry;
    let failed = |part: Part, out: &mut Regs, r: &Regs| -> Result<Regs, Refused> {
        let mut s = *r;
        part(6, &mut s)?;
        out.set_carry(true);
        Ok(*out)
    };
    let mut r = replay_path(entry)?;
    let is_last_race = r.eflags & crate::framework::ZERO_FLAG != 0;
    if last_race_missing(is_last_race, || get(RECORDED)) {
        return failed(part, &mut out, &r);
    }
    if copied_first(r.ebx) {
        r.ebx = a(LAST_RACE_PATH);
        part(1, &mut r)?;
        if r.carry() {
            return failed(part, &mut out, &r);
        }
        r.eax = a(LAST_RACE_PATH);
    }
    r.ebx = 0;
    part(2, &mut r)?;
    if r.carry() {
        return failed(part, &mut out, &r);
    }
    put(HANDLE, r.eax);
    put(OPEN, 1);
    put(BLOCK, 0);
    put(FRAME, 0);
    let read = |part: Part, r: &mut Regs, buffer: usize, len: u32| -> Result<bool, Refused> {
        r.eax = get(HANDLE);
        r.ebx = a(buffer);
        r.ecx = len;
        part(3, r)?;
        Ok(read_good(
            r.carry(),
            r.eax,
            r.ecx,
            get(buffer),
            buffer == HEADER,
        ))
    };
    if !read(part, &mut r, HEADER, HEADER_LEN)? {
        return failed(part, &mut out, &r);
    }
    // `files::version_differs` (eax the version, kept): on another version the close gets
    // eax 0x50400.
    r.eax = VERSION;
    if !version_good(get(HEADER + 8)) {
        return failed(part, &mut out, &r);
    }
    r.eax = get(HANDLE);
    r.ebx = 2;
    r.ecx = INDEX_LEN.wrapping_neg();
    part(4, &mut r)?;
    if r.carry() || !read(part, &mut r, INDEX, INDEX_LEN)? {
        return failed(part, &mut out, &r);
    }
    match index_blocks(get(BLOCKS)) {
        Blocks::Bad => return failed(part, &mut out, &r),
        Blocks::Held(n) => put(BLOCKS, n),
        Blocks::Kept => {}
    }
    let mut block = 0u32;
    loop {
        r.eax = block;
        part(5, &mut r)?;
        if r.carry() {
            return failed(part, &mut out, &r);
        }
        crate::race::viewer::move_dwords(CAMERAS + 8 * block as usize, BLOCK_DATA, CAMERA_DWORDS);
        r.ecx = 0;
        r.esi = a(BLOCK_DATA) + 8;
        r.edi = a(CAMERAS) + 8 * block + 8;
        block += 1;
        if block == get(BLOCKS) {
            break;
        }
    }
    let (mark_in, mark_out) = opened_marks(get(BLOCKS));
    put(MARK_IN, mark_in);
    put(MARK_OUT, mark_out);
    put(BLOCK_CURSOR, a(BLOCK_DATA));
    r.eax = 0;
    part(5, &mut r)?;
    if r.carry() {
        return failed(part, &mut out, &r);
    }
    unsafe {
        std::ptr::copy_nonoverlapping(
            a(CARS_FROM) as *const u8,
            a(CAR_HEADER) as *mut u8,
            CAR_HEADER_LEN,
        )
    };
    r.ecx = 0;
    r.esi = a(CARS_FROM) + CAR_HEADER_LEN as u32;
    r.edi = a(CAR_HEADER) + CAR_HEADER_LEN as u32;
    part(7, &mut r)?;
    out.set_carry(false);
    out.set_zero(opened_zero(r.eax));
    Ok(out)
}

/// In the game: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::files::handle;
    unsafe {
        match place {
            1 => {
                let done = crate::files::copy_file::copy_file(r.eax as _, r.ebx as _)?;
                r.set_carry(!done);
            }
            2 => match handle::open_by_mode(r.eax as _, r.ebx)? {
                Some(file) => {
                    r.eax = file;
                    r.set_carry(false);
                }
                None => {
                    r.eax = u32::MAX;
                    r.set_carry(true);
                }
            },
            3 => {
                let (n, ok) = handle::read(r.eax, r.ebx as *mut u8, r.ecx);
                r.eax = n;
                r.set_carry(!ok);
            }
            4 => {
                let at = handle::seek(r.eax, r.ecx as i32, r.ebx);
                r.set_carry(at.is_none());
            }
            5 => {
                let failed = crate::drive::read_replay_frame_0::read_replay_frame_0(r.eax)?;
                r.set_carry(failed);
            }
            6 => *r = crate::race::close_viewed_replay::close_viewed_replay(r)?,
            _ => *r = crate::race::replay_patches::apply_replay_patches(r)?,
        }
    }
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| open_viewed_replay(r, &mut game_part))
}
