//! `screens::draw_menu_item_text` (0x471E99): draws a menu item's text.
//!
//! In:  eax = string id, ebx = font, ecx = x, edx = y, ebp = 1 for spaced-out letters
//! Out: every register preserved
//! Normally one `draw_string_by_id`. With ebp = 1 the string is built in the line buffer
//! (`g_lineBuffer`) and drawn one character at a time, 18 pixels apart, from x - 26, each
//! through a one-character text at `byte_A8BC54`; the font is kept in `dword_7559EC`.
//! One caller, `draw_menu_items`.
//! Manual patch with a private copy (0x57 bytes, four calls relocated): a conditional jump
//! sits in the first bytes.
//! Refuses, in the spaced-out way, an empty string (the original draws on from the bytes
//! after its NUL) and one of 0x100 bytes or more (it would run into the one-character text).

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_ptr, game_write, hook_asm, rva};
use crate::screens::harness::{Left, check_draw, dispatch_draw, drawing_unavailable};
use crate::screens::{LINE_BUFFER_RVA, font_at, font_is_loaded, set_line};
use crate::service::harness::{Region, at, capture, restore};
use crate::strings::string_table::{blob, is_table_id};
use crate::strings::text_from_ids::{TEXT_LEN, text};
use crate::text::draw_string_by_id::{draw_string_by_id_at, string_text};
use crate::text::draw_text::{Canvas, draw_text_at};
use crate::text::font::Font;
use crate::text::measure_string_by_id::TEST_STRINGS;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_menu_item_text",
    rva(0x471E99),
    thunk,
    &ORIGINAL,
)
.private_copy(0x57)
.relocate_calls(&[0x12, 0x17, 0x3F, 0x50])
.manual_patch()
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const SPACED_FONT_RVA: usize = rva(0x7559EC); // dword_7559EC
const ONE_CHARACTER_RVA: usize = rva(0xA8BC54); // byte_A8BC54
const LINE_LEN: usize = 0x100; // from g_lineBuffer to byte_A8BC54

/// What the function may write, the shared text buffer included.
pub fn item_text_regions() -> [Region; 4] {
    [
        at(SPACED_FONT_RVA),
        (game_addr(LINE_BUFFER_RVA) as u32, LINE_LEN),
        at(ONE_CHARACTER_RVA),
        (text() as u32, TEXT_LEN),
    ]
}

// ---- Rust version ----

pub unsafe fn draw_menu_item_text(
    id: u32,
    font: *mut Font,
    x: i32,
    y: i32,
    spaced: bool,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    unsafe {
        if !spaced {
            draw_string_by_id_at(id, font, x, y, canvas)?;
            return Ok(());
        }
        let line = string_text(id)?;
        if line.len() == 1 {
            return Err(BadInput("an empty string, spaced out").into());
        }
        if line.len() > LINE_LEN {
            return Err(BadInput("a string longer than the line buffer").into());
        }
        game_write(SPACED_FONT_RVA, font as u32);
        set_line(&line);
        let one: *mut u8 = game_ptr(ONE_CHARACTER_RVA);
        let mut x = x.wrapping_sub(0x1A);
        for &character in &line[..line.len() - 1] {
            game_write(ONE_CHARACTER_RVA, character as u32);
            draw_text_at(font, one, x, y, canvas)?;
            x = x.wrapping_add(0x12);
        }
    }
    Ok(())
}

// ---- hook ----

fn rust(r: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe {
        draw_menu_item_text(
            r.eax,
            r.ebx as *mut Font,
            r.ecx as i32,
            r.edx as i32,
            r.ebp == 1,
            canvas,
        )
    }?;
    Ok(Left::nothing())
}

fn state() -> Vec<u8> {
    capture(&item_text_regions())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw(&HOOK, r, call_original, &rust, &state);
}

// ---- self-test: the game's strings and one of its fonts ----

pub const TEST_FONT: usize = 0x51E584;

/// Why a test that draws the game's strings cannot run, or None if it can.
pub fn text_unavailable() -> Option<&'static str> {
    drawing_unavailable().or_else(|| {
        (blob().is_null()
            || !font_is_loaded(TEST_FONT)
            || !TEST_STRINGS.iter().all(|&id| is_table_id(id))
            || TEST_STRINGS
                .iter()
                .any(|&id| unsafe { string_text(id) }.is_ok_and(|s| s.len() < 2)))
        .then_some("the strings or the font are not loaded")
    })
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = text_unavailable() {
        return t.skip(why);
    }
    let regions = item_text_regions();
    let saved = capture(&regions);
    for id in TEST_STRINGS {
        for spaced in [0u32, 1, 2] {
            let before = Regs {
                eax: id,
                ebx: font_at(TEST_FONT) as u32,
                ecx: 320,
                edx: 140,
                ebp: spaced,
                ..Regs::SENTINEL
            };
            check_draw(
                &mut t,
                &format!("string={id:#x} ebp={spaced}"),
                call_original,
                before,
                &|| restore(&regions, &saved),
                &rust,
                &state,
            );
        }
    }
    restore(&regions, &saved);
    t.finish()
}
