//! The menu's buttons: a left end, a run of 32-pixel middle pieces and a right end, all sprites of
//! `g_spriteTable`. Three functions of the game draw them, alike but for the sprites they take, the
//! work variables they use and two slips (see `Kind`); and five small ones choose among them for a
//! menu item.
//!
//! A button is drawn from an x by its alignment: 2 = x is its left edge, 3 = x is where its
//! right end piece goes, 1 = x is its middle. Its width is that of the middle pieces. Any
//! other alignment is fatal error 0x8070202n "Invalid Active Vbutt Align Tag", and a width of
//! 0 is fatal error 0x80701010 (after the first piece is drawn).
//! The sprites come from the five dwords the item's +0x20 points at: +0 and +4 the plain
//! ends, +8 and +0xC the highlighted ends, +0x10 the middle. Every piece is blitted with the
//! button's x as its palette (leftover ebx): these sprites do not use one.

use crate::errors::{BadInput, FatalError, Refused};
use crate::gfx::blit_bar::BAR_HAS_NO_LENGTH;
use crate::gfx::blit_glyph::BlitCall;
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::item_sprites_store::{ITEM_SPRITES, ItemSprite};
use crate::menu::loop_store::LOOP_FLAGS;
use crate::session::Dword;
use crate::text::draw_text::Canvas;
use crate::text::measure_string_by_id::measure_string_by_id_at;

/// One of the three button functions.
pub struct Kind {
    /// Which of the item's five sprite numbers the two ends are.
    left_at: usize,
    right_at: usize,
    width: &'static Dword,
    x: &'static Dword,
    y: &'static Dword,
    left: &'static Dword,
    right: &'static Dword,
    /// `draw_button` only. Right-aligned, it goes wrong in two places: from the second whole
    /// middle piece on it continues in the centred branch's loop, which draws rightwards and
    /// ends with a second right end; and when the width is a multiple of 32 it falls through
    /// into the centred branch after the left end, drawing a second, centred button.
    slips: bool,
    /// `draw_button_alt` only. Left-aligned, two of its blits still read `draw_button`'s work
    /// variables: the whole middle pieces take their y from `dword_7559B4`, and the last,
    /// shorter one takes its x from `dword_7559B0`. Its one caller never asks for that
    /// alignment.
    borrows: bool,
    bad_align: FatalError,
}

const BAD_ALIGN: &str = "Invalid Active Vbutt Align Tag";

/// `screens::draw_button` (0x4688A1).
pub const PLAIN: Kind = Kind {
    left_at: 0,
    right_at: 1,
    width: &DRAW_WORK.seconds,
    x: &DRAW_WORK.work_b0,
    y: &DRAW_WORK.work_b4,
    left: &DRAW_WORK.glyph,
    right: &DRAW_WORK.work_c4,
    slips: true,
    borrows: false,
    bad_align: FatalError {
        code: 0x8070_2021,
        message: BAD_ALIGN,
    },
};

/// `screens::draw_button_highlighted` (0x468BBE).
pub const HIGHLIGHTED: Kind = Kind {
    left_at: 2,
    right_at: 3,
    slips: false,
    bad_align: FatalError {
        code: 0x8070_2022,
        message: BAD_ALIGN,
    },
    ..PLAIN
};

/// `screens::draw_button_alt` (0x468EDD).
pub const ALT: Kind = Kind {
    left_at: 0,
    right_at: 1,
    width: &DRAW_WORK.figure,
    x: &DRAW_WORK.hhmm,
    y: &DRAW_WORK.work_f4,
    left: &DRAW_WORK.work_00,
    right: &DRAW_WORK.work_04,
    slips: false,
    borrows: true,
    bad_align: FatalError {
        code: 0x8070_2023,
        message: BAD_ALIGN,
    },
};

// Menu item fields the button functions read.

pub const ALIGN_CENTRE: u32 = 1;
pub const ALIGN_RIGHT: u32 = 3;

// ---- Rust version of the three button functions ----

/// What the three functions do: the item is only read for its sprites (`set`, its +0x20).
pub fn draw_button(
    kind: &Kind,
    width: u32,
    x: i32,
    y: i32,
    align: u32,
    set: ItemSprite,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let list = match set {
        ItemSprite::List(n) => &ITEM_SPRITES.lists[n],
        ItemSprite::Sprite(0) => return Err(BadInput("a button without sprites").into()),
        // The original reads the five numbers at the sprite number as at an address.
        ItemSprite::Sprite(_) => return Err(BadInput("a button's sprites at no list").into()),
    };
    let sprites = [
        list[kind.left_at].get(),
        list[kind.right_at].get(),
        list[4].get(),
    ];
    kind.width.set(width);
    kind.x.set(x as u32);
    kind.y.set(y as u32);
    kind.left.set(sprites[0]);
    kind.right.set(sprites[1]);
    DRAW_WORK.middle.set(sprites[2]);
    // Converted: `rc2k_core::screens::buttons::draw`; the pieces blitted from the sprite table.
    use rc2k_core::screens::buttons::{Borrowed, Error};
    let borrowed = Borrowed {
        y: PLAIN.y.get() as i32,
        x: PLAIN.x.get() as i32,
    };
    let core = rc2k_core::screens::buttons::Kind {
        slips: kind.slips,
        borrows: kind.borrows,
    };
    let (end, drawn) = rc2k_core::screens::buttons::draw(
        core,
        width,
        x,
        y,
        align,
        sprites,
        borrowed,
        &mut |glyph, x, y| canvas.blit_glyph(BlitCall::sprite(glyph, x, y)),
    );
    kind.x.set(end as u32);
    match drawn {
        Ok(()) => Ok(()),
        Err(Error::NoLength) => Err(BAR_HAS_NO_LENGTH.into()),
        Err(Error::BadAlign) => Err(kind.bad_align.into()),
        Err(Error::Draw(e)) => Err(e),
    }
}

// ---- Rust version of the five that choose for an item ----

/// `screens::draw_item_button` (0x469283): the button an item describes itself (width at
/// +8, alignment at +0xC), highlighted if the item is.
pub fn draw_item_button(
    item: &crate::menu::menus_store::MenuItem,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let highlighted = LOOP_FLAGS.item.get() == item.id.get();
    draw_button(
        if highlighted { &HIGHLIGHTED } else { &PLAIN },
        item.text_x.get(),
        item.x.get() as i32,
        item.y.get() as i32,
        item.text_y.get(),
        item.sprites.get(),
        canvas,
    )
}

/// The other four: a button as wide as the item's text, measured with the item's font at
/// +0x4C, plus 12. The original measures from whatever x its caller left in ecx; a measured
/// width does not depend on it, so it is measured from 0. Refused: a measuring font that
/// does not only measure (it would draw the text at that x).
pub fn draw_text_button(
    kind: &Kind,
    align: u32,
    item: &crate::menu::menus_store::MenuItem,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let font = crate::text::measure_text::only_measures(item.measuring_font.get())?;
    let measured = measure_string_by_id_at(item.string.get(), font, 0, canvas)?;
    draw_button(
        kind,
        rc2k_core::screens::buttons::text_width(measured.width),
        item.x.get() as i32,
        item.y.get() as i32,
        align,
        item.sprites.get(),
        canvas,
    )
}
