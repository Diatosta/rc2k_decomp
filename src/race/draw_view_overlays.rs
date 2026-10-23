//! `race::draw_view_overlays` (0x4559A6): what a view draws over its scene, unless
//! `unk_71AF80` is set (as `race::draw_view_scene`): the HUD's overlay states, render
//! state 3 = 3, the view's marks (`race::run_view_marks`), the overlay states again, the
//! race's sound tick (eax 0xE), a flat triangle (`race::draw_flat_triangle`), the depth
//! buffer on, the filters; the own car drawn (`race::draw_own_car`) with `dword_621900` 0
//! meanwhile (put back after), the screen's tint, the draw batch flushed, the filters.
//!
//! In:  nothing
//! Out: the registers as the last call leaves them; no flags read
//!
//! Callers: the views of `sub_45429B` (five). Its calls through `part` by place
//! (`CALLEES`); in the game the sound tick's and the flush's Rust versions, the others (all
//! hooked) through the game. Shadow mode runs the original alone (it draws). Self-test: a
//! copy whose 12 calls go to stand-ins, against the Rust version, covered and not; the
//! calls, their registers and `dword_621900` compared.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};

/// The calls by place, as IDA shows them.
pub const CALLEES: [usize; 10] = [
    0x45A943, // 0 hud::overlay_states
    0x44EB72, // 1 scene::set_render_state
    0x456336, // 2 race::run_view_marks
    0x421BA9, // 3 sound::race_sound_tick
    0x455912, // 4 race::draw_flat_triangle
    0x44E41F, // 5 scene::depth_buffer_on
    0x44E8FB, // 6 scene::set_filters
    0x455AD3, // 7 race::draw_own_car
    0x455A11, // 8 race::draw_screen_tint
    0x45FD1D, // 9 race::flush_draw_batch
];

const COVERED_RVA: usize = rva(0x71AF80); // unk_71AF80
const OWN_CAR_SWITCH_RVA: usize = rva(0x621900); // dword_621900

/// A call by place (`CALLEES`).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn draw_view_overlays(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    unsafe {
        if game_read::<u32>(COVERED_RVA) != 0 {
            return Ok(r);
        }
        part(0, &mut r)?;
        r.eax = 3;
        r.ebx = 3;
        part(1, &mut r)?;
        part(2, &mut r)?;
        part(0, &mut r)?;
        r.eax = 0xE;
        for place in [3, 4, 5, 6] {
            part(place, &mut r)?;
        }
        let kept: u32 = game_read(OWN_CAR_SWITCH_RVA);
        game_write(OWN_CAR_SWITCH_RVA, 0u32);
        let drawn = part(7, &mut r);
        game_write(OWN_CAR_SWITCH_RVA, kept);
        drawn?;
        for place in [8, 9, 6] {
            part(place, &mut r)?;
        }
    }
    Ok(r)
}

/// In the game: the sound tick's and the flush's Rust versions, the others' through the
/// game (their hooks).
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    match place {
        3 => crate::sound::race_sound_tick::race_sound_tick_in(
            r,
            &mut crate::sound::turn_parts::game_part,
        ),
        9 => unsafe { crate::race::flush_draw_batch::flush_draw_batch_in_game(r) },
        _ => {
            crate::calls::call(rva(CALLEES[place as usize]), r);
            Ok(())
        }
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { draw_view_overlays(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
