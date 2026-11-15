//! Makes the font a highlighted menu item is drawn with, whose colours pulse between those of two
//! fonts.
//! In the half pulses (length `half_pulse`) where `time & half_pulse` is not 0 the blend goes from
//! font A towards B, in the others from B towards A. The font it starts from is copied to the
//! blended font (`FontObjects::blended`). Unless `blend_mode` is 0, a 15-entry palette is then made
//! in `FontStores::blended_palette` from the two fonts' palettes, `time & (half_pulse - 1)` of the
//! way (with `blend_mode` = 2 that amount t is eased: t * t >> 8, then the same on 255 - t), and
//! the blended font is given that palette. Refuses null fonts, and null palettes when a palette is
//! made.

use crate::errors::BadInput;
use crate::gfx::blend_palette::blend_palette;
use crate::gfx::owner::Gfx;
use crate::gfx::pixel_format::surface_format;
use crate::gfx::typefaces::FontRef;
use crate::strings::owner::Strings;

const PALETTE_ENTRIES: u32 = 15;

pub fn blend_font(
    strings: &Strings,
    gfx: &'static Gfx,
    system: &crate::system::owner::System,
    time: u32,
    half_pulse: u32,
    a: impl Into<FontRef>,
    b: impl Into<FontRef>,
) -> Result<(), BadInput> {
    let (a, b) = (a.into(), b.into());
    let mode: u32 = strings.text.blend_mode.get();
    let (from_a, t) = rc2k_core::strings::text::pulse(time, half_pulse, mode == 2);
    let (from, to) = if from_a { (a, b) } else { (b, a) };
    // The palettes are read after the copy, so if the second font is the blended font itself, its
    // palette is by then the first one's: kept.
    let from_font = from.object(&gfx.fonts.typefaces).get();
    let to_palette = if to.is_blended() {
        from_font.palette
    } else {
        to.object(&gfx.fonts.typefaces).get().palette
    };
    if mode != 0 && (from_font.palette == 0 || to_palette == 0) {
        return Err(BadInput("a font without a palette"));
    }
    gfx.fonts.typefaces.blended.set(from_font);
    if mode == 0 {
        return Ok(());
    }
    blend_palette(
        &gfx.fonts,
        &gfx.pixel.formats,
        from_font.palette,
        to_palette,
        crate::gfx::glyph_store::BLENDED_PLACE,
        PALETTE_ENTRIES,
        t,
        &surface_format(system),
    )?;
    gfx.fonts
        .typefaces
        .blended
        .palette
        .set(crate::gfx::glyph_store::BLENDED_PLACE);
    Ok(())
}
