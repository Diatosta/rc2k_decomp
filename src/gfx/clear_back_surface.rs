//! `gfx::clear_back_surface` (0x40579A): with a front surface (`g_frontSurface`), the
//! screen locked (`gfx::lock_surface`; failing, nothing), its rows (`g_screenHeight` of
//! `g_screenRowDwords` dwords from `dword_4F4988`, `dword_4F498C` bytes between them)
//! zeroed and unlocked (`gfx::unlock_surface`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! Callers: `race::viewer_frame` (at the replay's end), `gfx::set_screen_mode`,
//! `sub_405B10`. Its calls through `part`: 1 the lock (carry), 2 the unlock; in the game the
//! platform's through the game. A height of 0 is refused (the original goes round 2^32
//! times), before anything. Shadow mode runs the original alone (it locks the screen).
//! Self-test: a copy whose calls go to stand-ins, against the Rust version, the screen's
//! variables pointed at a buffer of the test's: no front surface, the lock failing, two
//! layouts.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, rva};
use crate::physics::flow::g;
use crate::race::viewer::{Part, regs_entry};

const FRONT: usize = 0x4F32B4; // g_frontSurface
const PIXELS: usize = 0x4F4988; // dword_4F4988
const HEIGHT: usize = 0x4F4950; // g_screenHeight
const ROW_DWORDS: usize = 0x4F4960; // g_screenRowDwords
const GAP: usize = 0x4F498C; // dword_4F498C

/// The Rust version: the registers it leaves (the entry's).
pub fn clear_back_surface(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    if g(FRONT) == 0 {
        return Ok(*entry);
    }
    if g(HEIGHT) == 0 {
        return Err(BadInput("a screen of no rows: the original goes round 2^32 times").into());
    }
    let mut r = *entry;
    part(1, &mut r)?;
    if r.carry() {
        return Ok(*entry);
    }
    let mut row = g(PIXELS);
    for _ in 0..g(HEIGHT) {
        let dwords = g(ROW_DWORDS);
        unsafe { std::ptr::write_bytes(row as *mut u32, 0, dwords as usize) };
        row = row.wrapping_add(4 * dwords).wrapping_add(g(GAP));
    }
    r.eax = 0;
    r.edx = 0;
    r.ecx = 0;
    r.edi = row;
    part(2, &mut r)?;
    Ok(*entry)
}

/// In the game: the platform's lock and unlock through the game.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    let at = if place == 1 {
        crate::gfx::take_screenshot::LOCK
    } else {
        crate::gfx::take_screenshot::UNLOCK
    };
    crate::calls::call(rva(at), r);
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| clear_back_surface(r, &mut game_part))
}
