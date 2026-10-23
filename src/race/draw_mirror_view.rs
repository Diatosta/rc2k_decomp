//! `race::draw_mirror_view` (0x454806): the mirror's view, unless `dword_64BB18` or
//! `dword_71C3E0` is set or `dword_621480` (the mirror on, a reading) is 0:
//! `dword_63DC58` kept in `dword_63E170`; `dword_606294` and `g_gridReach` saved, made
//! `flt_5C5E2C`'s value (eax too) and 2; the mirror's window (`race::mirror_view_window`),
//! `dword_63DC58` back from `dword_63E170`, the backdrop (`race::draw_view_backdrop`), the
//! scene (`race::draw_view_scene`); the two put back; the full view's window again
//! (`race::full_view_window_2`).
//!
//! In:  nothing
//! Out: the registers as the last call leaves them; no flags read
//!
//! Callers: the views of `sub_45429B` (five). Its calls through `part` by place
//! (`CALLEES`); in the game `draw_view_scene`'s Rust version, the others (all hooked)
//! through the game. Shadow mode runs the original alone (it draws). Self-test: a copy
//! whose four calls go to stand-ins (which change the saved two, to see them put back),
//! against the Rust version, each switch; the calls, their registers and the variables
//! compared.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};

/// The calls by place, as IDA shows them.
pub const CALLEES: [usize; 4] = [
    0x455F17, // 0 race::mirror_view_window
    0x4651E6, // 1 race::draw_view_backdrop
    0x4557AB, // 2 race::draw_view_scene
    0x456004, // 3 race::full_view_window_2
];

const SWITCHES: [usize; 3] = [rva(0x64BB18), rva(0x71C3E0), rva(0x621480)];
const VIEW_VALUE_RVA: usize = rva(0x63DC58); // dword_63DC58
const VIEW_VALUE_KEPT_RVA: usize = rva(0x63E170); // dword_63E170
const FAR_RVA: usize = rva(0x606294); // dword_606294
const MIRROR_FAR_RVA: usize = rva(0x5C5E2C); // flt_5C5E2C
const GRID_REACH_RVA: usize = rva(0x60A87C); // g_gridReach

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_mirror_view(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    unsafe {
        let g = |a: usize| game_read::<u32>(a);
        if g(SWITCHES[0]) != 0 || g(SWITCHES[1]) != 0 || g(SWITCHES[2]) == 0 {
            return Ok(r);
        }
        game_write(VIEW_VALUE_KEPT_RVA, g(VIEW_VALUE_RVA));
        let saved = (g(FAR_RVA), g(GRID_REACH_RVA));
        r.eax = g(MIRROR_FAR_RVA);
        game_write(FAR_RVA, r.eax);
        game_write(GRID_REACH_RVA, 2u32);
        let drawn = (|| {
            crate::race::draw_mirror_view::game_part(0, &mut r)?;
            game_write(VIEW_VALUE_RVA, g(VIEW_VALUE_KEPT_RVA));
            crate::race::draw_mirror_view::game_part(1, &mut r)?;
            crate::race::draw_mirror_view::game_part(2, &mut r)
        })();
        game_write(GRID_REACH_RVA, saved.1);
        game_write(FAR_RVA, saved.0);
        drawn?;
        crate::race::draw_mirror_view::game_part(3, &mut r)?;
    }
    Ok(r)
}

/// In the game: `draw_view_scene`'s Rust version, the others' through the game (their
/// hooks).
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    if place == 2 {
        return unsafe { crate::race::draw_view_scene::draw_view_scene_in_game(r) };
    }
    crate::calls::call(rva(CALLEES[place as usize]), r);
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { draw_mirror_view(r) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
