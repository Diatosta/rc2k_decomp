//! `menu::leave_question_loop` (0x47FA94): the loop of the "leave?" question that
//! `menu::confirm_leaving` puts.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own
//! (`SCREEN`), as extracted from the disassembly: background `mainscr.png` through
//! `menu::load_background` (carry afterwards = fatal error 0x80712345); leaving is not asked
//! about; its own 3D list goes to `g_sceneList` before the first frame only.
//! A reading, not yet a visit: its one caller is `menu::confirm_leaving`, and it sets
//! `g_askingYesNo` to 1 while it is up, which is what makes `handle_yes_no` listen.
//! Callers: `menu::confirm_leaving`.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::rva;
use crate::menu::screen_loop::{List, Screen, Step};

// ---- Rust version: `menu::screen_loop` with this ----

pub const SCREEN: Screen = Screen {
    menu: rva(0x7544D4),
    saved_item: rva(0x7476D0),
    lists: &[
        List {
            pointer: rva(0x7554F4), // g_unreadList
            table: rva(0x758B72),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758C9E),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x7591DE),
            run: false,
        },
    ],
    first_frame_lists: Some(&[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x759002),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758C9E),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x7591DE),
            run: false,
        },
    ]),
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[],
    set_up_first: &[Step::Write(rva(0x7554CC), 0x1)],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::ArgAddr(rva(0x731BE4)),
        Step::CallOrFatal(rva(0x47EFFF), 0x80712345),
    ],
    after_first_frame: &[],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[Step::Write(rva(0x7554CC), 0x0)],
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
