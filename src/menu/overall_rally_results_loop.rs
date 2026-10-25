//! `menu::overall_rally_results_loop` (0x466B21): the loop of the fourth results screen after a
//! stage, the rally's standings so far for all classes.
//!
//! Named `screen_466b21_loop` until then.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own
//! (`SCREEN`): before the loop it sets the string of the menu's first item to 0x138 if
//! `dword_75AADC` is 1 and to 0x139 otherwise, as the stage results screen does;
//! its background comes from `menu::load_background`, after which a set carry flag is fatal
//! error 0x80712345; leaving is asked about, and a "no" sets the screen up again.
//! If `dword_75AB34` is 1 when the screen is left, nothing more is done: no question,
//! and the highlight is not saved.
//! Callers: `sub_4A3BA2` and the code at 0x4A9572.

use crate::errors::Refused;
use crate::framework::{game_write, rva};
use crate::menu::run_draw_list::DRAW_LIST_RVA;
use crate::menu::screen_loop::{Callee, Carried, FRAME_CALLBACK, List, Screen, Step};
use crate::menu::state_store::MENU_STATE;

// ---- game data ----

const MENU_RVA: usize = rva(0x74FD20); // off_74FD20
const FIRST_ITEM_STRING_RVA: usize = MENU_RVA + 4 + 0x14;
const BACKGROUND_RVA: usize = rva(0x732560); // "files\g\graphics\screens\results1.png"

// ---- Rust version: `menu::screen_loop` with this ----

unsafe fn set_first_item(c: &mut Carried) -> Result<(), Refused> {
    unsafe {
        c.arg = if MENU_STATE.menus.which_title.get() == 1 {
            0x138
        } else {
            0x139
        };
        game_write(FIRST_ITEM_STRING_RVA, c.arg);
    }
    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: MENU_RVA,
    saved_item: rva(0x74778C), // dword_74778C
    lists: &[
        List {
            pointer: rva(0x755504),
            table: rva(0x7590DE),
            run: false,
        },
        List {
            pointer: DRAW_LIST_RVA,
            table: rva(0x758E36),
            run: false,
        },
        List {
            pointer: rva(0x7554FC),
            table: rva(0x7592A2),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: FRAME_CALLBACK,
    later_frame_callback: None,
    before: &[Step::Do(set_first_item)],
    set_up_first: &[],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::ArgAddr(BACKGROUND_RVA),
        Step::CallOrFatal(Callee::LoadBackground, 0x8071_2345),
    ],
    after_first_frame: &[],
    confirm: Some(&[]),
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[],
    escape_leaves: false,
    each_frame: &[],
    short_action_check: false,
    leave_at_once: Some(rva(0x75AB34)), // dword_75AB34
    next_frame_repeats_first: false,
    action_saved_item: None,
    action_menu: None,
    no_quit_check: false,
    before_asking: &[],
    ask_unless: None,
    leaving: &[],
    leave_saved_item: None,
    keys_menu: None,
    after_restored: &[],
    result: None,
    late_highlight: false,
    before_first_frame: &[],
    before_frame: &[],
    after_blink: &[],
    restart_action_check: false,
    always_asks: false,
};
