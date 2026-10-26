//! `menu::rally_stages_loop` (0x472596): the loop of the screen that shows a rally's stages,
//! on the map (`ukmap.png`), before the service area in a championship.
//!
//! Screen: the owner's screenshot of 2026-10-10, from a visit the Rust version ran (hook.log
//! has "menu::screen_472596_loop: the Rust version runs", its name then, just before the
//! service area's loop): title "PORMENORES DA ETAPA", the rally's name, and a table "es /
//! localização / distância / tempo" of the rally's six stages and three service areas over
//! a map, the next stage in red. The owner: "the screen before the service area in
//! championship that shows that rally's stages".
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own
//! (`SCREEN`): before the loop it clears the Escape flag `dword_755A3C`;
//! its background comes from `menu::load_background`, after which a set carry flag is fatal
//! error 0x80712345; leaving is asked about, and a "no" sets the screen up again.
//! Callers: the event loops `sub_4A0E60` and `sub_4AA4D7`. Ungated thunk; shadow mode runs the original alone; each real call writes a
//! line to hook.log. Self-test: `screen_loop::check_screen`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, rva};
use crate::menu::class_stage_results_loop::LOAD_BACKGROUND;
use crate::menu::run_draw_list::DRAW_LIST_RVA;
use crate::menu::screen_loop::{FRAME_CALLBACK, List, Screen, Step, screen_entry};

// ---- game data ----

const MENU_RVA: usize = rva(0x74DB04); // off_74DB04
const BACKGROUND_RVA: usize = rva(0x73245F); // "files\g\graphics\screens\ukmap.png"

// ---- Rust version: `menu::screen_loop` with this ----

pub const SCREEN: Screen = Screen {
    menu: MENU_RVA,
    saved_item: rva(0x747770), // dword_747770
    lists: &[
        List {
            pointer: rva(0x755504),
            table: rva(0x75905E),
            run: false,
        },
        List {
            pointer: DRAW_LIST_RVA,
            table: rva(0x758D56),
            run: false,
        },
        List {
            pointer: rva(0x7554FC),
            table: rva(0x759236),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: FRAME_CALLBACK,
    later_frame_callback: None,
    before: &[Step::Write(rva(0x755A3C), 0)],
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
    leave_at_once: None,
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
    also_reads: &[],
    also_writes: &[],
};

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    screen_entry("menu::rally_stages_loop", r, &SCREEN)
}
