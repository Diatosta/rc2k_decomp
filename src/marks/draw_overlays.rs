//! `marks::draw_overlays` (0x4C74DA): the race frame's layer of marks drawn before the car's
//! shadow (a reading): `scene::depth_buffer_on`, `scene::depth_test_on`, render states
//! 0x15 = 4, 0x13 = 5, 0x14 = 6, 0x1C = 1, 0x1B = 1; `marks::draw_plane`,
//! `marks::squares_frame`, `marks::hexagons_top`; then the second shadow, the twin of
//! `marks::draw_shadow` without its depth calls (`draw_shadow`'s `SECOND`).
//!
//! In:  nothing
//! Out: the registers as the path leaves them
//!
//! Caller: the race's frame (0x45632B, just before `draw_shadow`). Hooked through a
//! private copy (a call at the entry). The calls through `part` (1 the depth buffer, 2 the
//! depth test, 3 a render state, 4 `draw_plane`, 5 `squares_frame`, 6 `hexagons_top`, 7
//! `draw_fan`); the shadow's vector helpers their Rust versions. Refuses the shadow's
//! refusals (null settings, a corner past `g_marksCorners`, an angle `fsincos` cannot
//! take), before anything. Shadow mode runs the original alone (it draws). Self-test: a
//! copy whose calls go to stand-ins (the vector helpers real), against the Rust version,
//! as `draw_shadow`'s on the second shadow's variables.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read};
use crate::marks::draw_shadow::{SECOND, draw_shadow_with};
use crate::marks::setup::SETTINGS_RVA;

const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (1, 0x1C), (1, 0x1B)];

/// A call by place (see the header).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The shadow's refusals, checked before anything is drawn.
fn shadow_refuses() -> Option<Refused> {
    let corner: u32 = unsafe { game_read(SECOND.corner()) };
    if corner == 0xFFFF_FFFF {
        return None;
    }
    if corner >= 0x100 {
        return Some(BadInput("a shadow corner past g_marksCorners").into());
    }
    if unsafe { game_read::<u32>(SETTINGS_RVA) } == 0 {
        return Some(BadInput("no marks settings").into());
    }
    SECOND.angle_refuses()
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_overlays(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    if let Some(e) = shadow_refuses() {
        return Err(e);
    }
    part(1, &mut g)?;
    part(2, &mut g)?;
    for (value, state) in STATES {
        [g.eax, g.ebx] = [value, state];
        part(3, &mut g)?;
    }
    for place in [4, 5, 6] {
        part(place, &mut g)?;
    }
    unsafe {
        draw_shadow_with(&g, &SECOND, false, &mut |p, r| {
            part(if p == 4 { 7 } else { p }, r)
        })
    }
}

/// In the game: the calls' Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::marks::{draw_plane, draw_shadow, hexagons_top, squares_frame};
    unsafe {
        match place {
            1..=3 => draw_shadow::game_part(place, r)?,
            4 => *r = draw_plane::draw_plane(r, &mut draw_plane::game_part)?,
            5 => *r = squares_frame::squares_frame(r, &mut squares_frame::game_part)?,
            6 => *r = hexagons_top::hexagons_top(r, &mut hexagons_top::game_part)?,
            _ => draw_shadow::game_part(4, r)?,
        }
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { draw_overlays(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
