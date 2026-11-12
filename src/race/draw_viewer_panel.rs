//! `race::draw_viewer_panel` (0x41DA25): the replay viewer's panel: its texture (`viewer_panel_texture`)
//! locked (`d3d::lock_texture`; failing, nothing), cleared, and written on: text 0x1A4 and the time
//! at the frame shown (block * 100 + frame, `strings::append_time`), text 0x1A5 and the replay's
//! length, the names of the block's two cameras (`ViewerAndKinds.viewer_panel.camera_name_ids[0]` by its bytes 0 and 1, at most 0x2E;
//! `strings::append_string_by_id`), each three times 0x78 to the left and 0xD down, texts 0x560 and
//! 0x561 (the fonts at `FontObjects.objects[58].first_char` ..; `text::draw_text_keep_regs`, 0x402368); unlocked
//! (`d3d::unlock_texture`), copied to its device texture (`viewer_device_texture`,
//! `d3d::copy_to_device_texture`), made the texture (`d3d::set_texture`) and drawn as ten quads
//! (`drive::tyre_quad`, the texture's and the screen's rectangles `texture_rect`, `screen_rect`),
//! the switch lights dimmed (0x555555) by the block's camera byte 5 (0: the second, 1: the first).
//!
//! Caller: `race::viewer_frame`.

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::session::Dword;
use crate::text::draw_text::GameCanvas;
const FONT_A: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::PANEL_A;
const FONT_B: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::PANEL_B;
const FONT_C: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::PANEL_C;
const FONT_D: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::PANEL_D;
const FONT_E: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::PANEL_E;

pub fn draw_viewer_panel(
    so: crate::strings::append_string_by_id::StringOwners<'static>,
    race: &Race,
    gfx: &'static crate::gfx::owner::Gfx,
    strings: &crate::strings::owner::Strings,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let replay = &race.replay;
    let text_stores = &strings.text;
    let shared = &race.shared;
    // Converted: `rc2k_core::race::viewer_flow::{shown_time, replay_length, camera_name,
    // switch_lights}`.
    use rc2k_core::race::viewer_flow::{camera_name, replay_length, shown_time, switch_lights};
    if crate::d3d::lock_texture::lock_texture_in(
        gfx,
        shared.replay_file().viewer_panel_texture.get(),
    )?
    .carry
    {
        return Ok(());
    }
    // The surface cleared (as `rep stosd` row by row). A height of 0 is refused: the
    // original goes round 2^32 times.
    let height = gfx.drawing.draw_surface.height.get();
    if height == 0 {
        return Err(crate::errors::BadInput("a panel texture of no rows").into());
    }
    crate::gfx::rows::clear_surface_rows(0, height, gfx)?;
    // A string of the game's texts drawn with a font at (x, y).
    let string =
        |id: u32, font: crate::gfx::typefaces::FontId, x: i32, y: i32| -> Result<(), Refused> {
            crate::text::draw_string_by_id::draw_string_by_id_at(
                so,
                id,
                font,
                x,
                y,
                &mut GameCanvas::new(gfx),
            )?;
            Ok(())
        };
    // The text buffer drawn with a font at (x, y).
    let text = |font: crate::gfx::typefaces::FontId, x: i32, y: i32| -> Result<(), Refused> {
        let font = font.object(&gfx.fonts.typefaces);
        let mut copy = font.get();
        crate::text::draw_text_keep_regs::draw_text_keep_regs(
            &mut copy,
            &crate::cells::StoreBytes(&text_stores.buffer.text),
            x,
            y,
            &mut GameCanvas::new(gfx),
        )?;
        if copy != font.get() {
            font.set(copy);
        }
        Ok(())
    };
    // The time shown, the replay's length.
    let append_time = |hundredths: u32| {
        let time = crate::strings::append_time::append_time(
            hundredths,
            crate::strings::append_time::show_hours(),
        );
        for (n, &b) in time.iter().enumerate() {
            text_stores.buffer.text.set(n, b);
        }
        text_stores.buffer.text.set(time.len(), 0);
    };
    string(0x1A4, FONT_A, 4, 0)?;
    append_time(shown_time(
        replay.replay_view().replay_block.get(),
        replay.replay_view().replay_frame.get(),
    ));
    text(FONT_B, 4, 8)?;
    string(0x1A5, FONT_C, 0x60, 0x19)?;
    append_time(replay_length(shared.replay_file().index_block_count.get()));
    text(FONT_D, 0x60, 0x21)?;
    // The two cameras' names, three times each, 0xD lower and 0x78 to the left.
    let mut y = 0x32;
    for byte in 0..2usize {
        let camera = camera_name(crate::race::viewer::camera_byte_at(
            race,
            byte.wrapping_add(8 * replay.replay_view().replay_block.get() as usize),
        )?);
        let id = race.viewer_panel().camera_name(camera as usize);
        let mut out = 0usize;
        crate::strings::append_string_by_id::append_string_by_id(
            so,
            u32::from(id),
            &mut |bytes| {
                for (k, &b) in bytes.iter().enumerate() {
                    text_stores.buffer.text.set(out + k, b);
                }
                out += bytes.len();
            },
        )?;
        text_stores.buffer.text.set(out, 0);
        if byte == 1 {
            y += 0xD;
        }
        text(FONT_A, 4, y)?;
        let mut x = 4i32;
        for _ in 0..2 {
            y += 0xD;
            x -= 0x78;
            text(FONT_A, x, y)?;
        }
    }
    string(0x560, FONT_E, 0x75, 0)?;
    string(0x561, FONT_E, 0x75, 0xF)?;
    crate::d3d::unlock_texture::unlock_texture(shared.replay_file().viewer_panel_texture.get())?;
    crate::d3d::copy_to_device_texture::copy_to_device_texture(
        shared.replay_file().viewer_panel_texture.get(),
        shared.replay_file().viewer_device_texture.get(),
    );
    crate::d3d::set_texture::set_texture(
        crate::workers::game_scratch(),
        shared.replay_file().viewer_device_texture.get(),
    );
    // The quads.
    let camera_5 = crate::race::viewer::camera_byte_at(
        race,
        5usize.wrapping_add(8 * replay.replay_view().replay_block.get() as usize),
    )?;
    let (first, second) = switch_lights(camera_5);
    let white = 0x00FF_FFFF;
    let quad = |colour: u32| {
        crate::drive::tyre_quad::tyre_quad(
            race.viewer_panel()
                .texture_rect
                .each_ref()
                .map(crate::session::Dword::get),
            &race.viewer_panel().screen_rect,
            colour,
            race.viewer_panel(),
            system,
            crate::workers::game_scratch().vertices,
        )
    };
    // `texture_rect`: the texture's rectangle; `screen_rect`: the screen's.
    let (uv, rect) = (
        &race.viewer_panel().texture_rect,
        &race.viewer_panel().screen_rect,
    );
    let set = |cells: &[Dword; 4], v: [u32; 4]| {
        for (cell, x) in cells.iter().zip(v) {
            cell.set(x);
        }
    };
    let add = |cell: &Dword, v: u32| cell.set(cell.get().wrapping_add(v));
    set(uv, [0, 0, 0xC8, 0x32]);
    set(rect, [4, 2, 0x64, 0x19]);
    quad(white)?;
    uv[1].set(0x32);
    race.viewer_panel().screen_rect[0].set(0x2B8);
    quad(white)?;
    set(uv, [0xDC, 0, 0x1A, 0x1A]);
    set(rect, [1, 0x20, 0xD, 0xD]);
    quad(first)?;
    uv[1].set(0x1E);
    race.viewer_panel().screen_rect[0].set(0x191);
    quad(second)?;
    set(uv, [0, 0x64, 0x100, 0x1A]);
    set(rect, [0x11, 0x20, 0x80, 0xD]);
    quad(first)?;
    for _ in 0..2 {
        add(&uv[1], 0x1A);
        add(&rect[0], 0x78);
        quad(first)?;
    }
    uv[1].set(0xB2);
    race.viewer_panel().screen_rect[0].set(0x1A1);
    quad(second)?;
    for _ in 0..2 {
        add(&uv[1], 0x1A);
        add(&rect[0], 0x78);
        quad(second)?;
    }
    Ok(())
}
