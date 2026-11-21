//! Solid rectangles and lines straight on the surface (`gfx::fill_rect`, `gfx::frame_rect`,
//! `gfx::draw_hline`): the message box's fill and frame. The colour is the byte itself on an 8-bit
//! surface, else `palette[colour]` (a table of 256 dwords). Clipped to the clip rectangle; the work
//! variables are the blitter's skips.

use crate::errors::Refused;
use crate::gfx::owner::Gfx;
use crate::gfx::surface::{clip, surface};

fn set(a: &std::cell::Cell<u32>, v: i32) {
    a.set(v as u32);
}

fn core_clip(gfx: &Gfx) -> rc2k_core::gfx::raster::Clip {
    let c = clip(gfx);
    rc2k_core::gfx::raster::Clip {
        left: c.left,
        right: c.right,
        top: c.top,
        bottom: c.bottom,
    }
}

fn core_surface(gfx: &Gfx) -> rc2k_core::gfx::raster::Surface {
    let s = surface(gfx);
    rc2k_core::gfx::raster::Surface {
        bits: s.bits_per_pixel,
        bytes_per_pixel: s.bytes_per_pixel,
        pitch: s.pitch,
    }
}

/// The palette a colour is looked up in: 256 dwords.
#[derive(Clone, Copy)]
pub enum Palette<'a> {
    /// A font's palette as the game's data keeps it: a place of `gfx::glyph_store`.
    Held(u32),
    /// A palette of Rust's game state: a store, bounded by it.
    Of(&'a dyn crate::memory::cells::Store),
}

impl Palette<'_> {
    /// Entry `colour`.
    fn entry(self, fonts: &crate::gfx::owner::FontStores, colour: u32) -> u32 {
        let what = "a palette entry";
        let bytes = match self {
            Palette::Held(place) => {
                crate::gfx::glyph_store::read_bytes(fonts, place, 4 * colour, 4, what)
            }
            Palette::Of(store) => {
                let mut entry = vec![0; 4];
                store.load(4 * colour as usize, &mut entry).map(|()| entry)
            }
        };
        match bytes {
            Ok(b) => u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
            Err(e) => crate::fatal::raise::end(what, e),
        }
    }
}

/// `row` made `count` pixels of `pixel` (2 or 4 bytes, or the one byte of an 8-bit surface).
fn repeat_pixel(row: &mut Vec<u8>, pixel: &[u8], count: usize) {
    row.clear();
    row.resize(count * pixel.len(), 0);
    match *pixel {
        [a, b, c, d] => row.as_chunks_mut::<4>().0.fill([a, b, c, d]),
        [a, b] => row.as_chunks_mut::<2>().0.fill([a, b]),
        [a] => row.fill(a),
        _ => {
            for chunk in row.chunks_exact_mut(pixel.len().max(1)) {
                chunk.copy_from_slice(pixel);
            }
        }
    }
}

/// The plan's work variables, written for the blitter.
fn set_work(plan: &rc2k_core::gfx::raster::Plan, gfx: &Gfx) {
    let drawing = &gfx.drawing;
    let w = plan.work;
    for (at, value) in [
        (&drawing.blit.left_skip, w.left_skip),
        (&drawing.blit.visible_width, w.width),
        (&drawing.blit.top_skip, w.top_skip),
        (&drawing.blit.rows_left, w.rows),
    ] {
        if let Some(v) = value {
            set(at, v);
        }
    }
}

/// A solid shape given as runs of pixels: the work variables, then every run written whole in
/// colour `colour` (on the canvas, and in the 2D record when that is kept). The shape the
/// quads are noted for goes through `carry_out`.
fn carry_out_runs(
    plan: &rc2k_core::gfx::raster::Plan,
    runs: &[(i32, usize)],
    colour: u32,
    palette: Palette<'_>,
    gfx: &Gfx,
) {
    set_work(plan, gfx);
    let Some(target) = surface(gfx).target else {
        crate::fatal::raise::end("gfx::rect", crate::errors::BadInput("no surface"))
    };
    if runs.is_empty() {
        return;
    }
    let recording =
        crate::gfx::draw2d::enabled() && crate::gfx::draw2d::is_screen(surface(gfx).target);
    let bytes = pixel(&gfx.fonts, plan.bits, colour, palette);
    let mut row = Vec::new();
    for &(offset, count) in runs {
        repeat_pixel(&mut row, &bytes, count);
        if let Err(e) = crate::gfx::surface::write(target, offset as u32, &row, gfx) {
            crate::fatal::raise::end("gfx::rect", e)
        }
        if recording {
            crate::gfx::draw2d::mirror(offset as u32, &row);
        }
    }
}

/// The plan's work variables, then its pixels in colour `colour`.
fn carry_out(plan: &rc2k_core::gfx::raster::Plan, colour: u32, palette: Palette<'_>, gfx: &Gfx) {
    let recording =
        crate::gfx::draw2d::enabled() && crate::gfx::draw2d::is_screen(surface(gfx).target);
    set_work(plan, gfx);
    let Some(target) = surface(gfx).target else {
        crate::fatal::raise::end("gfx::rect", crate::errors::BadInput("no surface"))
    };
    // On the screen the pixels are noted as solid quads for the GPU to draw: the canvas is not
    // written.
    let by_quad = plan.bits == 32
        && !plan.pixels.is_empty()
        && crate::gfx::draw2d::is_screen(Some(target))
        && {
            let bytes = pixel(&gfx.fonts, plan.bits, colour, palette);
            let screen = surface(gfx);
            <[u8; 4]>::try_from(bytes.as_slice()).is_ok_and(|bytes| {
                crate::gfx::draw2d::quads::note_pixels(
                    &plan.pixels,
                    bytes,
                    screen.bytes_per_pixel,
                    screen.pitch,
                )
            })
        };
    if (!by_quad || recording) && !plan.pixels.is_empty() {
        // The pixel's bytes once, written a run of neighbouring pixels at a time.
        let bytes = pixel(&gfx.fonts, plan.bits, colour, palette);
        for (offset, count) in rc2k_core::gfx::raster::runs(&plan.pixels, bytes.len() as i32) {
            let run = bytes.repeat(count);
            if !by_quad && let Err(e) = crate::gfx::surface::write(target, offset as u32, &run, gfx)
            {
                crate::fatal::raise::end("gfx::rect", e)
            }
            if recording {
                crate::gfx::draw2d::mirror(offset as u32, &run);
            }
        }
    }
}

/// Notes a shape drawn on the screen for the 2D record: its `plans` as they were carried out.
fn note(
    shape: crate::gfx::draw2d::Shape,
    plans: &[rc2k_core::gfx::raster::Plan],
    colour: u32,
    palette: Palette<'_>,
    gfx: &Gfx,
) {
    // The pixel's bytes by the depth of the first plan that sets a pixel.
    let drawn = plans
        .iter()
        .find(|plan| !plan.pixels.is_empty())
        .map(|plan| plan.bits);
    note_drawn(shape, drawn, colour, palette, gfx);
}

/// `note` for a shape whose pixels are drawn at depth `drawn` (none: no pixel is set).
fn note_drawn(
    shape: crate::gfx::draw2d::Shape,
    drawn: Option<u32>,
    colour: u32,
    palette: Palette<'_>,
    gfx: &Gfx,
) {
    use crate::gfx::draw2d::{Canvas, Command, Rect, push};
    if !crate::gfx::draw2d::enabled() || !crate::gfx::draw2d::is_screen(surface(gfx).target) {
        return;
    }
    let pixel = drawn
        .map(|bits| pixel(&gfx.fonts, bits, colour, palette))
        .unwrap_or_default();
    push(Command::Rect(Rect {
        shape,
        canvas: Canvas {
            surface: core_surface(gfx),
            clip: core_clip(gfx),
        },
        pixel,
    }));
}

/// The pixel for colour `colour` by the surface's depth: `palette[colour]` as a word or a dword,
/// else the byte itself.
fn pixel(
    fonts: &crate::gfx::owner::FontStores,
    bits: u32,
    colour: u32,
    palette: Palette<'_>,
) -> Vec<u8> {
    let entry = || palette.entry(fonts, colour);
    match bits {
        16 => (entry() as u16).to_le_bytes().to_vec(),
        32 => entry().to_le_bytes().to_vec(),
        _ => vec![colour as u8],
    }
}

/// A rectangle (x, y, width, height) kept in the first 16 bytes of a buffer.
pub fn rect_in(bytes: &impl crate::memory::session::ByteBuffer) -> [i32; 4] {
    std::array::from_fn(|n| bytes.get_u32_at(4 * n) as i32)
}

/// `gfx::fill_rect`: the rectangle (x, y, width, height) filled, clipped, with `palette[colour]`
/// on a 16- or 32-bit surface and with the byte itself on an 8-bit one. Refuses (before anything)
/// any other depth.
pub fn fill_rect(
    rect: [i32; 4],
    colour: u32,
    palette: Palette<'_>,
    gfx: &Gfx,
) -> Result<(), Refused> {
    if !crate::gfx::draw2d::quads::wanted() {
        let (plan, rows) =
            rc2k_core::gfx::raster::fill_runs(rect, &core_clip(gfx), &core_surface(gfx))?;
        carry_out_runs(&plan, &rows, colour, palette, gfx);
        let drawn = (!rows.is_empty()).then_some(plan.bits);
        note_drawn(
            crate::gfx::draw2d::Shape::Fill(rect),
            drawn,
            colour,
            palette,
            gfx,
        );
        return Ok(());
    }
    let plan = rc2k_core::gfx::raster::fill(rect, &core_clip(gfx), &core_surface(gfx))?;
    carry_out(&plan, colour, palette, gfx);
    note(
        crate::gfx::draw2d::Shape::Fill(rect),
        &[plan],
        colour,
        palette,
        gfx,
    );
    Ok(())
}

/// `gfx::draw_hline`: `length` pixels from (x, y) rightwards, clipped (the left clip: what is cut
/// is -x, right only for a left edge of 0).
pub fn draw_hline(x: i32, y: i32, length: i32, colour: u32, palette: Palette<'_>, gfx: &Gfx) {
    if !crate::gfx::draw2d::quads::wanted() {
        let (plan, rows) =
            rc2k_core::gfx::raster::hline_runs(x, y, length, &core_clip(gfx), &core_surface(gfx));
        carry_out_runs(&plan, &rows, colour, palette, gfx);
        let shape = crate::gfx::draw2d::Shape::HLine { x, y, length };
        let drawn = (!rows.is_empty()).then_some(plan.bits);
        note_drawn(shape, drawn, colour, palette, gfx);
        return;
    }
    let plan = rc2k_core::gfx::raster::hline(x, y, length, &core_clip(gfx), &core_surface(gfx));
    carry_out(&plan, colour, palette, gfx);
    let shape = crate::gfx::draw2d::Shape::HLine { x, y, length };
    note(shape, &[plan], colour, palette, gfx);
}

/// `gfx::frame_rect`: the rectangle's outline moved by (dx, dy): the top and bottom rows
/// (`draw_hline`), then the left and right columns between them (`draw_vline`).
pub fn frame_rect(rect: [i32; 4], dx: i32, dy: i32, colour: u32, palette: Palette<'_>, gfx: &Gfx) {
    let plans = rc2k_core::gfx::raster::frame(rect, dx, dy, &core_clip(gfx), &core_surface(gfx));
    for plan in &plans {
        carry_out(plan, colour, palette, gfx);
    }
    note(
        crate::gfx::draw2d::Shape::Frame { rect, dx, dy },
        &plans,
        colour,
        palette,
        gfx,
    );
}
