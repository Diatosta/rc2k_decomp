//! `screens::draw_savegame_names` (0x46D45A): the names of the saved games on the page
//! shown, in two columns of five.
//!
//! In:  nothing
//! Out: every register preserved; carry of the last `draw_text`
//!
//! The ten names of 0x20 bytes from `byte_526BB4` + page (`dword_75523C`) times 0x140,
//! at x = 0xCF for the first five and 0x21C for the others, y = 0xB5, 0xD2, 0xEF, 0x10C,
//! 0x129, with the font at 0x51E8C4. In two draw lists (the load and save screens). The
//! twin of `screens::draw_setup_slot_names`.
//! Refuses a page past the tenth (the list has 100 names).
//! Self-test: the ten pages of the owner's own list, nothing drawn.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::name_lists::{SAVEGAME_NAMES_RVA, SAVEGAME_PAGE_RVA};
use crate::screens::harness::{Left, check_draw, dispatch_draw, drawing_unavailable};
use crate::screens::{font_at, font_is_loaded};
use crate::service::harness::{at, capture, restore};
use crate::text::draw_text::{Canvas, draw_text_at};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::draw_savegame_names",
    rva(0x46D45A),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const FONT: usize = 0x51E8C4;
const COLUMNS: [i32; 2] = [0xCF, 0x21C];
const ROWS: [i32; 5] = [0xB5, 0xD2, 0xEF, 0x10C, 0x129];
const NAME_LEN: usize = 0x20;
const PAGES: u32 = 10;

// ---- Rust version ----

pub unsafe fn draw_savegame_names(canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    let page: u32 = unsafe { game_read(SAVEGAME_PAGE_RVA) };
    if page >= PAGES {
        return Err(BadInput("a page past the saved games' names").into());
    }
    let mut name = game_addr(SAVEGAME_NAMES_RVA) + page as usize * 10 * NAME_LEN;
    let mut carry = false;
    for x in COLUMNS {
        for y in ROWS {
            carry =
                unsafe { draw_text_at(font_at(FONT), name as *const u8, x, y, canvas) }?.too_wide;
            name += NAME_LEN;
        }
    }
    Ok(carry)
}

// ---- hook ----

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    Ok(Left::carry(unsafe { draw_savegame_names(canvas) }?))
}

fn state() -> Vec<u8> {
    Vec::new()
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_draw(&HOOK, r, call_original, &rust, &state);
}

// ---- self-test ----

/// True if every name ends within its 0x20 bytes.
fn names_end() -> bool {
    (0..PAGES as usize * 10).all(|n| {
        let name = game_addr(SAVEGAME_NAMES_RVA) + n * NAME_LEN;
        (0..NAME_LEN).any(|i| unsafe { ((name + i) as *const u8).read() } == 0)
    })
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = drawing_unavailable() {
        return t.skip(why);
    }
    if !font_is_loaded(FONT) || !names_end() {
        return t.skip("the font is not loaded or a name has no end");
    }
    let regions = [at(SAVEGAME_PAGE_RVA)];
    let saved = capture(&regions);
    for page in 0..PAGES {
        check_draw(
            &mut t,
            &format!("page={page}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe { game_write(SAVEGAME_PAGE_RVA, page) },
            &rust,
            &state,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
