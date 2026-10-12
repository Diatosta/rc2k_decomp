//! `menu::draw_menu` (0x4746B7): draws a menu, its fixed labels and then its items.
//!
//! In:  edi = the menu; ebx, ecx as they come (`draw_menu_items` uses them)
//! Out: every register preserved
//! `pusha`, `screens::draw_menu_labels`, `screens::draw_menu_items`, `popa`. Called by
//! `menu::draw_frame` and `menu::frame_callback`, and by the screens with a frame callback
//! of their own.
//! Manual patch with a private copy (0x0D bytes, both calls relocated): the first call
//! starts at the second byte, inside what the patch overwrites.
//! Refuses what the two functions refuse.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_ptr, game_write, hook_asm, rva};
use crate::screens::draw_menu_item_text::{TEST_FONT, text_unavailable};
use crate::screens::draw_menu_items::draw_menu_items;
use crate::screens::draw_menu_labels::draw_menu_labels;
use crate::screens::font_at;
use crate::screens::harness::{Left, check_draw, dispatch_draw_rewound, drawing_unavailable};
use crate::screens::{draw_menu_items, draw_menu_labels};
use crate::service::harness::{at, capture, restore};
use crate::service::state::MENU_ITEM_RVA;
use crate::text::blend_font::blend_regions;
use crate::text::draw_text::Canvas;
use crate::text::font::Font;
use crate::text::measure_string_by_id::{MEASURING_FONT, TEST_STRINGS, measuring_unavailable};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("menu::draw_menu", rva(0x4746B7), thunk, &ORIGINAL)
    .private_copy(0x0D)
    .relocate_calls(&[0x01, 0x06])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// `ebx` and `ecx` are those registers at entry.
pub unsafe fn draw_menu(
    menu: u32,
    ebx: u32,
    ecx: u32,
    canvas: &mut dyn Canvas,
) -> Result<(), Refused> {
    unsafe {
        draw_menu_labels(menu, canvas)?;
        draw_menu_items(menu, ebx, ecx, canvas)
    }
}

// ---- hook ----

fn rust(r: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_menu(r.edi, r.ebx, r.ecx, canvas) }?;
    Ok(Left::nothing())
}

/// What the two functions leave behind.
fn state() -> Vec<u8> {
    [draw_menu_labels::state(), draw_menu_items::state()].concat()
}

extern "C" fn dispatch(r: &mut Regs) {
    // As for `draw_menu_items`: the blended font is put back before the original runs.
    dispatch_draw_rewound(&HOOK, r, call_original, &rust, &state, &blend_regions());
}

// ---- self-test: a menu of its own with two labels and three plain items, each of them
// highlighted in turn, with the game's strings and fonts ----

const END: u32 = u32::MAX;
const ITEM_DWORDS: usize = 0x5C / 4;
/// An item's +0x1C: nothing goes behind its text.
const NOTHING: u32 = 0xFFFF_FFA9;

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = drawing_unavailable()
        .or_else(text_unavailable)
        .or_else(measuring_unavailable)
    {
        return t.skip(why);
    }
    let font = font_at(TEST_FONT);
    if unsafe { font.read_unaligned() }.palette == 0 {
        return t.skip("the test font has no palette to blend");
    }
    let measuring: *mut Font = game_ptr(rva(MEASURING_FONT));
    let all = [
        &draw_menu_items::regions()[..],
        &[at(MENU_ITEM_RVA), (measuring as u32, 0x20)],
    ]
    .concat();
    let saved = (capture(&all), draw_menu_labels::state());
    let put_back = || {
        restore(&all, &saved.0);
        unsafe {
            std::ptr::copy_nonoverlapping(
                saved.1.as_ptr(),
                crate::strings::text_from_ids::text(),
                saved.1.len(),
            )
        };
    };

    for label_count in [1usize, 2] {
        let mut labels = Vec::new();
        for (n, id) in TEST_STRINGS.into_iter().take(label_count).enumerate() {
            labels.extend([90 + 200 * n as u32, 30 + 20 * n as u32, font as u32, id]);
        }
        labels.push(END);

        let mut menu = vec![0xAAAA_AAAAu32; 1 + 3 * ITEM_DWORDS + 1];
        menu[0] = labels.as_ptr() as u32;
        for n in 0..3 {
            let item = &mut menu[1 + n * ITEM_DWORDS..][..ITEM_DWORDS];
            let n = n as u32;
            item[0] = 300 + 20 * n; // x
            item[1] = 100 + 50 * n; // y
            item[2] = 120 + 20 * n; // the text's x
            item[3] = 104 + 50 * n; // the text's y
            item[4] = n + 1; // type
            item[5] = TEST_STRINGS[n as usize % 3];
            item[6] = font as u32;
            item[7] = NOTHING;
            item[8] = 0;
            item[0x4C / 4] = measuring as u32;
            item[0x50 / 4] = font as u32;
            item[0x58 / 4] = 0; // not spaced out
        }
        *menu.last_mut().unwrap() = END;

        for highlighted in 0..=3u32 {
            let before = Regs {
                ebx: 0x1234,
                ecx: 7,
                edi: menu.as_ptr() as u32,
                ..Regs::SENTINEL
            };
            check_draw(
                &mut t,
                &format!("{label_count} labels, highlighted item {highlighted}"),
                call_original,
                before,
                &|| {
                    put_back();
                    unsafe { game_write(MENU_ITEM_RVA, highlighted) };
                },
                &rust,
                &state,
            );
        }
    }
    put_back();
    t.finish()
}
