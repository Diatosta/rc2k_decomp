//! `race::viewer_palettes` (0x41D8BF): for the viewer's page, kind 0x35's and 0x2A's first texture
//! and slot (`g_kindFirstTextures` / `g_kindFirstSlots` +0x350, +0x324) into `viewer_first_texture_a` ..
//! `viewer_device_texture`, and the six fonts' palettes pointed at the glyph store's viewer palette, which gets the fonts' 256
//! colours (`g_fontPalettes`) taken apart in the screen's format (`gfx::pixel_to_components`,
//! `g_surfaceFormat`), made opaque and put together in the texture format
//! (`gfx::components_to_pixel`, `[Textures.kinds[0] + 0xC] + 8`).
//!
//! Caller: `race::viewer_load_stage`.

use crate::errors::BadInput;
use crate::race::owner_shared::RaceShared;

/// The replay viewer's panel fonts, whose palettes are pointed at the colours, in the
/// original's order.
const PALETTES: [crate::gfx::typefaces::FontId; 6] = {
    use crate::gfx::typefaces::fonts::*;
    [PANEL_A, PANEL_B, PANEL_C, PANEL_D, PANEL_E, PANEL_F]
};

pub fn viewer_palettes(
    shared: &RaceShared,
    scene: &crate::scene::owner::Scene,
    gfx: &'static crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), BadInput> {
    let textures = &scene.textures;
    // The textures and slots kept: (to, from).
    for (to, from) in [
        (
            &shared.replay_file().viewer_first_texture_a,
            &textures.kinds.get().kind_first_textures[212],
        ), // g_kindFirstTextures + 0x350
        (
            &shared.replay_file().viewer_list_texture,
            &textures.kinds.get().kind_first_slots[212],
        ), // g_kindFirstSlots + 0x350
        (
            &shared.replay_file().viewer_panel_texture,
            &textures.kinds.get().kind_first_textures[201],
        ), // g_kindFirstTextures + 0x324
        (
            &shared.replay_file().viewer_device_texture,
            &textures.kinds.get().kind_first_slots[201],
        ), // g_kindFirstSlots + 0x324
    ] {
        to.set(from.get());
    }
    for font in PALETTES {
        font.object(&gfx.fonts.typefaces)
            .palette
            .set(crate::gfx::glyph_store::VIEWER_PALETTE);
    }
    {
        let screen = crate::gfx::pixel_format::surface_format(system);
        let texture = crate::gfx::pixel_format::layout(
            system,
            gfx,
            crate::gfx::pixel_format::texture_kind_format(3, gfx),
        )?;
        // Converted: `rc2k_core::race::viewer_flow::{font_colour, FONT_COLOURS}` (colour
        // by colour, as the original).
        use rc2k_core::race::viewer_flow::{FONT_COLOURS, font_colour};
        let from = gfx.fonts.font_block.palettes.get();
        for n in 0..FONT_COLOURS as u32 {
            let colour = font_colour(
                u32::from_le_bytes(
                    crate::gfx::glyph_store::read_bytes(
                        &gfx.fonts,
                        from,
                        4 * n,
                        4,
                        "a font palette",
                    )?
                    .try_into()
                    .map_err(|_| BadInput("a font palette"))?,
                ),
                |c| crate::gfx::pixel_to_components::pixel_to_components(&screen, c),
                |p| crate::gfx::components_to_pixel::components_to_pixel(&texture, p, gfx),
            );
            crate::gfx::glyph_store::write_bytes(
                &gfx.fonts,
                crate::gfx::glyph_store::VIEWER_PALETTE,
                4 * n,
                &colour.to_le_bytes(),
                "a font palette",
            )?;
        }
    }
    Ok(())
}
