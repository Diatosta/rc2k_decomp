//! The menu's buttons: a left end, a run of 32-pixel middle pieces and a right end, all sprites
//! of the sprite table. Three drawers differ in the sprites they take, whether they leave their
//! place behind and what they leave behind (see `Kind`); five small ones choose among them for a menu item.
//!
//! A button is drawn from an x by its alignment: 2 = x is its left edge, 3 = x is where its right
//! end piece goes, 1 = x is its middle. Any other alignment is fatal error 0x8070202n "Invalid
//! Active Vbutt Align Tag", and a middle width of 0 is fatal error 0x80701010 (after the first
//! piece is drawn). The item's sprites are the plain ends, the highlighted ends and the middle.
//! The palette passed with every piece is the button's x: these sprites do not use one.

use crate::errors::{BadInput, FatalError, Refused};
use crate::gfx::blit_bar::BAR_HAS_NO_LENGTH;
use crate::gfx::blit_glyph::BlitCall;
use crate::menu::item_sprites_store::{ITEM_SPRITES, ItemSprite};
use crate::menu::owner::Menus;
use crate::text::draw_text::Canvas;
use crate::text::measure_string_by_id::measure_string_by_id_at;

/// One of the three button functions.
pub struct Kind {
    /// Which of the item's five sprite numbers the two ends are.
    left_at: usize,
    right_at: usize,
    /// The plain and highlighted buttons leave their place in `DrawWork.button_x` / `button_y`.
    keeps_place: bool,
    /// `ALT` only. Left-aligned, two of its blits still read the plain button's place: the whole
    /// middle pieces take their y from `DrawWork.button_y`, and the last, shorter one takes its x
    /// from `DrawWork.button_x`. Nothing draws it left-aligned.
    borrows: bool,
    bad_align: FatalError,
}

const BAD_ALIGN: &str = "Invalid Active Vbutt Align Tag";

/// The plain button.
pub const PLAIN: Kind = Kind {
    left_at: 0,
    right_at: 1,
    keeps_place: true,
    borrows: false,
    bad_align: FatalError {
        code: 0x8070_2021,
        message: BAD_ALIGN,
    },
};

/// The highlighted button.
pub(super) const HIGHLIGHTED: Kind = Kind {
    left_at: 2,
    right_at: 3,
    bad_align: FatalError {
        code: 0x8070_2022,
        message: BAD_ALIGN,
    },
    ..PLAIN
};

/// The alternative button, on its own work variables (but see `borrows`).
pub const ALT: Kind = Kind {
    left_at: 0,
    right_at: 1,
    keeps_place: false,
    borrows: true,
    bad_align: FatalError {
        code: 0x8070_2023,
        message: BAD_ALIGN,
    },
};

// Menu item fields the button functions read.

pub const ALIGN_CENTRE: u32 = 1;
pub const ALIGN_RIGHT: u32 = 3;

// ---- the three button functions ----

/// What the three functions do: the item is only read for its sprites (`set`, its `sprites`).
pub fn draw_button(
    menus: &Menus,
    kind: &Kind,
    width: u32,
    (x, y): (i32, i32),
    align: u32,
    set: ItemSprite,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let list = match set {
        ItemSprite::List(n) => &ITEM_SPRITES[n],
        ItemSprite::Sprite(0) => return Err(BadInput("a button without sprites").into()),
        // A sprite number where the item should point at a list of five is refused.
        ItemSprite::Sprite(_) => return Err(BadInput("a button's sprites at no list").into()),
    };
    let sprites = [list[kind.left_at], list[kind.right_at], list[4]];
    let place = &menus.screens.draw_work;
    if kind.keeps_place {
        place.button_x.set(x as u32);
        place.button_y.set(y as u32);
    }
    // The logic is in `rc2k_core::screens::buttons::draw`; the pieces blitted from the sprite
    // table.
    use rc2k_core::screens::buttons::{Borrowed, Error};
    let borrowed = Borrowed {
        y: place.button_y.get() as i32,
        x: place.button_x.get() as i32,
    };
    let core = rc2k_core::screens::buttons::Kind {
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
        &mut |glyph, x, y| {
            let call = BlitCall::sprite(menus, glyph, x, y, canvas.clip_right());
            canvas.blit_glyph(call)
        },
    );
    if kind.keeps_place {
        place.button_x.set(end as u32);
    }
    match drawn {
        Ok(()) => Ok(()),
        Err(Error::NoLength) => Err(BAR_HAS_NO_LENGTH.into()),
        Err(Error::BadAlign) => Err(kind.bad_align.into()),
        Err(Error::Draw(e)) => Err(e),
    }
}

// ---- the five that choose for an item ----

/// The button an item describes itself (`width_or_text_x`, `alignment_or_text_y`), highlighted if the
/// item is.
pub(super) fn draw_item_button(
    menus: &Menus,
    item: &crate::menu::menus_store::MenuItem,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let highlighted = menus.screens.loop_flags.item.get() == item.fixed().id;
    draw_button(
        menus,
        if highlighted { &HIGHLIGHTED } else { &PLAIN },
        item.fixed().width_or_text_x as u32,
        (item.fixed().x, item.fixed().y),
        item.fixed().alignment_or_text_y as u32,
        item.sprites.get(),
        canvas,
    )
}

/// The other four: a button as wide as the item's text (measured with the item's font) plus
/// 12. The text is measured from x 0, as its width does not depend on x. Refused: a measuring
/// font that does not only measure (it would draw the text at that x).
pub(super) fn draw_text_button(
    o: crate::owners::Owners<'static>,
    kind: &Kind,
    align: u32,
    item: &crate::menu::menus_store::MenuItem,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    let menus = o.menus;
    let font = crate::text::measure_text::only_measures(
        crate::menu::menus_store::font_of(item.fixed().measuring_font),
        &o.gfx.fonts.typefaces,
    )?;
    let measured = measure_string_by_id_at(
        crate::strings::append_string_by_id::StringOwners::of(o),
        item.string.get(),
        font,
        0,
        canvas,
    )?;
    draw_button(
        menus,
        kind,
        rc2k_core::screens::buttons::text_width(measured.width),
        (item.fixed().x, item.fixed().y),
        align,
        item.sprites.get(),
        canvas,
    )
}
