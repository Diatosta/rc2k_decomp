//! `race::draw_viewer_panel` (0x41DA25): the replay viewer's panel: its texture
//! (`dword_5BECB8`) locked (`d3d::lock_texture`; failing, nothing), cleared, and written
//! on: text 0x1A4 and the time at the frame shown (block * 100 + frame, `strings::append_time`),
//! text 0x1A5 and the replay's length, the names of the block's two cameras
//! (`word_5C291C` by its bytes 0 and 1, at most 0x2E; `strings::append_string_by_id`),
//! each three times 0x78 to the left and 0xD down, texts 0x560 and 0x561 (the fonts at
//! `stru_51EAE4` ..; `text::draw_text_keep_regs`, `sub_402368`); unlocked
//! (`d3d::unlock_texture`), copied to its device texture (`dword_5BECBC`, `sub_41A8F9`),
//! made the texture (`d3d::set_texture`) and drawn as ten quads (`drive::tyre_quad`, the
//! texture's and the screen's rectangles `dword_5C2A1C`, `dword_5C2A0C`), the switch lights
//! dimmed (0x555555) by the block's camera byte 5 (0: the second, 1: the first).
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! Caller: `race::viewer_frame`.

use crate::errors::Refused;
use crate::gfx::screen_state::DRAW_SURFACE;
use crate::pointers::GamePtr;
use crate::race::replay_file_store::REPLAY_FILE;
use crate::race::viewer_screen_store::VIEWER_PANEL;
use crate::race::viewer_state_store::REPLAY_VIEW;
use crate::session::Dword;
use crate::strings::text_buffer::TEXT_BUFFER;
use crate::text::draw_text::GameCanvas;
const FONT_A: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::PANEL_A;
const FONT_B: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::PANEL_B;
const FONT_C: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::PANEL_C;
const FONT_D: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::PANEL_D;
const FONT_E: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::PANEL_E;

/// The Rust version (the original keeps every register of its caller).
pub fn draw_viewer_panel() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::viewer_flow::{shown_time, replay_length, camera_name,
    // switch_lights}`.
    use rc2k_core::race::viewer_flow::{camera_name, replay_length, shown_time, switch_lights};
    if crate::d3d::lock_texture::lock_texture(REPLAY_FILE.viewer_panel_texture.get())?.carry {
        return Ok(());
    }
    // The surface cleared (as `rep stosd` row by row; a height of 0 goes round 2^32
    // times in the original as here: the texture has rows).
    let mut at = crate::gfx::surface::address()?;
    let mut rows = DRAW_SURFACE.height.get();
    loop {
        let dwords = DRAW_SURFACE.row_dwords.get();
        crate::pointers::GamePtr::new(at).fill(0, 4 * dwords as usize, "the panel's surface")?;
        at = at
            .wrapping_add(4 * dwords)
            .wrapping_add(DRAW_SURFACE.row_gap.get());
        rows = rows.wrapping_sub(1);
        if rows == 0 {
            break;
        }
    }
    // A string of the game's texts drawn with a font at (x, y).
    let string =
        |id: u32, font: crate::gfx::typefaces::FontId, x: i32, y: i32| -> Result<(), Refused> {
            crate::text::draw_string_by_id::draw_string_by_id_at(
                id,
                font.as_ptr(),
                x,
                y,
                &mut GameCanvas,
            )?;
            Ok(())
        };
    // The text buffer drawn with a font at (x, y).
    let text = |font: crate::gfx::typefaces::FontId, x: i32, y: i32| -> Result<(), Refused> {
        let font = GamePtr::new(font.addr());
        let mut copy: crate::text::font::Font = font.read("a font record")?;
        crate::text::draw_text_keep_regs::draw_text_keep_regs(
            &mut copy,
            TEXT_BUFFER.text.as_ptr() as *const u8,
            x,
            y,
            &mut GameCanvas,
        )?;
        if copy != font.read::<crate::text::font::Font>("a font record")? {
            font.write(copy, "a font record")?;
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
            TEXT_BUFFER.text.set(n, b);
        }
        TEXT_BUFFER.text.set(time.len(), 0);
    };
    string(0x1A4, FONT_A, 4, 0)?;
    append_time(shown_time(
        REPLAY_VIEW.replay_block.get(),
        REPLAY_VIEW.replay_frame.get(),
    ));
    text(FONT_B, 4, 8)?;
    string(0x1A5, FONT_C, 0x60, 0x19)?;
    append_time(replay_length(REPLAY_FILE.index_block_count.get()));
    text(FONT_D, 0x60, 0x21)?;
    // The two cameras' names, three times each, 0xD lower and 0x78 to the left.
    let mut y = 0x32;
    for byte in 0..2usize {
        let camera = camera_name(crate::race::viewer::camera_byte_at(
            byte.wrapping_add(8 * REPLAY_VIEW.replay_block.get() as usize),
        )?);
        let id = crate::race::viewer_screen_store::VIEWER_PANEL.camera_name(camera as usize);
        let mut out = 0usize;
        crate::strings::append_string_by_id::append_string_by_id(u32::from(id), &mut |bytes| {
            for (k, &b) in bytes.iter().enumerate() {
                TEXT_BUFFER.text.set(out + k, b);
            }
            out += bytes.len();
        })?;
        TEXT_BUFFER.text.set(out, 0);
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
    crate::d3d::unlock_texture::unlock_texture(REPLAY_FILE.viewer_panel_texture.get())?;
    crate::d3d::copy_to_device_texture::copy_to_device_texture(
        REPLAY_FILE.viewer_panel_texture.get(),
        REPLAY_FILE.viewer_device_texture.get(),
    );
    crate::d3d::set_texture::set_texture(REPLAY_FILE.viewer_device_texture.get());
    // The quads.
    let camera_5 = crate::race::viewer::camera_byte_at(
        5usize.wrapping_add(8 * REPLAY_VIEW.replay_block.get() as usize),
    )?;
    let (first, second) = switch_lights(camera_5);
    let white = 0x00FF_FFFF;
    let quad = |colour: u32| {
        crate::drive::tyre_quad::tyre_quad(
            VIEWER_PANEL.texture_rect[0].addr(),
            VIEWER_PANEL.screen_rect[0].addr(),
            colour,
        )
    };
    // dword_5C2A1C: the texture's rectangle; dword_5C2A0C: the screen's.
    let (uv, rect) = (&VIEWER_PANEL.texture_rect, &VIEWER_PANEL.screen_rect);
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
    VIEWER_PANEL.screen_rect[0].set(0x2B8);
    quad(white)?;
    set(uv, [0xDC, 0, 0x1A, 0x1A]);
    set(rect, [1, 0x20, 0xD, 0xD]);
    quad(first)?;
    uv[1].set(0x1E);
    VIEWER_PANEL.screen_rect[0].set(0x191);
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
    VIEWER_PANEL.screen_rect[0].set(0x1A1);
    quad(second)?;
    for _ in 0..2 {
        add(&uv[1], 0x1A);
        add(&rect[0], 0x78);
        quad(second)?;
    }
    Ok(())
}
