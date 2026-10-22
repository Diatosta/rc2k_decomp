//! `gfx::clear_front_surface` (0x4059F4): the front (shown) surface made black.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//! Nothing without `g_frontSurface`. The surface locked (`sub_405996`: DirectDraw's `Lock`
//! into the description at `dword_4F3400`, tried again while the surface is lost; failing,
//! a log line and the carry): nothing more. Else `g_screenHeight` rows of
//! `g_screenRowDwords` zero dwords from the description's memory (`dword_4F3424`), each
//! row `dword_4F3410` (the pitch) on from the last, then unlocked (`sub_4059D5`: `Unlock`;
//! failing, fatal error 0x80208272, raised there). The lock and unlock are the platform's
//! (DirectDraw), through the game; in the standalone program the lock fails (its front
//! surface is SDL's window: `standalone.rs` would need a lock of its own for this to clear
//! it), so nothing is written. Refuses (before anything) a height of 0: the original's
//! `dec edx; jnz` would clear 2^32 rows. Six callers (the screen mode's set-up, the films,
//! the menus). Shadow mode runs the original alone (the lock is made once). Self-test: a
//! copy whose six variables are the test's and whose lock and unlock go to stand-ins (the
//! lock failing or not), on a buffer of the test's, against the Rust version.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_addr, rva, standalone};

// ---- game data ----

/// The calls by place: 0 the lock (carry: failed), 1 the unlock.
const CALLEES: [usize; 2] = [0x405996, 0x4059D5];
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// Where the variables are (the game's, or a test's).
#[derive(Clone, Copy)]
pub struct FrontVars {
    /// g_frontSurface
    pub front: u32,
    /// dword_4F3424: the locked memory
    pub memory: u32,
    /// dword_4F3410: the pitch
    pub pitch: u32,
    /// g_screenRowBytes
    pub row_bytes: u32,
    /// g_screenHeight
    pub height: u32,
    /// g_screenRowDwords
    pub row_dwords: u32,
}

impl FrontVars {
    pub fn game() -> Self {
        let a = |x: usize| game_addr(rva(x)) as u32;
        Self {
            front: a(0x4F32B4),
            memory: a(0x4F3424),
            pitch: a(0x4F3410),
            row_bytes: a(0x4F495C),
            height: a(0x4F4950),
            row_dwords: a(0x4F4960),
        }
    }
}

// ---- Rust version ----

unsafe fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// The rows cleared: `rows` of `dwords` zero dwords from `memory`, each row's start `gap`
/// bytes after the last one's end.
pub unsafe fn clear_rows(memory: u32, dwords: u32, gap: u32, rows: u32) {
    let mut at = memory;
    for _ in 0..rows {
        for k in 0..dwords {
            unsafe { (at.wrapping_add(4 * k) as *mut u32).write_unaligned(0) };
        }
        at = at.wrapping_add(4 * dwords).wrapping_add(gap);
    }
}

pub unsafe fn clear_front_surface(entry: &Regs, v: &FrontVars, part: Part) -> Result<(), Refused> {
    unsafe {
        if get(v.front) == 0 {
            return Ok(());
        }
        let rows = get(v.height);
        if rows == 0 {
            return Err(BadInput("a screen height of 0 (2^32 rows)").into());
        }
        let mut r = *entry;
        part(0, &mut r)?;
        if r.carry() {
            return Ok(());
        }
        let gap = get(v.pitch).wrapping_sub(get(v.row_bytes));
        clear_rows(get(v.memory), get(v.row_dwords), gap, rows);
        let mut r = *entry;
        part(1, &mut r)?;
        Ok(())
    }
}

/// In the game: the lock and unlock through the game (DirectDraw). In the standalone
/// program the lock fails.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    if standalone() {
        r.set_carry(place == 0);
        return Ok(());
    }
    crate::calls::call(rva(CALLEES[place as usize]), r);
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry = *r;
    unsafe { clear_front_surface(&entry, &FrontVars::game(), &mut game_part) }?;
    Ok(())
}
