//! `marks::swap_mask` (0x4C95F5): when the settings' +0x2704 differs from the last seen
//! (`dword_934D6E`, then it), the marks' mask texture is refilled from `g_marksMaskBlock`
//! (or `g_marksBMaskBlock` when it is set; the choice into `dword_934D72`): the texture slot
//! `dword_5D0E10` locked (`sub_41AA4B`, the platform's), rows 0 .. 0x7D of 0x100 pixels
//! (`g_surface`'s width each) put at y + 0x80 by `gfx::put_surface_pixel`, unlocked
//! (`sub_41AB02`), and slot `dword_5D1E10` forgotten (`scene::forget_slot_texture`).
//!
//! In:  nothing (`g_marksSettings`)
//! Out: unchanged when nothing changed; else eax `dword_5D1E10`, ecx the width, edx 0x7E,
//!      ebp and esi as the last pixel left them
//!
//! Caller: `marks::run_frame` (0x4D1549). Its calls through `part` (1 the lock, 2 a pixel,
//! 3 the unlock, 4 the forgetting); in the game the platform's two through the game
//! (section 15), the others their Rust versions. Refuses null settings (before anything) and
//! a locked texture (`g_surface` after the lock) wider than the mask's 0x100 (unlocked again,
//! what was written put back). Shadow mode runs the original alone (it tells
//! the device). Self-test: a copy whose calls go to stand-ins, against the Rust version,
//! the switch changed or not, on a narrow test surface width.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, game_write, rva};
use crate::marks::setup::SETTINGS_RVA;

const SEEN_RVA: usize = rva(0x934D6E); // dword_934D6E
const CHOSEN_RVA: usize = rva(0x934D72); // dword_934D72
const MASK_RVA: usize = rva(0x934DB6); // g_marksMaskBlock
const BMASK_RVA: usize = rva(0x934DFA); // g_marksBMaskBlock
const SLOT_RVA: usize = rva(0x5D0E10); // dword_5D0E10
const FORGET_RVA: usize = rva(0x5D1E10); // dword_5D1E10
const WIDTH_RVA: usize = rva(0x4F49A0); // g_surface.width
const LOCK_RVA: usize = rva(0x41AA4B); // the platform's texture lock
const UNLOCK_RVA: usize = rva(0x41AB02); // the platform's texture unlock

/// A call by place (see the header).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
// Converted: `rc2k_core::marks::masks::{swapped, check_width, texture_pixels}`
pub unsafe fn swap_mask(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    use rc2k_core::marks::masks;
    let mut g = *entry;
    unsafe {
        let s: u32 = game_read(SETTINGS_RVA);
        if s == 0 {
            return Err(BadInput("no marks settings").into());
        }
        g.esi = s;
        let switch = ((s + 0x2704) as *const u32).read_unaligned();
        g.eax = switch;
        let Some(second) = masks::swapped(switch, game_read(SEEN_RVA)) else {
            return Ok(g);
        };
        let kept: [u32; 2] = [game_read(SEEN_RVA), game_read(CHOSEN_RVA)];
        game_write(SEEN_RVA, g.eax);
        g.eax = game_read(if second { BMASK_RVA } else { MASK_RVA });
        game_write(CHOSEN_RVA, g.eax);
        g.eax = game_read(SLOT_RVA);
        part(1, &mut g)?;
        // The width is the locked texture's (`g_surface` while it is locked), not the
        // screen's: checked here (2026-10-18: checked before the lock, it refused every
        // frame). A row wider than the mask's would read past its block: unlocked, put
        // back and refused.
        let width: u32 = game_read(WIDTH_RVA);
        if let Err(e) = masks::check_width(width) {
            let mut u = Regs {
                eax: game_read(SLOT_RVA),
                ..g
            };
            part(3, &mut u)?;
            game_write(SEEN_RVA, kept[0]);
            game_write(CHOSEN_RVA, kept[1]);
            return Err(e.into());
        }
        for (row, col, at) in masks::texture_pixels(width) {
            g.ecx = col;
            g.ebp = at;
            g.esi = game_read(CHOSEN_RVA);
            g.eax = ((g.esi + 4 * g.ebp) as *const u32).read_unaligned();
            g.edx = row + masks::TEXTURE_ROW_AT;
            part(2, &mut g)?;
            g.edx = row;
        }
        // The loops' counters, read back for their last tests.
        g.ecx = width.max(1);
        g.edx = masks::TEXTURE_ROWS;
        g.eax = game_read(SLOT_RVA);
        part(3, &mut g)?;
        g.eax = game_read(FORGET_RVA);
        part(4, &mut g)?;
    }
    Ok(g)
}

/// In the game: the platform's lock and unlock through the game, the others' Rust
/// versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            1 => crate::calls::call(LOCK_RVA, r),
            2 => {
                let left = crate::gfx::put_surface_pixel::put_surface_pixel(r)?;
                r.eax = left.eax;
            }
            3 => crate::calls::call(UNLOCK_RVA, r),
            _ => crate::scene::forget_slot_texture::forget_slot_texture(r.eax)?,
        }
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { swap_mask(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
