//! A string ("not in this version" and the like, string 0x1FE) in a box in the middle of the
//! surface: measured, the box 0x32 wider and 0x19 higher filled and framed, then the string drawn
//! centred. Takes the x the measuring starts at. Refuses what `gfx::fill_rect` refuses (a surface
//! of neither 16 nor 32 bits).

use crate::errors::{BadInput, Refused};
use crate::session::{GameThread, PlainDword};
use crate::text::draw_text::Canvas;
use rc2k_core::screens::message_box::{FILL, FRAME, STRING, layout};

/// The box's rectangle (left, top, width, height).
pub static BOX: GameThread<[PlainDword; 4]> = GameThread::new([const { PlainDword::new(0) }; 4]);

pub fn draw_message_box(
    o: crate::owners::Owners<'static>,
    x: i32,
    canvas: &mut dyn Canvas,
    gfx: &'static crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    use crate::gfx::rect::{Palette, fill_rect, frame_rect};
    use crate::text::draw_string_by_id::draw_string_by_id_at;
    use crate::text::measure_string_by_id::measure_string_by_id_at;
    let bits = crate::gfx::surface::surface(gfx).bits_per_pixel;
    if bits != 16 && bits != 32 {
        return Err(BadInput("a message box on a surface of neither 16 nor 32 bits").into());
    }
    // The logic is in `rc2k_core::screens::message_box::layout`.
    let m = measure_string_by_id_at(
        crate::strings::append_string_by_id::StringOwners::of(o),
        STRING,
        crate::gfx::typefaces::fonts::MEASURE,
        x,
        canvas,
    )?;
    let (sw, sh) = (
        gfx.drawing.draw_surface.width.get(),
        gfx.drawing.draw_surface.height.get(),
    );
    let font = crate::gfx::typefaces::fonts::BOX_TEXT;
    let b = layout(
        m.width,
        m.outcome.y,
        sw,
        sh,
        crate::gfx::typefaces::fonts::BOX_TEXT
            .object(&gfx.fonts.typefaces)
            .line_height
            .get(),
    );
    let cells = BOX.get();
    cells[2].set(b.rect[2]);
    cells[3].set(b.rect[3]);
    cells[0].set(b.rect[0]);
    cells[1].set(b.rect[1]);
    let rect = cells.each_ref().map(|c| c.get() as i32);
    let palettes = Palette::Held(gfx.fonts.font_block.palettes.get());
    fill_rect(rect, FILL, palettes, gfx)?;
    frame_rect(rect, 0, 0, FRAME, palettes, gfx);
    draw_string_by_id_at(
        crate::strings::append_string_by_id::StringOwners::of(o),
        STRING,
        font,
        b.text_x,
        b.text_y,
        canvas,
    )?;
    Ok(())
}
