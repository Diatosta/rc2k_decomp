//! `menu::arcade_results_loop` (0x4891EF): a results screen, by its title "RESULTADOS" (a
//! reading)
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `results1.png`; leaving is asked about; left
//! without the question, the highlight is not saved; its own 3D list goes to `g_sceneList`
//! before the first frame only.
//! Ran in the Rust version inside the arcade mode on 2026-10-12 (hook.log); the owner said
//! the arcade loops are named right. Named by its address until then.
//! Callers: `sub_487E2C`. Ungated thunk; shadow mode runs the original alone; each real call
//! writes a line to hook.log. Self-test: `screen_loop::check_screen`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::rva;
use crate::menu::screen_loop::{List, Screen, Step};

// ---- Rust version: `menu::screen_loop` with this ----

pub const SCREEN: Screen = Screen {
    menu: rva(0x74D184),
    saved_item: rva(0x74766C),
    lists: &[
        List {
            pointer: rva(0x7554F4), // g_unreadList
            table: rva(0x758B72),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758E76),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x7592BE),
            run: false,
        },
    ],
    first_frame_lists: Some(&[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x7590FA),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758E76),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x7592BE),
            run: false,
        },
    ]),
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[Step::Write(rva(0x755980), 0x0)],
    set_up_first: &[],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::ArgAddr(rva(0x732BFF)),
        Step::CallOrFatal(rva(0x47EFFF), 0x80712345),
    ],
    after_first_frame: &[],
    confirm: Some(&[]),
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: true,
    after: &[Step::Write(rva(0x7556A4), 0x6F)],
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
};
