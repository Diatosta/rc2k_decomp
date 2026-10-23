//! `hud::overlay_pass` (0x45A893): the race frame's 2D pass: `dword_6D14F2` the players
//! (`dword_6238A4`); `scene::depth_buffer_off`; `hud::overlay_states`; with `dword_63DB98`
//! 2, a player whose car block's +0xFA8 is set (`dword_710394` / `dword_711374`) keeps only
//! bit 0x40 of its HUD flags (`dword_6D14F6` / `dword_6D14FA`, put back after) and its
//! `unk_6D1486` / `unk_6D1496` is 0; `hud::draw_hud`; `scene::depth_buffer_on`;
//! `scene::full_view_window`; render state 3 = 3; `dword_63E168` and `dword_63E16C` 1;
//! `hud::draw_view_strips`; an empty function (`nullsub_9`); `scene::depth_buffer_off`;
//! `scene::set_filters`.
//!
//! In:  nothing
//! Out: as the last call leaves them
//!
//! Caller: `sub_4547AE` (0x454786). The window is its Rust version; the rest through
//! `part` (1 / 4 the depth buffer off / on, 2 the states, 3 the HUD, 5 the render state,
//! 6 the strips, 7 the filters), in the game their Rust versions.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};

const PLAYERS_RVA: usize = rva(0x6238A4); // dword_6238A4
const PLAYERS_COPY_RVA: usize = rva(0x6D14F2); // dword_6D14F2
const FLAGS_RVA: [usize; 2] = [rva(0x6D14F6), rva(0x6D14FA)]; // the HUD flags
const MODE_RVA: usize = rva(0x63DB98); // dword_63DB98
const BLOCK_RVA: [usize; 2] = [rva(0x710394), rva(0x711374)]; // the car blocks' +0xFA8
const CLEARED_RVA: [usize; 2] = [rva(0x6D1486), rva(0x6D1496)];
const ON_RVA: [usize; 2] = [rva(0x63E168), rva(0x63E16C)];

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn overlay_pass(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        g.eax = game_read(PLAYERS_RVA);
        game_write(PLAYERS_COPY_RVA, g.eax);
        crate::hud::overlay_pass::game_part(1, &mut g)?;
        crate::hud::overlay_pass::game_part(2, &mut g)?;
        let kept: [u32; 2] = FLAGS_RVA.map(|r| game_read(r));
        if game_read::<u32>(MODE_RVA) == 2 {
            for n in 0..2 {
                if game_read::<u32>(BLOCK_RVA[n]) != 0 {
                    game_write(FLAGS_RVA[n], game_read::<u32>(FLAGS_RVA[n]) & 0x40);
                    game_write(CLEARED_RVA[n], 0u32);
                }
            }
        }
        let done = crate::hud::overlay_pass::game_part(3, &mut g);
        game_write(FLAGS_RVA[1], kept[1]);
        game_write(FLAGS_RVA[0], kept[0]);
        done?;
        crate::hud::overlay_pass::game_part(4, &mut g)?;
        crate::scene::full_view_window::full_view_window().apply(&mut g);
        [g.eax, g.ebx] = [3, 3];
        crate::hud::overlay_pass::game_part(5, &mut g)?;
        game_write(ON_RVA[0], 1u32);
        game_write(ON_RVA[1], 1u32);
        crate::hud::overlay_pass::game_part(6, &mut g)?;
        crate::hud::overlay_pass::game_part(1, &mut g)?;
        crate::hud::overlay_pass::game_part(7, &mut g)?;
    }
    Ok(g)
}

/// In the game: the calls' Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::hud::*;
    unsafe {
        match place {
            1 => crate::scene::depth_buffer_on::depth_buffer(r, false),
            2 => *r = overlay_states::overlay_states(r)?,
            3 => *r = draw_hud::draw_hud(r)?,
            4 => crate::scene::depth_buffer_on::depth_buffer(r, true),
            5 => overlay_states::game_part(place, r)?,
            6 => *r = draw_view_strips::draw_view_strips(r)?,
            _ => crate::scene::set_filters::set_filters(r),
        }
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { overlay_pass(r) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
