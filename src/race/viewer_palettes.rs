//! `race::viewer_palettes` (0x41D8BF): for the viewer's page, kind 0x35's and 0x2A's first
//! texture and slot (`g_kindFirstTextures` / `g_kindFirstSlots` +0x350, +0x324) into
//! `dword_5BECB0` .. `dword_5BECBC`, and the six fonts' palettes pointed at
//! `unk_960FA0`, which gets the fonts' 256 colours (`g_fontPalettes`) taken apart in the
//! screen's format (`gfx::pixel_to_components`, `g_surfaceFormat`), made opaque and put
//! together in the texture format (`gfx::components_to_pixel`, `[off_5ABAC0 + 0xC] + 8`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! Caller: `race::viewer_load_stage`.

use crate::errors::BadInput;
use crate::race::replay_file_store::REPLAY_FILE;

/// The replay viewer's panel fonts, whose palettes are pointed at the colours, in the
/// original's order.
const PALETTES: [crate::gfx::typefaces::FontId; 6] = {
    use crate::gfx::typefaces::fonts::*;
    [PANEL_A, PANEL_B, PANEL_C, PANEL_D, PANEL_E, PANEL_F]
};

/// The Rust version.
pub fn viewer_palettes() -> Result<(), BadInput> {
    // The textures and slots kept: (to, from).
    for (to, from) in [
        (
            &REPLAY_FILE.viewer_first_texture_a,
            &crate::scene::owner::scene()
                .textures
                .kinds
                .kind_first_textures[212],
        ), // g_kindFirstTextures + 0x350
        (
            &REPLAY_FILE.viewer_list_texture,
            &crate::scene::owner::scene().textures.kinds.kind_first_slots[212],
        ), // g_kindFirstSlots + 0x350
        (
            &REPLAY_FILE.viewer_panel_texture,
            &crate::scene::owner::scene()
                .textures
                .kinds
                .kind_first_textures[201],
        ), // g_kindFirstTextures + 0x324
        (
            &REPLAY_FILE.viewer_device_texture,
            &crate::scene::owner::scene().textures.kinds.kind_first_slots[201],
        ), // g_kindFirstSlots + 0x324
    ] {
        to.set(from.get());
    }
    for font in PALETTES {
        font.object()
            .palette
            .set(crate::race::viewer_buffers_store::VIEWER_BUFFERS.colours[0].addr());
    }
    {
        let screen = crate::gfx::pixel_format::surface_format()?;
        let texture =
            crate::gfx::pixel_format::layout(crate::gfx::pixel_format::texture_kind_format(3))?;
        // Converted: `rc2k_core::race::viewer_flow::{font_colour, FONT_COLOURS}` (colour
        // by colour, as the original).
        use rc2k_core::race::viewer_flow::{FONT_COLOURS, font_colour};
        let from = crate::gfx::owner::gfx().fonts.menu_fonts.palettes.get();
        let to = &crate::race::viewer_buffers_store::VIEWER_BUFFERS.colours;
        for n in 0..FONT_COLOURS as u32 {
            let colour = font_colour(
                u32::from_le_bytes(
                    crate::gfx::glyph_store::read_bytes(from, 4 * n, 4, "a font palette")?
                        .try_into()
                        .map_err(|_| BadInput("a font palette"))?,
                ),
                |c| crate::gfx::pixel_to_components::pixel_to_components(&screen, c),
                |p| crate::gfx::components_to_pixel::components_to_pixel(&texture, p),
            );
            to[n as usize].set(colour);
        }
    }
    Ok(())
}
