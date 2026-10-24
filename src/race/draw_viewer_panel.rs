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
//! Caller: `race::viewer_frame`. Its calls through `part` (`CALLEES`; `strings::append_nul`
//! written inline); in the game their Rust versions, the texture's copy (Direct3D) and the
//! texture's setting through the game. Shadow mode runs the original alone (it draws).
//! Self-test: a copy whose calls go to stand-ins, against the Rust version, a surface of the
//! test's locked: camera bytes 5 of 0, 1 and 2, names in and past the table, the lock
//! failing.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::physics::flow::{a, g, p};
use crate::race::viewer::{BLOCK, BLOCKS, CAMERA_NAMES, CAMERAS, FRAME, gb};
use crate::text::draw_text::GameCanvas;
use crate::text::font::Font;

const TEXTURE: usize = 0x5BECB8; // dword_5BECB8: the panel's texture
const DEVICE_TEXTURE: usize = 0x5BECBC; // dword_5BECBC
const TEXT: usize = 0x93E430; // text: the game's text buffer
const FONT_A: usize = 0x51EAE4; // stru_51EAE4
const FONT_B: usize = 0x51EB44; // stru_51EB44
const FONT_C: usize = 0x51EB04; // unk_51EB04
const FONT_D: usize = 0x51EB64; // stru_51EB64
const FONT_E: usize = 0x51EAC4; // unk_51EAC4
const UV: usize = 0x5C2A1C; // dword_5C2A1C: the texture's rectangle
const RECT: usize = 0x5C2A0C; // dword_5C2A0C: the screen's
const PIXELS: usize = 0x4F49C0; // g_surface.pixels
const HEIGHT: usize = 0x4F49A4; // g_surface.height
const ROW_DWORDS: usize = 0x4F49B4; // g_surface.row_dwords
const ROW_GAP: usize = 0x4F49C4; // g_surface.row_gap

/// The Rust version (the original keeps every register of its caller).
pub fn draw_viewer_panel() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::viewer_flow::{shown_time, replay_length, camera_name,
    // switch_lights}`.
    use rc2k_core::race::viewer_flow::{camera_name, replay_length, shown_time, switch_lights};
    unsafe {
        if crate::d3d::lock_texture::lock_texture(g(TEXTURE))?.carry {
            return Ok(());
        }
        // The surface cleared (as `rep stosd` row by row; a height of 0 goes round 2^32
        // times in the original as here: the texture has rows).
        let mut at = g(PIXELS);
        let mut rows = g(HEIGHT);
        loop {
            let dwords = g(ROW_DWORDS);
            std::ptr::write_bytes(at as *mut u32, 0, dwords as usize);
            at = at.wrapping_add(4 * dwords).wrapping_add(g(ROW_GAP));
            rows = rows.wrapping_sub(1);
            if rows == 0 {
                break;
            }
        }
        // A string of the game's texts drawn with a font at (x, y).
        let string = |id: u32, font: usize, x: i32, y: i32| -> Result<(), Refused> {
            crate::text::draw_string_by_id::draw_string_by_id_at(
                id,
                a(font) as *mut Font,
                x,
                y,
                &mut GameCanvas,
            )?;
            Ok(())
        };
        // The text buffer drawn with a font at (x, y).
        let text = |font: usize, x: i32, y: i32| -> Result<(), Refused> {
            let font = a(font) as *mut Font;
            let mut copy = font.read_unaligned();
            crate::text::draw_text_keep_regs::draw_text_keep_regs(
                &mut copy,
                a(TEXT) as *const u8,
                x,
                y,
                &mut GameCanvas,
            )?;
            if copy != font.read_unaligned() {
                font.write_unaligned(copy);
            }
            Ok(())
        };
        // The text buffer's end, a NUL put.
        let nul = |end: *mut u8| end.write(0);
        // The time shown, the replay's length.
        let append_time = |hundredths: u32| {
            let time = crate::strings::append_time::append_time(
                hundredths,
                crate::strings::append_time::show_hours(),
            );
            std::ptr::copy_nonoverlapping(time.as_ptr(), a(TEXT) as *mut u8, time.len());
            nul((a(TEXT) as *mut u8).add(time.len()));
        };
        string(0x1A4, FONT_A, 4, 0)?;
        append_time(shown_time(g(BLOCK), g(FRAME)));
        text(FONT_B, 4, 8)?;
        string(0x1A5, FONT_C, 0x60, 0x19)?;
        append_time(replay_length(g(BLOCKS)));
        text(FONT_D, 0x60, 0x21)?;
        // The two cameras' names, three times each, 0xD lower and 0x78 to the left.
        let mut y = 0x32;
        for byte in 0..2usize {
            let camera = camera_name(gb((CAMERAS + byte).wrapping_add(8 * g(BLOCK) as usize)));
            let id = (a(CAMERA_NAMES + 2 * camera as usize) as *const u16).read_unaligned();
            let mut out = a(TEXT) as *mut u8;
            crate::strings::append_string_by_id::append_string_by_id(
                u32::from(id),
                &mut |from, n| {
                    std::ptr::copy_nonoverlapping(from, out, n);
                    out = out.add(n);
                },
            );
            nul(out);
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
        crate::d3d::unlock_texture::unlock_texture(g(TEXTURE))?;
        crate::d3d::copy_to_device_texture::copy_to_device_texture(g(TEXTURE), g(DEVICE_TEXTURE));
        crate::d3d::set_texture::set_texture(g(DEVICE_TEXTURE));
        // The quads.
        let camera_5 = gb((CAMERAS + 5).wrapping_add(8 * g(BLOCK) as usize));
        let (first, second) = switch_lights(camera_5);
        let white = 0x00FF_FFFF;
        let quad = |colour: u32| crate::drive::tyre_quad::tyre_quad(a(UV), a(RECT), colour);
        let set = |at: usize, v: [u32; 4]| {
            for (n, x) in v.into_iter().enumerate() {
                p(at + 4 * n, x);
            }
        };
        let add = |at: usize, v: u32| p(at, g(at).wrapping_add(v));
        set(UV, [0, 0, 0xC8, 0x32]);
        set(RECT, [4, 2, 0x64, 0x19]);
        quad(white)?;
        p(UV + 4, 0x32);
        p(RECT, 0x2B8);
        quad(white)?;
        set(UV, [0xDC, 0, 0x1A, 0x1A]);
        set(RECT, [1, 0x20, 0xD, 0xD]);
        quad(first)?;
        p(UV + 4, 0x1E);
        p(RECT, 0x191);
        quad(second)?;
        set(UV, [0, 0x64, 0x100, 0x1A]);
        set(RECT, [0x11, 0x20, 0x80, 0xD]);
        quad(first)?;
        for _ in 0..2 {
            add(UV + 4, 0x1A);
            add(RECT, 0x78);
            quad(first)?;
        }
        p(UV + 4, 0xB2);
        p(RECT, 0x1A1);
        quad(second)?;
        for _ in 0..2 {
            add(UV + 4, 0x1A);
            add(RECT, 0x78);
            quad(second)?;
        }
    }
    Ok(())
}
