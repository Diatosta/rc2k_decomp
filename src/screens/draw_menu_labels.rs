//! `screens::draw_menu_labels` (0x471EF0): draws the fixed labels of a menu.
//!
//! In:  edi = the menu
//! Out: every register preserved; carry clear
//! The menu's first dword points at a list of labels, 0x10 bytes each: x, y, font, string id.
//! The list ends with -1, but the first label is drawn without being checked for it. Each is
//! drawn with `draw_string_by_id`.
//! One caller, the menu's drawing step `sub_4746B7`.
//! Refuses a menu without a label list, and a list without an end in its first 256 labels.
//! Manual patch with a private copy (0x1D bytes, one call relocated): the loop comes back to
//! the fourth byte, inside what the patch overwrites. The first build hooked it without one
//! and the original's second pass through the loop ran into the patch (2026-10-10).

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::harness::{Left, check_draw, dispatch_draw, draw_entry, drawing_unavailable};
use crate::screens::{font_at, font_is_loaded};
use crate::strings::string_table::{blob, is_table_id};
use crate::strings::text_from_ids::{TEXT_LEN, text};
use crate::text::draw_text::Canvas;
use std::slice::from_raw_parts;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("screens::draw_menu_labels", rva(0x471EF0), thunk, &ORIGINAL)
        .private_copy(0x1D)
        .relocate_calls(&[0x0E])
        .manual_patch()
        .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LABEL_LEN: u32 = 0x10;
const END: u32 = u32::MAX;
const LABEL_LIMIT: u32 = 256;

unsafe fn dword(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

// ---- Rust version ----

pub unsafe fn draw_menu_labels(menu: u32, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    unsafe {
        let labels = dword(menu);
        if labels == 0 {
            return Err(BadInput("a menu without a label list").into());
        }
        if !(1..=LABEL_LIMIT).any(|n| dword(labels + n * LABEL_LEN) == END) {
            return Err(BadInput("a label list without an end").into());
        }
        // Converted: `rc2k_core::screens::menus::labels` (the list read here: the first label
        // drawn without being checked for the end, as the original).
        use crate::screens::game_screen::{FontAt, MenuScreen};
        let mut list = Vec::new();
        let mut label = labels;
        loop {
            list.push(rc2k_core::screens::menus::Label {
                x: dword(label) as i32,
                y: dword(label + 4) as i32,
                font: FontAt(dword(label + 8)),
                id: dword(label + 0xC),
            });
            label += LABEL_LEN;
            if dword(label) == END {
                break;
            }
        }
        rc2k_core::screens::menus::labels(&mut MenuScreen::new(canvas), &list)
    }
}

// ---- hook ----

pub fn state() -> Vec<u8> {
    unsafe { from_raw_parts(text(), TEXT_LEN) }.to_vec()
}

fn rust(r: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_menu_labels(r.edi, canvas) }?;
    Ok(Left::carry(false))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw(&HOOK, r, call_original, &rust, &state);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("screens::draw_menu_labels", r, &rust)
}

// ---- self-test: label lists of its own, with the game's font and strings ----

const TEST_FONT: usize = 0x51E584;
const TEST_STRINGS: [u32; 3] = [0x4E5, 0x4E6, 0x4E7];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = drawing_unavailable() {
        return t.skip(why);
    }
    if blob().is_null()
        || !font_is_loaded(TEST_FONT)
        || !TEST_STRINGS.iter().all(|&id| is_table_id(id))
    {
        return t.skip("the strings or the font are not loaded");
    }
    let saved = state();
    let font = font_at(TEST_FONT) as u32;
    for count in [1usize, 3] {
        let mut labels = Vec::new();
        for (n, id) in TEST_STRINGS.into_iter().take(count).enumerate() {
            labels.extend([100 + 150 * n as u32, 40 + 30 * n as u32, font, id]);
        }
        labels.push(END);
        let menu = [labels.as_ptr() as u32, 0xAAAA_AAAA];
        let before = Regs {
            edi: menu.as_ptr() as u32,
            ..Regs::SENTINEL
        };
        check_draw(
            &mut t,
            &format!("{count} labels"),
            call_original,
            before,
            &|| unsafe { std::ptr::copy_nonoverlapping(saved.as_ptr(), text(), TEXT_LEN) },
            &rust,
            &state,
        );
    }
    unsafe { std::ptr::copy_nonoverlapping(saved.as_ptr(), text(), TEXT_LEN) };
    t.finish()
}
