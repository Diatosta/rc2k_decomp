//! `net::draw_button` (0x416CE0): a button of the network screens: string eax measured
//! (`text::measure_string_by_id` with `palette`), its frame drawn from the menu sprites
//! (`g_spriteTable` glyphs 0x11, 0x12 once per 32 pixels of the width, at least once, and
//! 0x13: `gfx::blit_glyph`), the string drawn on it (`text::draw_string_by_id`) in
//! `unk_51E704`, or `unk_51E644` when it is the highlighted item (`g_listChoice`) and either
//! being edited (`dword_52A4B8`) or in the blink's lit half (bit 3 of `dword_51D41F` -
//! `dword_51D423` clear), or `stru_51E7C4` with flag 2.
//!
//! In:  the string, the flags (1: the frame centred on x, else ending half its width left of
//!      x and the text centred too; 2: the dim font), x, y, the item
//! Out: nothing (the original keeps every register: `pusha` .. `popa`)
//! The arguments are kept in `dword_5AB490` .., the width in `dword_5AB4A0`, the frame's
//! tiles in `dword_5AB4A4`. The blits get ebx as the measuring and the blits before leave
//! it (the sprites are drawn as they are). Callers: `net::setting_label`,
//! `net::lobby_button`.

use crate::errors::Refused;
use crate::net::lobby::{CHOICE, EDITED, FRAME_CLOCK, MOVE_CLOCK, PALETTE, a, g, p};
use crate::text::font::Font;
use rc2k_core::net::screens::{self, ButtonFont};

const ARGS: usize = 0x5AB490; // dword_5AB490: the string, flags, x, y; +0x18 the item
const WIDTH: usize = 0x5AB4A0; // dword_5AB4A0
const TILES: usize = 0x5AB4A4; // dword_5AB4A4
const ITEM: usize = 0x5AB4A8; // dword_5AB4A8
pub const FONT_PLAIN: usize = 0x51E704; // unk_51E704
pub const FONT_LIT: usize = 0x51E644; // unk_51E644
const FONT_DIM: usize = 0x51E7C4; // stru_51E7C4

/// The font of item `item`'s text: lit when it is the highlighted item and being edited
/// or in the blink's lit half.
pub fn item_font(item: u32, plain: usize, lit: usize) -> u32 {
    // Converted: `rc2k_core::net::screens::{item_lit, blink_lit}`.
    let blink = || screens::blink_lit(g(FRAME_CLOCK), g(MOVE_CLOCK));
    if screens::item_lit(item, g(CHOICE), || g(EDITED), blink) {
        a(lit)
    } else {
        a(plain)
    }
}

/// The Rust version.
pub fn draw_button(string: u32, flags: u32, x: u32, y: u32, item: u32) -> Result<(), Refused> {
    use crate::text::draw_text::GameCanvas;
    unsafe {
        p(ARGS, string);
        p(ARGS + 4, flags);
        p(ARGS + 8, x);
        p(ARGS + 12, y);
        p(ITEM, item);
        let measured = crate::text::measure_string_by_id::measure_string_by_id_at(
            string,
            a(PALETTE) as *mut Font,
            x as i32,
            &mut GameCanvas,
        )?;
        p(WIDTH, measured.width);
        // Converted: `rc2k_core::net::screens` (`button`, `item_lit`, `button_font`).
        let place = screens::button(g(WIDTH), g(ARGS + 8), flags);
        p(TILES, place.tiles);
        if flags & screens::CENTRED != 0 {
            p(ARGS + 8, place.text_x);
        }
        // The blits are handed the palette the measuring and each blit before leave in ebx
        // (the sprites are drawn as they are).
        let mut palette = measured.lines;
        let mut blit = |glyph: u32, at: u32| -> Result<(), Refused> {
            palette = crate::net::lobby::blit(glyph, at, g(ARGS + 12), palette)?;
            Ok(())
        };
        let mut left = place.left;
        blit(screens::LEFT_END, left)?;
        left = left.wrapping_add(screens::END_WIDTH);
        let kept = left;
        let mut tiles = g(TILES);
        loop {
            blit(screens::TILE, left)?;
            left = left.wrapping_add(screens::TILE_WIDTH);
            tiles = tiles.wrapping_sub(1);
            if tiles == 0 {
                break;
            }
        }
        blit(screens::RIGHT_END, kept.wrapping_add(g(WIDTH)))?;
        let lit = screens::item_lit(
            item,
            g(CHOICE),
            || g(EDITED),
            || screens::blink_lit(g(FRAME_CLOCK), g(MOVE_CLOCK)),
        );
        let font = a(match screens::button_font(flags, lit) {
            ButtonFont::Plain => FONT_PLAIN,
            ButtonFont::Lit => FONT_LIT,
            ButtonFont::Dim => FONT_DIM,
        });
        crate::text::draw_string_by_id::draw_string_by_id_at(
            g(ARGS),
            font as *mut Font,
            g(ARGS + 8) as i32,
            g(ARGS + 12) as i32,
            &mut GameCanvas,
        )?;
    }
    Ok(())
}
