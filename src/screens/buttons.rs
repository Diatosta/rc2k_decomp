//! The menu's buttons: a left end, a run of 32-pixel middle pieces and a right end, all
//! sprites of `g_spriteTable`. Three functions of the game draw them, alike but for the
//! sprites they take, the work variables they use and two slips (see `Kind`); and five small
//! ones choose among them for a menu item. Their hooks are in the `draw_button*` and
//! `draw_item_button*` files; what they share is here.
//!
//! A button is drawn from an x by its alignment: 2 = x is its left edge, 3 = x is where its
//! right end piece goes, 1 = x is its middle. Its width is that of the middle pieces. Any
//! other alignment is fatal error 0x8070202n "Invalid Active Vbutt Align Tag", and a width of
//! 0 is fatal error 0x80701010 (after the first piece is drawn).
//! The sprites come from the five dwords the item's +0x20 points at: +0 and +4 the plain
//! ends, +8 and +0xC the highlighted ends, +0x10 the middle. Every piece is blitted with the
//! button's x as its palette (leftover ebx): these sprites do not use one.

use crate::errors::{BadInput, FatalError, Refused};
use crate::framework::{Regs, SelfTest, game_addr, game_read, game_write, rva};
use crate::gfx::blit_bar::BAR_HAS_NO_LENGTH;
use crate::gfx::blit_glyph::BlitCall;
use crate::gfx::blit_marker_pair::sprite_table;
use crate::screens::harness::{CallOriginal, Draw, check_draw};
use crate::service::harness::{Region, at, capture, restore};
use crate::service::state::MENU_ITEM_RVA;
use crate::strings::text_from_ids::{TEXT_LEN, text};
use crate::text::draw_text::Canvas;
use crate::text::font::Font;
use crate::text::measure_string_by_id::{MEASURING_FONT, TEST_STRINGS, measure_string_by_id_at};

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

impl Kind {
    /// The work variables this function leaves behind.
    pub fn regions(&self) -> [Region; 6] {
        [
            at(self.width_rva),
            at(self.x_rva),
            at(self.y_rva),
            at(self.left_rva),
            at(self.right_rva),
            at(MIDDLE_RVA),
        ]
    }

    pub fn state(&self) -> Vec<u8> {
        capture(&self.regions())
    }
}

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
pub const ALIGN_LEFT: u32 = 2;
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
        // Converted: `rc2k_core::buttons::draw`; the pieces blitted from the sprite table.
        use rc2k_core::buttons::{Borrowed, Error};
        let table = sprite_table();
        let borrowed = Borrowed {
            y: game_read(PLAIN.y_rva),
            x: game_read(PLAIN.x_rva),
        };
        let core = rc2k_core::buttons::Kind {
            slips: kind.slips,
            borrows: kind.borrows,
        };
        let (end, drawn) = rc2k_core::buttons::draw(
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
            rc2k_core::buttons::text_width(measured.width),
            dword(item + ITEM_X) as i32,
            dword(item + ITEM_Y) as i32,
            align,
            item,
            canvas,
        )
    }
}

// ---- self-tests ----

/// A menu item for the tests, as dwords, and the sprite numbers it points at.
pub struct TestItem {
    pub words: [u32; 0x5C / 4],
    _sprites: Box<[u32; 5]>,
}

impl TestItem {
    pub fn new() -> Self {
        let sprites = Box::new([0x40u32, 0x41, 0x42, 0x43, 0x44]);
        let mut words = [0xAAAA_AAAAu32; 0x5C / 4];
        words[(ITEM_X / 4) as usize] = 300;
        words[(ITEM_Y / 4) as usize] = 200;
        words[(ITEM_SPRITES / 4) as usize] = sprites.as_ptr() as u32;
        words[(ITEM_MEASURING_FONT / 4) as usize] = game_addr(rva(MEASURING_FONT)) as u32;
        words[(ITEM_STRING / 4) as usize] = TEST_STRINGS[0];
        Self {
            words,
            _sprites: sprites,
        }
    }

    pub fn set(&mut self, offset: u32, value: u32) {
        self.words[(offset / 4) as usize] = value;
    }

    pub fn addr(&self) -> u32 {
        self.words.as_ptr() as u32
    }
}

impl Default for TestItem {
    fn default() -> Self {
        Self::new()
    }
}

const TEST_WIDTHS: [u32; 9] = [1, 7, 31, 32, 33, 64, 65, 100, 200];

/// The self-test of one of the three button functions. Never a width of 0 or an alignment
/// outside 1 .. 3: those are fatal errors.
pub fn check_kind(t: &mut SelfTest, kind: &'static Kind, call_original: CallOriginal, rust: Draw) {
    // `draw_button`'s variables too: `draw_button_alt` reads two of them.
    let regions = [kind.regions(), PLAIN.regions()].concat();
    let saved = capture(&regions);
    let item = TestItem::new();
    for align in [ALIGN_CENTRE, ALIGN_LEFT, ALIGN_RIGHT] {
        for width in TEST_WIDTHS {
            let before = Regs {
                eax: width,
                ebx: 300,
                ecx: 200,
                edx: align,
                edi: item.addr(),
                ..Regs::SENTINEL
            };
            check_draw(
                t,
                &format!("align={align} width={width}"),
                call_original,
                before,
                &|| restore(&regions, &[0xA5; 48]),
                rust,
                &|| kind.state(),
            );
        }
    }
    restore(&regions, &saved);
}

/// The self-test of `draw_item_button`: both states of the highlight.
pub fn check_item_button(t: &mut SelfTest, call_original: CallOriginal, rust: Draw) {
    let regions = [&PLAIN.regions()[..], &[at(MENU_ITEM_RVA)]].concat();
    let saved = capture(&regions);
    let mut item = TestItem::new();
    item.set(ITEM_TYPE, 7);
    for highlighted in [7u32, 3] {
        for align in [ALIGN_CENTRE, ALIGN_LEFT, ALIGN_RIGHT] {
            for width in TEST_WIDTHS {
                item.set(ITEM_WIDTH, width);
                item.set(ITEM_ALIGN, align);
                let before = Regs {
                    edi: item.addr(),
                    ..Regs::SENTINEL
                };
                check_draw(
                    t,
                    &format!(
                        "highlighted={} align={align} width={width}",
                        highlighted == 7
                    ),
                    call_original,
                    before,
                    &|| {
                        restore(&regions[..6], &[0xAA; 24]);
                        unsafe { game_write(MENU_ITEM_RVA, highlighted) };
                    },
                    rust,
                    &|| PLAIN.state(),
                );
            }
        }
    }
    restore(&regions, &saved);
}

/// The self-test of one of the four text buttons: the game's strings, measured with its
/// measuring font. The caller has checked `measuring_unavailable`.
pub fn check_text_button(
    t: &mut SelfTest,
    kind: &'static Kind,
    call_original: CallOriginal,
    rust: Draw,
) {
    let font = (game_addr(rva(MEASURING_FONT)) as u32, 0x20);
    let regions = [&kind.regions()[..], &[font, (text() as u32, TEXT_LEN)]].concat();
    let saved = capture(&regions);
    let mut item = TestItem::new();
    for id in TEST_STRINGS {
        for measure_x in [0u32, 7] {
            item.set(ITEM_STRING, id);
            let before = Regs {
                ecx: measure_x,
                edi: item.addr(),
                ..Regs::SENTINEL
            };
            check_draw(
                t,
                &format!("string={id:#x} ecx={measure_x}"),
                call_original,
                before,
                &|| {
                    restore(&regions, &saved);
                    restore(&regions[..6], &[0xAA; 24]);
                },
                rust,
                &|| [kind.state(), capture(&regions[7..])].concat(),
            );
        }
    }
    restore(&regions, &saved);
}
