//! `audio::mix` (0x40BDC5): a turn of the game's mixer into the primary buffer.
//!
//! In:  nothing
//! Out: every register preserved (the carry of a failure is not read by its caller)
//!
//! `GetCurrentPosition` (+0x10) of `g_primaryBuffer` into `g_playCursor`; the bytes the
//! play cursor moved since the last turn (round the buffer's `g_primarySize`); nothing
//! for none; more than `g_mixMost` (40 ms) are cut to it and the write cursor put that far
//! ahead of the play cursor. Then `audio::mix_channels` for them, `Lock` (+0x2C) from
//! `g_writeCursor` into `g_lockedParts`, the write cursor moved on round the buffer,
//! `audio::write_mix`, `Unlock` (+0x4C). A failing call ends it with the carry, and
//! `DSERR_BUFFERLOST` sets `g_soundBufferLost`. (A `GetStatus` at 0x40BE9D is in it but not
//! reached.) One caller, the mixer's way of the turn.

use crate::audio::fill_buffer::LOCKED_PARTS_RVA;
use crate::audio::mixer::*;
use crate::audio::voices::*;
use crate::errors::Refused;
use crate::store::addr32 as a;

const BUFFER_LOST: i32 = 0x8878_0096u32 as i32;

#[derive(Clone, Copy)]
pub struct Vars {
    pub primary: u32,
    pub size: u32,
    pub most: u32,
    pub play: u32,
    pub write: u32,
    pub parts: u32,
    pub lost: u32,
}

impl Vars {
    pub fn game() -> Self {
        Self {
            primary: a(PRIMARY_RVA),
            size: a(PRIMARY_SIZE_RVA),
            most: a(MIX_MOST_RVA),
            play: a(PLAY_CURSOR_RVA),
            write: a(WRITE_CURSOR_RVA),
            parts: a(LOCKED_PARTS_RVA),
            lost: a(BUFFER_LOST_RVA),
        }
    }
}

/// True for no failure.
pub unsafe fn mix(v: &Vars) -> Result<bool, Refused> {
    unsafe {
        let primary = read(v.primary);
        let failed = |r: i32| {
            if r == BUFFER_LOST {
                write(v.lost, 1);
            }
            false
        };
        let old = read(v.play);
        let position: unsafe extern "system" fn(u32, u32, u32) -> i32 =
            std::mem::transmute(crate::audio::com::method(primary, 0x10));
        let r = position(primary, v.play, 0);
        if r != 0 {
            return Ok(failed(r));
        }
        // Converted: `rc2k_core::audio::streams::cursor_moved` and `round`.
        use rc2k_core::audio::streams::{cursor_moved, round};
        let size = read(v.size);
        let Some((bytes, write_at)) = cursor_moved(old, read(v.play), size, read(v.most)) else {
            return Ok(true);
        };
        if let Some(at) = write_at {
            write(v.write, at);
        }
        crate::audio::mix_channels::mix_channels_in_game(bytes)?;
        let lock: unsafe extern "system" fn(u32, u32, u32, u32, u32, u32, u32, u32) -> i32 =
            std::mem::transmute(crate::audio::com::method(primary, 0x2C));
        let p = v.parts;
        let r = lock(primary, read(v.write), bytes, p, p + 4, p + 8, p + 12, 0);
        if r != 0 {
            return Ok(failed(r));
        }
        write(v.write, round(bytes.wrapping_add(read(v.write)), size));
        crate::audio::write_mix::write_mix_in_game()?;
        let [p1, n1, p2, n2] = (p as *const [u32; 4]).read_unaligned();
        let unlock: unsafe extern "system" fn(u32, u32, u32, u32, u32) -> i32 =
            std::mem::transmute(crate::audio::com::method(primary, 0x4C));
        let r = unlock(primary, p1, n1, p2, n2);
        if r != 0 {
            return Ok(failed(r));
        }
    }
    Ok(true)
}
