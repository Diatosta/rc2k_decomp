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
use crate::net::lobby::PALETTE;
use crate::net::owner::Net;
use crate::net::store::NetVars;
use rc2k_core::net::screens::{self, ButtonFont};

pub const FONT_PLAIN: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::BUTTON;
pub const FONT_LIT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::BUTTON_LIT;
const FONT_DIM: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::BUTTON_DIM;

/// The font of item `item`'s text: lit when it is the highlighted item and being edited
/// or in the blink's lit half.
pub fn item_font(
    vars: &NetVars,
    item: u32,
    plain: crate::gfx::typefaces::FontId,
    lit: crate::gfx::typefaces::FontId,
) -> crate::gfx::typefaces::FontId {
    // Converted: `rc2k_core::net::screens::{item_lit, blink_lit}`.
    let blink = || {
        screens::blink_lit(
            crate::install::owner::install().pages.frame_clock.get(),
            crate::install::owner::install().pages.move_clock.get(),
        )
    };
    if screens::item_lit(
        item,
        crate::install::owner::install().pages.choice.get(),
        || vars.edited.get(),
        blink,
    ) {
        lit
    } else {
        plain
    }
}

/// The Rust version.
pub fn draw_button(
    net: &Net,
    string: u32,
    flags: u32,
    x: u32,
    y: u32,
    item: u32,
) -> Result<(), Refused> {
    use crate::text::draw_text::GameCanvas;
    net.button.text.set(string);
    net.button.flags.set(flags);
    net.button.x.set(x);
    net.button.y.set(y);
    net.button.item.set(item);
    let measured = crate::text::measure_string_by_id::measure_string_by_id_at(
        string,
        PALETTE.game_ptr(),
        x as i32,
        &mut GameCanvas,
    )?;
    net.button.width.set(measured.width);
    // Converted: `rc2k_core::net::screens` (`button`, `item_lit`, `button_font`).
    let place = screens::button(net.button.width.get(), net.button.x.get(), flags);
    net.button.tiles.set(place.tiles);
    if flags & screens::CENTRED != 0 {
        net.button.x.set(place.text_x);
    }
    let blit = |glyph: u32, at: u32| -> Result<(), Refused> {
        crate::net::lobby::blit(glyph, at, net.button.y.get())
    };
    let mut left = place.left;
    blit(screens::LEFT_END, left)?;
    left = left.wrapping_add(screens::END_WIDTH);
    let kept = left;
    let mut tiles = net.button.tiles.get();
    loop {
        blit(screens::TILE, left)?;
        left = left.wrapping_add(screens::TILE_WIDTH);
        tiles = tiles.wrapping_sub(1);
        if tiles == 0 {
            break;
        }
    }
    blit(
        screens::RIGHT_END,
        kept.wrapping_add(net.button.width.get()),
    )?;
    let lit = screens::item_lit(
        item,
        crate::install::owner::install().pages.choice.get(),
        || net.vars.edited.get(),
        || {
            screens::blink_lit(
                crate::install::owner::install().pages.frame_clock.get(),
                crate::install::owner::install().pages.move_clock.get(),
            )
        },
    );
    let font = match screens::button_font(flags, lit) {
        ButtonFont::Plain => FONT_PLAIN,
        ButtonFont::Lit => FONT_LIT,
        ButtonFont::Dim => FONT_DIM,
    };
    crate::text::draw_string_by_id::draw_string_by_id_at(
        net.button.text.get(),
        font.game_ptr(),
        net.button.x.get() as i32,
        net.button.y.get() as i32,
        &mut GameCanvas,
    )?;
    Ok(())
}
