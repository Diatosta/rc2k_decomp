//! `effects::swap_mask` (0x4C95F5): when the settings' +0x2704 differs from the last seen
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
//! Caller: `effects::run_frame` (0x4D1549). in the game the platform's two through the game,
//! the others their Rust versions. Refuses null settings (before anything) and a locked
//! texture (`g_surface` after the lock) wider than the mask's 0x100 (unlocked again, what
//! was written put back).

use crate::effects::mask_store::{MASK_BLOCKS, Mask};
use crate::errors::{BadInput, Refused};
use crate::pointers::GamePtr;
use crate::scene::kind_texture_store::KIND_TEXTURES;

/// The Rust version (see the header): the lock and unlock (`d3d::lock_texture`,
/// `unlock_texture`), `gfx::put_surface_pixel`, `scene::forget_slot_texture`.
// Converted: `rc2k_core::effects::masks::{swapped, check_width, texture_pixels}`
pub fn swap_mask() -> Result<(), Refused> {
    use crate::effects::load_mask_pictures::{lock, put_pixel};
    use rc2k_core::effects::masks;
    unsafe {
        let s: u32 = crate::effects::setup::SETTINGS.pointer.get();
        if s == 0 {
            return Err(BadInput("no marks settings").into());
        }
        let switch = GamePtr::new(s + 0x2704).read::<u32>("a mark's field")?;
        let Some(second) = masks::swapped(switch, MASK_BLOCKS.seen.get()) else {
            return Ok(());
        };
        let kept: [u32; 2] = [MASK_BLOCKS.seen.get(), MASK_BLOCKS.chosen.get()];
        MASK_BLOCKS.seen.set(switch);
        let chosen = if second { Mask::Bordered } else { Mask::Plain };
        MASK_BLOCKS.chosen.set(chosen as u32);
        lock(KIND_TEXTURES.kind_first_textures[202].bits(), true)?;
        // The width is the locked texture's (`g_surface` while it is locked), not the
        // screen's: checked here (2026-10-18: checked before the lock, it refused every
        // frame). A row wider than the mask's would read past its block: unlocked, put
        // back and refused.
        let width: u32 = crate::gfx::screen_state::DRAW_SURFACE.width.get();
        if let Err(e) = masks::check_width(width) {
            lock(KIND_TEXTURES.kind_first_textures[202].bits(), false)?;
            MASK_BLOCKS.seen.set(kept[0]);
            MASK_BLOCKS.chosen.set(kept[1]);
            return Err(e.into());
        }
        let pixels = chosen.pixels().with(|p| {
            masks::texture_pixels(width)
                .map(|(row, col, at)| Some((row, col, *p.get(at as usize)?)))
                .collect::<Option<Vec<_>>>()
        })?;
        let Some(pixels) = pixels else {
            lock(KIND_TEXTURES.kind_first_textures[202].bits(), false)?;
            MASK_BLOCKS.seen.set(kept[0]);
            MASK_BLOCKS.chosen.set(kept[1]);
            return Err(BadInput("a mask pixel past its block").into());
        };
        for (row, col, pixel) in pixels {
            put_pixel(pixel, col, row + masks::TEXTURE_ROW_AT)?;
        }
        lock(KIND_TEXTURES.kind_first_textures[202].bits(), false)?;
        crate::scene::forget_slot_texture::forget_slot_texture(
            KIND_TEXTURES.kind_first_slots[202].bits(),
        )?;
    }
    Ok(())
}
