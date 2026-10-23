//! `menu::overall_rally_results_loop` (0x466B21): the loop of the fourth results screen after a
//! stage, the rally's standings so far for all classes.
//!
//! Screen: the owner's screenshot of 2026-10-11, from a visit the Rust version ran (hook.log
//! has it right after the class standings): title "QUADRO DE LIDERANÇA GERAL", the same
//! layout as the class standings. Named `screen_466b21_loop` until then.
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
//! Callers: `sub_4A3BA2` and the code at 0x4A9572. Ungated thunk; shadow mode runs the original alone; each real call writes a
//! line to hook.log. Self-test: `screen_loop::check_screen`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::menu::class_stage_results_loop::LOAD_BACKGROUND;
use crate::menu::run_draw_list::DRAW_LIST_RVA;
use crate::menu::screen_loop::{Env, FRAME_CALLBACK, List, Screen, Step, screen_entry};

// ---- game data ----

const MENU_RVA: usize = rva(0x74FD20); // off_74FD20
const FIRST_ITEM_STRING_RVA: usize = MENU_RVA + 4 + 0x14;
const WHICH_TITLE_RVA: usize = rva(0x75AADC); // dword_75AADC
const BACKGROUND_RVA: usize = rva(0x732560); // "files\g\graphics\screens\results1.png"

// ---- Rust version: `menu::screen_loop` with this ----

unsafe fn set_first_item(g: &mut Regs, _: &mut dyn Env) {
    unsafe {
        g.eax = if game_read::<u32>(WHICH_TITLE_RVA) == 1 {
            0x138
        } else {
            0x139
        };
        g.edi = game_addr(MENU_RVA) as u32 + 4;
        game_write(FIRST_ITEM_STRING_RVA, g.eax);
    }
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
        Step::EaxAddr(BACKGROUND_RVA),
        Step::CallOrFatal(LOAD_BACKGROUND, 0x8071_2345),
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

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    screen_entry("menu::overall_rally_results_loop", r, &SCREEN)
}
