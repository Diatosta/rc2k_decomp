//! `menu::reload_front_end` (0x48140D): when `dword_7587E8` is 1 (the scene was let go for
//! a race), the front end's scene set up again: `scene::set_up_engine`,
//! `scene::set_up_front_end_scene`, `scene::set_up_engine`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! 36 callers (the menus after their races). The three calls through `part` (1 the engine,
//! 2 the scene), in the game their Rust versions (`set_up_engine_in_game`,
//! `set_up_front_end_scene_in_game`). Shadow mode runs the original alone (it loads the
//! scene). Self-test: a copy whose calls go to stand-ins, against the Rust version, with
//! the switch 0 and 1.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, rva};

const SWITCH_RVA: usize = rva(0x7587E8); // dword_7587E8

/// A call by place: 1 `set_up_engine`, 2 `set_up_front_end_scene`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn reload_front_end(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    g.eax = unsafe { game_read(SWITCH_RVA) };
    if g.eax == 1 {
        for place in [1, 2, 1] {
            part(place, &mut g)?;
        }
    }
    Ok(*entry)
}

/// In the game: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            1 => crate::scene::set_up_engine::set_up_engine_in_game(r),
            _ => crate::scene::set_up_front_end_scene::set_up_front_end_scene_in_game(r)?,
        }
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { reload_front_end(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
