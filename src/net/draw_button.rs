//! `net::draw_button` (0x416CE0): a button of the network screens: the string measured
//! (`text::measure_string_by_id` with `palette`), its frame drawn from the menu sprites
//! (`g_spriteTable` glyphs 0x11, 0x12 once per 32 pixels of the width, at least once, and
//! 0x13: `gfx::blit_glyph`), the string drawn on it (`text::draw_string_by_id`) in
//! `typefaces::BUTTON`, or `typefaces::BUTTON_LIT` when it is the highlighted item
//! (`g_listChoice`) and either being edited (`edited`) or in the blink's lit half (bit 3
//! of the frame clock, with the move clock clear), or `typefaces::BUTTON_DIM` with flag 2.
//!
//! Takes the string, the flags (1: the frame centred on x, else ending half its width left of x
//! and the text centred too; 2: the dim font), x, y and the item.
//! The arguments are kept in the button's variables (`text`, flags, ..), the width in
//! `width`, the frame's tiles in `tiles`. The blits get the measuring register as the
//! measuring and the blits before leave it (the sprites are drawn as they are). Callers:
//! `net::setting_label`, `net::lobby_button`.

use crate::errors::Refused;
use crate::install::owner::Install;
use crate::net::lobby::PALETTE;
use crate::net::owner::Net;
use rc2k_core::net::screens::{self, ButtonFont};

pub const FONT_PLAIN: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::BUTTON;
pub const FONT_LIT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::BUTTON_LIT;
const FONT_DIM: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::BUTTON_DIM;

/// The font of item `item`'s text: lit when it is the highlighted item and being edited
/// or in the blink's lit half.
pub fn item_font(
    net: &Net,
    install: &Install,
    item: u32,
    plain: crate::gfx::typefaces::FontId,
    lit: crate::gfx::typefaces::FontId,
) -> crate::gfx::typefaces::FontId {
    // Converted: `rc2k_core::net::screens::{item_lit, blink_lit}`.
    let blink = || {
        screens::blink_lit(
            install.pages.frame_clock.get(),
            install.pages.move_clock.get(),
        )
    };
    if screens::item_lit(
        item,
        install.pages.choice.get(),
        || net.edit().edited.get(),
        blink,
    ) {
        lit
    } else {
        plain
    }
}
#[allow(
    clippy::too_many_arguments,
    reason = "the button's string, flags, place and item, and the owners"
)]
pub fn draw_button(
    so: crate::strings::append_string_by_id::StringOwners<'static>,
    net: &Net,
    install: &Install,
    string: u32,
    flags: u32,
    x: u32,
    y: u32,
    item: u32,
    gfx: &crate::gfx::owner::Gfx,
    menus: &crate::menu::owner::Menus,
) -> Result<(), Refused> {
    use crate::text::draw_text::GameCanvas;
    net.button.get().text.set(string);
    net.button.get().flags.set(flags);
    net.button.get().x.set(x);
    net.button.get().y.set(y);
    net.button.get().item.set(item);
    let measured = crate::text::measure_string_by_id::measure_string_by_id_at(
        so,
        string,
        PALETTE,
        x as i32,
        &mut GameCanvas::new(gfx),
    )?;
    net.button.get().width.set(measured.width);
    // Converted: `rc2k_core::net::screens` (`button`, `item_lit`, `button_font`).
    let place = screens::button(
        net.button.get().width.get(),
        net.button.get().x.get(),
        flags,
    );
    net.button.get().tiles.set(place.tiles);
    if flags & screens::CENTRED != 0 {
        net.button.get().x.set(place.text_x);
    }
    let blit = |glyph: u32, at: u32| -> Result<(), Refused> {
        crate::net::lobby::blit(glyph, at, net.button.get().y.get(), gfx, menus)
    };
    let mut left = place.left;
    blit(screens::LEFT_END, left)?;
    left = left.wrapping_add(screens::END_WIDTH);
    let kept = left;
    let mut tiles = net.button.get().tiles.get();
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
        kept.wrapping_add(net.button.get().width.get()),
    )?;
    let lit = screens::item_lit(
        item,
        install.pages.choice.get(),
        || net.edit().edited.get(),
        || {
            screens::blink_lit(
                install.pages.frame_clock.get(),
                install.pages.move_clock.get(),
            )
        },
    );
    let font = match screens::button_font(flags, lit) {
        ButtonFont::Plain => FONT_PLAIN,
        ButtonFont::Lit => FONT_LIT,
        ButtonFont::Dim => FONT_DIM,
    };
    crate::text::draw_string_by_id::draw_string_by_id_at(
        so,
        net.button.get().text.get(),
        font,
        net.button.get().x.get() as i32,
        net.button.get().y.get() as i32,
        &mut GameCanvas::new(gfx),
    )?;
    Ok(())
}
