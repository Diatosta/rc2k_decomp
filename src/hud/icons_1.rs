//! `hud::icons_1` (0x4B5839): layout 1's icons: the scale `dword_75C0C0`; the first entry shifted -0xC8, at 0xC8; the texture on one (`dword_75C034`); the second shifted 0xC8, at 0xC8.
//!
//! In:  ebp the HUD's record
//! Out: as the last `draw_icon` leaves them
//!
//! The icons through `part` (1), in the game `draw_icon`'s Rust version. Shadow mode runs
//! the original alone (it draws). Self-test: a copy whose calls go to a stand-in, against
//! the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::hud::batch::TEXTURE_RVA;
use crate::hud::draw_icon::{SCALE_RVA, SHIFT_RVA};

/// A call by place: 1 `draw_icon`.
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn icons_1(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let icon = |g: &mut Regs, ebx: u32, edi: u32, part: &mut Part| {
            [g.ebx, g.edi] = [ebx, edi];
            part(1, g)
        };
        let mut part = part;
        game_write(SCALE_RVA, game_read::<u32>(rva(0x75C0C0)));
        game_write(SHIFT_RVA, 0xFFFF_FF38u32);
        icon(&mut g, 0xC8, 0, &mut part)?;
        game_write(TEXTURE_RVA, game_read::<u32>(TEXTURE_RVA).wrapping_add(1));
        game_write(SHIFT_RVA, 0xC8u32);
        icon(&mut g, 0xC8, 0x10, &mut part)?;
    }
    Ok(g)
}

/// In the game: `draw_icon`'s Rust version.
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::hud::draw_icon as di;
    *r = unsafe { di::draw_icon(r, &mut crate::hud::add_quad::game_part) }?;
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { icons_1(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
