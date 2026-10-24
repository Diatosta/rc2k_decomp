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
use crate::framework::{game_read, game_write, rva};
use crate::gfx::blit_bar::BAR_HAS_NO_LENGTH;
use crate::gfx::blit_glyph::BlitCall;
use crate::gfx::blit_marker_pair::sprite_table;
use crate::service::state::MENU_ITEM_RVA;
use crate::text::draw_text::Canvas;
use crate::text::font::Font;
use crate::text::measure_string_by_id::measure_string_by_id_at;

/// One of the three button functions.
pub struct Kind {
    /// Where in the item's five sprite numbers the two ends are.
    left_at: u32,
    right_at: u32,
    width_rva: usize,
    x_rva: usize,
    y_rva: usize,
    left_rva: usize,
    right_rva: usize,
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

const MIDDLE_RVA: usize = rva(0x7559D8); // dword_7559D8
const BAD_ALIGN: &str = "Invalid Active Vbutt Align Tag";

/// `screens::draw_button` (0x4688A1).
pub const PLAIN: Kind = Kind {
    left_at: 0,
    right_at: 4,
    width_rva: rva(0x7559AC), // seconds
    x_rva: rva(0x7559B0),     // dword_7559B0
    y_rva: rva(0x7559B4),     // dword_7559B4
    left_rva: rva(0x7559C0),  // glyph
    right_rva: rva(0x7559C4), // dword_7559C4
    slips: true,
    borrows: false,
    bad_align: FatalError {
        code: 0x8070_2021,
        message: BAD_ALIGN,
    },
};

/// `screens::draw_button_highlighted` (0x468BBE).
pub const HIGHLIGHTED: Kind = Kind {
    left_at: 8,
    right_at: 0xC,
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
    right_at: 4,
    width_rva: rva(0x7559EC), // dword_7559EC
    x_rva: rva(0x7559F0),     // hhmm
    y_rva: rva(0x7559F4),     // dword_7559F4
    left_rva: rva(0x755A00),  // dword_755A00
    right_rva: rva(0x755A04), // dword_755A04
    slips: false,
    borrows: true,
    bad_align: FatalError {
        code: 0x8070_2023,
        message: BAD_ALIGN,
    },
};

// Menu item fields the button functions read.
const ITEM_X: u32 = 0;
const ITEM_Y: u32 = 4;
const ITEM_WIDTH: u32 = 8;
const ITEM_ALIGN: u32 = 0xC;
const ITEM_TYPE: u32 = 0x10;
const ITEM_STRING: u32 = 0x14;
const ITEM_SPRITES: u32 = 0x20;
const ITEM_MEASURING_FONT: u32 = 0x4C;

pub const ALIGN_CENTRE: u32 = 1;
pub const ALIGN_RIGHT: u32 = 3;

pub unsafe fn dword(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

// ---- Rust version of the three button functions ----

/// What the three functions do: `item` is only read for its sprites.
pub unsafe fn draw_button(
    kind: &Kind,
    width: u32,
    x: i32,
    y: i32,
    align: u32,
    item: u32,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    unsafe {
        let set = dword(item.wrapping_add(ITEM_SPRITES));
        if item == 0 || set == 0 {
            return Err(BadInput("a button without sprites").into());
        }
        let sprites = [
            dword(set + kind.left_at),
            dword(set + kind.right_at),
            dword(set + 0x10),
        ];
        game_write(kind.width_rva, width);
        game_write(kind.x_rva, x);
        game_write(kind.y_rva, y);
        game_write(kind.left_rva, sprites[0]);
        game_write(kind.right_rva, sprites[1]);
        game_write(MIDDLE_RVA, sprites[2]);
        // Converted: `rc2k_core::screens::buttons::draw`; the pieces blitted from the sprite table.
        use rc2k_core::screens::buttons::{Borrowed, Error};
        let table = sprite_table();
        let borrowed = Borrowed {
            y: game_read(PLAIN.y_rva),
            x: game_read(PLAIN.x_rva),
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
            &mut |glyph, x, y, palette| {
                canvas.blit_glyph(BlitCall::new(glyph, x, y, palette, table))
            },
        );
        game_write(kind.x_rva, end);
        match drawn {
            Ok(()) => Ok(()),
            Err(Error::NoLength) => Err(BAR_HAS_NO_LENGTH.into()),
            Err(Error::BadAlign) => Err(kind.bad_align.into()),
            Err(Error::Draw(e)) => Err(e.into()),
        }
    }
}

// ---- Rust version of the five that choose for an item ----

/// `screens::draw_item_button` (0x469283): the button an item describes itself (width at
/// +8, alignment at +0xC), highlighted if the item is.
pub unsafe fn draw_item_button(item: u32, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    if item == 0 {
        return Err(BadInput("no menu item").into());
    }
    unsafe {
        let highlighted = game_read::<u32>(MENU_ITEM_RVA) == dword(item + ITEM_TYPE);
        draw_button(
            if highlighted { &HIGHLIGHTED } else { &PLAIN },
            dword(item + ITEM_WIDTH),
            dword(item + ITEM_X) as i32,
            dword(item + ITEM_Y) as i32,
            dword(item + ITEM_ALIGN),
            item,
            canvas,
        )
    }
}

/// The other four: a button as wide as the item's text, measured with the item's font at
/// +0x4C, plus 12. `measure_x` is the x the measuring is given: the caller's ecx.
pub unsafe fn draw_text_button(
    kind: &Kind,
    align: u32,
    item: u32,
    measure_x: i32,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    if item == 0 {
        return Err(BadInput("no menu item").into());
    }
    unsafe {
        let font = dword(item + ITEM_MEASURING_FONT) as *mut Font;
        let measured = measure_string_by_id_at(dword(item + ITEM_STRING), font, measure_x, canvas)?;
        draw_button(
            kind,
            rc2k_core::screens::buttons::text_width(measured.width),
            dword(item + ITEM_X) as i32,
            dword(item + ITEM_Y) as i32,
            align,
            item,
            canvas,
        )
    }
}
