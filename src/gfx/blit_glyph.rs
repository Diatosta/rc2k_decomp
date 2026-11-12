//! `gfx::blit_glyph` (0x4069B2): draws one run-length-encoded glyph on the global surface,
//! clipped to the global clip rectangle.
//!
//! Takes: glyph index, x, y, palette (256 dwords), glyph table
//! No surface, a null table and a glyph index past the table are fatal errors in the original:
//! `glyph_record` returns them as errors.
//!
//! A glyph table starts with one dword offset per glyph (so the first offset / 4 is the glyph
//! count). A normal record is `[x offset][y offset][width][height]` in bytes, then per row
//! runs of `[count][gap][count pixels]`, a count of 0 ending the row. Its pixels are palette
//! indices: stored raw on an 8-bit surface, through the palette on 16-bit and 32-bit ones.
//! An extended record starts with a dword >= 0xFFFFFFFC, then the same four values as words;
//! its counts, gaps and pixels are words (marker 0xFFFFFFFE) or dwords (any other marker) and
//! the pixels are copied as they are, whatever the surface depth.

use crate::errors::{BadInput, FatalError, Refused};
use crate::gfx::glyph_store::{self, Glyphs};
use crate::gfx::surface::{Clip, Surface, Target, clip, surface};
use rc2k_core::strings::bytes::Bytes as _;

// ---- game data ----

/// The four work globals the blitter leaves behind. Each is only written once the clipping
/// has got that far, so a glyph that is clipped away leaves the later ones as they were.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Scratch {
    /// Pixels clipped off on the left.
    pub left_skip: i32,
    /// Pixels of each row inside the clip rectangle.
    pub visible_w: i32,
    /// Rows clipped off at the top.
    pub top_skip: i32,
    /// Visible rows. Counted down to 0 while drawing, unless nothing is clipped horizontally.
    pub rows_left: i32,
}

fn read_scratch(gfx: &crate::gfx::owner::Gfx) -> Scratch {
    let drawing = &gfx.drawing;
    Scratch {
        left_skip: drawing.blit.left_skip.get() as i32,
        visible_w: drawing.blit.visible_width.get() as i32,
        top_skip: drawing.blit.top_skip.get() as i32,
        rows_left: drawing.blit.rows_left.get() as i32,
    }
}

fn write_scratch(s: &Scratch, gfx: &crate::gfx::owner::Gfx) {
    let drawing = &gfx.drawing;
    drawing.blit.left_skip.set(s.left_skip as u32);
    drawing.blit.visible_width.set(s.visible_w as u32);
    drawing.blit.top_skip.set(s.top_skip as u32);
    drawing.blit.rows_left.set(s.rows_left as u32);
}

/// The three conditions the original reports as fatal errors, each with a null message.
pub const NO_SURFACE: FatalError = FatalError {
    code: 0x80A1_9D97,
    message: "",
};
pub const NO_GLYPH_TABLE: FatalError = FatalError {
    code: 0x80A7_DF31,
    message: "",
};
pub const GLYPH_OUT_OF_RANGE: FatalError = FatalError {
    code: 0x80A7_DF32,
    message: "",
};

/// Record number `glyph` of a glyph table after the three checks
/// the original makes first.
pub fn glyph_record<'a>(
    fonts: &'a crate::gfx::owner::FontStores,
    surface: &Surface,
    table: u32,
    glyph: u32,
) -> Result<Glyphs<'a>, Refused> {
    if surface.target.is_none() {
        return Err(NO_SURFACE.into());
    }
    if table == 0 {
        return Err(NO_GLYPH_TABLE.into());
    }
    let records = glyph_store::bytes(fonts, table, "a glyph table")?;
    let count = records.u32_le(0).ok_or(BadInput("a glyph table"))?;
    if glyph >= count >> 2 {
        return Err(GLYPH_OUT_OF_RANGE.into());
    }
    let offset = records
        .u32_le(glyph.wrapping_mul(4) as usize)
        .ok_or(BadInput("a glyph table's offset"))?;
    let offset = offset as usize;
    Ok(match records {
        Glyphs::Held(b, from) => Glyphs::Held(b, from.wrapping_add(offset)),
        Glyphs::Jade(jade, from) => Glyphs::Jade(jade, from.wrapping_add(offset)),
        Glyphs::Cells(which, cells, from) => Glyphs::Cells(which, cells, from.wrapping_add(offset)),
    })
}

// Converted: `rc2k_core::gfx::blit`. Here the record is read as raw game memory; the core's
// offsets from the surface's start go to its target (`gfx::surface::Target`).

/// Draws the glyph `record` at (`x`, `y`). `put(offset, bytes)` receives every write, the
/// offset from the surface's start.
#[allow(clippy::too_many_arguments)] // the original's five inputs, the surface, the clip and the sink
pub fn blit_glyph(
    fonts: &crate::gfx::owner::FontStores,
    record: &Glyphs,
    x: i32,
    y: i32,
    palette: Option<u32>,
    surface: &Surface,
    clip: &Clip,
    scratch: &mut Scratch,
    put: &mut dyn FnMut(u32, &[u8]),
) -> Result<(), Refused> {
    use rc2k_core::gfx::blit as core;
    let mut work = core::Scratch {
        left_skip: scratch.left_skip,
        visible_w: scratch.visible_w,
        top_skip: scratch.top_skip,
        rows_left: scratch.rows_left,
    };
    // The palette read inside its allocation, and only when a colour is looked up (the
    // sprites have none: the original hands them leftover values it never reads, as their
    // records are direct colour); an entry outside, or a lookup with no palette, answers 0
    // here and the blit is refused afterwards (the core's colour has no failure).
    let colours = std::cell::OnceCell::new();
    let past_palette = std::cell::Cell::new(false);
    let colour = |n: u8| {
        let at = 4 * n as usize;
        let Some(palette) = palette else {
            past_palette.set(true);
            return 0;
        };
        let Ok(colours) = colours.get_or_init(|| glyph_store::bytes(fonts, palette, "a palette"))
        else {
            past_palette.set(true);
            return 0;
        };
        colours.u32_le(at).unwrap_or_else(|| {
            past_palette.set(true);
            0
        })
    };
    let blitted = core::blit_glyph(
        record,
        x,
        y,
        &colour,
        &rc2k_core::gfx::raster::Surface {
            bits: surface.bits_per_pixel,
            bytes_per_pixel: surface.bytes_per_pixel,
            pitch: surface.pitch,
        },
        &rc2k_core::gfx::raster::Clip {
            left: clip.left,
            right: clip.right,
            top: clip.top,
            bottom: clip.bottom,
        },
        &mut work,
        &mut |offset, bytes| put(offset, bytes),
    );
    let Ok(()) = blitted else {
        return Err(BadInput("a read past a glyph's record").into());
    };
    if past_palette.get() {
        return Err(BadInput(if palette.is_some() {
            "a colour past a palette's allocation"
        } else {
            "a palette-indexed glyph drawn with no palette"
        })
        .into());
    }
    *scratch = Scratch {
        left_skip: work.left_skip,
        visible_w: work.visible_w,
        top_skip: work.top_skip,
        rows_left: work.rows_left,
    };
    Ok(())
}

/// Draws glyph `glyph` of `table` on the game's surface with the game's clip rectangle, and
/// updates the blitter's work globals: what a call of the original does.
pub fn blit_glyph_on_surface(
    glyph: u32,
    table: u32,
    x: i32,
    y: i32,
    palette: Option<u32>,
    gfx: &crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    let surface = surface(gfx);
    let record = glyph_record(&gfx.fonts, &surface, table, glyph)?;
    let mut scratch = read_scratch(gfx);
    let target = surface.target.ok_or(NO_SURFACE)?;
    let surface_window = match target {
        Target::Locked(handle) => crate::gfx::locked_pixels::Place::start(handle),
        Target::Picture(_) => crate::gfx::locked_pixels::Place::start(0),
    };
    {
        blit_glyph(
            &gfx.fonts,
            &record,
            x,
            y,
            palette,
            &surface,
            &clip(gfx),
            &mut scratch,
            &mut |offset, bytes| match target {
                Target::Locked(_) => {
                    surface_window.must_write_bytes_at(offset, bytes, "the surface");
                }
                Target::Picture(n) => {
                    if let Err(e) = crate::gfx::picture::write_bytes(n, offset as usize, bytes) {
                        crate::fatal::raise::end("the surface", e)
                    }
                }
            },
        )
    }?;
    write_scratch(&scratch, gfx);
    Ok(())
}

// ---- a call, as the canvases pass it on ----

/// One call of `blit_glyph`, as its caller made it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BlitCall {
    pub glyph: u32,
    pub x: i32,
    pub y: i32,
    /// A font's palette (a glyph store handle); none for the sprite table's sprites, which
    /// are direct colour (`gfx::build_menu_sprites` checks).
    pub palette: Option<u32>,
    pub table: u32,
    /// `g_clipRight` at the time of the call: one caller narrows it around a blit.
    pub clip_right: i32,
}

impl BlitCall {
    /// A call made now, with the clip rectangle's right edge as the canvas has it at this
    /// moment: glyph `glyph` of a font's table with its palette.
    pub fn glyph(glyph: u32, x: i32, y: i32, palette: u32, table: u32, clip_right: i32) -> Self {
        Self {
            glyph,
            x,
            y,
            palette: Some(palette),
            table,
            clip_right,
        }
    }

    /// Sprite `glyph` of the sprite table (`g_spriteTable`), made now with the clip
    /// rectangle's right edge `clip_right`.
    pub fn sprite(
        menus: &crate::menu::owner::Menus,
        glyph: u32,
        x: i32,
        y: i32,
        clip_right: i32,
    ) -> Self {
        Self {
            glyph,
            x,
            y,
            palette: None,
            table: crate::gfx::blit_marker_pair::sprite_table(menus),
            clip_right,
        }
    }
}
