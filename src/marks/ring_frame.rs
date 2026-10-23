//! `marks::ring_frame` (0x4CB5B9): the emitters' ring drawn: render states 0x15 = 4, 0x13 =
//! 5, 0x14 = 6, 0x1C = 1, 0x1B = 1, `scene::set_texture` (`dword_5D1E34`),
//! `scene::depth_test_off`; `dword_939BA6` the settings' +0x7C's low 24 bits,
//! `g_marksKind1Choice` `dword_8BC50D` (the rain's drops); `marks::run_ring`; render state
//! 0x14 = 2; `marks::run_ribbons`.
//!
//! In:  nothing (`g_marksSettings`)
//! Out: the registers as `run_ribbons` leaves them
//!
//! Caller: `marks::run_frame` (0x4D160B). Its calls through `part` (1 a render state, 2 the
//! texture, 3 the depth test, 4 `run_ring`, 5 `run_ribbons`); in the game their Rust
//! versions. Refuses null settings, before anything. Shadow mode runs the original alone
//! (it draws). Self-test: a copy whose calls go to stand-ins, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, game_write, rva};
use crate::marks::setup::SETTINGS_RVA;

const TEXTURE_RVA: usize = rva(0x5D1E34); // dword_5D1E34
const TINT_RVA: usize = rva(0x939BA6); // dword_939BA6
const CHOICE_RVA: usize = rva(0x93A1D6); // g_marksKind1Choice
const DROPS_RVA: usize = rva(0x8BC50D); // dword_8BC50D
/// The render states set (value, state).
const STATES: [(u32, u32); 5] = [(4, 0x15), (5, 0x13), (6, 0x14), (1, 0x1C), (1, 0x1B)];

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn ring_frame(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let s: u32 = game_read(SETTINGS_RVA);
        if s == 0 {
            return Err(BadInput("no marks settings").into());
        }
        for (value, state) in STATES {
            g.eax = value;
            g.ebx = state;
            crate::marks::ring_frame::game_part(1, &mut g)?;
        }
        g.eax = game_read(TEXTURE_RVA);
        crate::marks::ring_frame::game_part(2, &mut g)?;
        crate::marks::ring_frame::game_part(3, &mut g)?;
        g.esi = game_read(SETTINGS_RVA);
        g.eax = ((g.esi + 0x7C) as *const u32).read_unaligned() & 0xFF_FFFF;
        game_write(TINT_RVA, g.eax);
        g.eax = game_read(DROPS_RVA);
        game_write(CHOICE_RVA, g.eax);
        crate::marks::ring_frame::game_part(4, &mut g)?;
        g.eax = 2;
        g.ebx = 0x14;
        crate::marks::ring_frame::game_part(1, &mut g)?;
        crate::marks::ring_frame::game_part(5, &mut g)?;
    }
    Ok(g)
}

/// In the game: the calls' Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            1 => {
                let (value, state) = (r.eax, r.ebx);
                crate::scene::set_render_state::set_render_state(value, state);
            }
            2 => crate::scene::set_texture::set_texture(r.eax),
            3 => crate::scene::depth_test_off::depth_test_off(),
            4 => {
                use crate::marks::run_ring as ring;
                let left = ring::run_ring(r, &mut ring::game_part)?;
                *r = Regs { esp: r.esp, ..left };
            }
            _ => {
                use crate::marks::run_ribbons as ribbons;
                let left = ribbons::run_ribbons(r, &mut ribbons::game_part)?;
                *r = Regs { esp: r.esp, ..left };
            }
        }
    }
    Ok(())
}
