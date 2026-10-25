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
//! Caller: `marks::run_frame` (0x4D1549). in the game the platform's two through the game,
//! the others their Rust versions. Refuses null settings (before anything) and a locked
//! texture (`g_surface` after the lock) wider than the mask's 0x100 (unlocked again, what
//! was written put back).

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::marks::store::{Slot, game_read, game_write};

const SEEN_RVA: Slot = Slot::at(0x934D6E); // dword_934D6E
const CHOSEN_RVA: Slot = Slot::at(0x934D72); // dword_934D72
const MASK_RVA: Slot = Slot::at(0x934DB6); // g_marksMaskBlock
const BMASK_RVA: Slot = Slot::at(0x934DFA); // g_marksBMaskBlock
const SLOT_RVA: usize = rva(0x5D0E10); // dword_5D0E10
const FORGET_RVA: usize = rva(0x5D1E10); // dword_5D1E10
const WIDTH_RVA: usize = rva(0x4F49A0); // g_surface.width

/// The Rust version (see the header): the lock and unlock (`d3d::lock_texture`,
/// `unlock_texture`), `gfx::put_surface_pixel`, `scene::forget_slot_texture`.
// Converted: `rc2k_core::marks::masks::{swapped, check_width, texture_pixels}`
pub unsafe fn swap_mask() -> Result<(), Refused> {
    use crate::marks::load_mask_pictures::{lock, put_pixel};
    use rc2k_core::marks::masks;
    unsafe {
        let s: u32 = crate::marks::setup::SETTINGS.pointer.get();
        if s == 0 {
            return Err(BadInput("no marks settings").into());
        }
        let switch = ((s + 0x2704) as *const u32).read_unaligned();
        let Some(second) = masks::swapped(switch, game_read(SEEN_RVA)) else {
            return Ok(());
        };
        let kept: [u32; 2] = [game_read(SEEN_RVA), game_read(CHOSEN_RVA)];
        game_write(SEEN_RVA, switch);
        game_write(
            CHOSEN_RVA,
            game_read::<u32>(if second { BMASK_RVA } else { MASK_RVA }),
        );
        lock(game_read(SLOT_RVA), true)?;
        // The width is the locked texture's (`g_surface` while it is locked), not the
        // screen's: checked here (2026-10-18: checked before the lock, it refused every
        // frame). A row wider than the mask's would read past its block: unlocked, put
        // back and refused.
        let width: u32 = game_read(WIDTH_RVA);
        if let Err(e) = masks::check_width(width) {
            lock(game_read(SLOT_RVA), false)?;
            game_write(SEEN_RVA, kept[0]);
            game_write(CHOSEN_RVA, kept[1]);
            return Err(e.into());
        }
        for (row, col, at) in masks::texture_pixels(width) {
            let chosen: u32 = game_read(CHOSEN_RVA);
            let pixel = ((chosen + 4 * at) as *const u32).read_unaligned();
            put_pixel(pixel, col, row + masks::TEXTURE_ROW_AT)?;
        }
        lock(game_read(SLOT_RVA), false)?;
        crate::scene::forget_slot_texture::forget_slot_texture(game_read(FORGET_RVA))?;
    }
    Ok(())
}
