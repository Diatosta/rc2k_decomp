//! `menu::championship_standings_loop` (0x4670DF): the loop of the championship table, the
//! points of every driver in each of the six rallies and in total.
//!
//! In those screenshots the 3D objects of that menu (green arrows, a floppy disk) are drawn over
//! the table. This loop puts its own empty 3D list in `g_sceneList` only before the first frame
//! after set-up (and in the unread `dword_7554F4` at the other two places); the first Rust version
//! left that out, so the list of the menu before stayed. Fixed with `Screen::first_frame_lists`;
//! the owner's screenshot of the fixed build (Rust version running, 2026-10-11) shows the table
//! without them. Named `screen_4670df_loop` until then.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`):
//! nothing happens before the loop (its set-up starts at the function's second byte); its
//! background comes from `menu::load_background`, after which a set carry flag is fatal error
//! 0x80712345; leaving is asked about, and a "no" sets the screen up again. If `dword_75AB34` is 1
//! when the screen is left, nothing more is done: no question, and the highlight is not saved. One
//! caller, `sub_4A5A47`.

use crate::framework::rva;
use crate::menu::run_draw_list::DRAW_LIST_RVA;
use crate::menu::screen_loop::{Callee, FRAME_CALLBACK, List, Screen, Step};

// ---- game data ----

const MENU_RVA: usize = rva(0x74FFCC); // off_74FFCC
const BACKGROUND_RVA: usize = rva(0x7328E7); // "files\g\graphics\screens\champoc.png"

// ---- Rust version: `menu::screen_loop` with this ----

pub const SCREEN: Screen = Screen {
    menu: MENU_RVA,
    saved_item: rva(0x7477BC), // dword_7477BC
    lists: &[
        List {
            pointer: rva(0x7554F4), // dword_7554F4
            table: rva(0x758B72),
            run: false,
        },
        List {
            pointer: DRAW_LIST_RVA,
            table: rva(0x758E5E),
            run: false,
        },
        List {
            pointer: rva(0x7554FC),
            table: rva(0x7592B6),
            run: false,
        },
    ],
    first_frame_lists: Some(&[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x7590F6),   // unk_7590F6: empty
            run: false,
        },
        List {
            pointer: DRAW_LIST_RVA,
            table: rva(0x758E5E),
            run: false,
        },
        List {
            pointer: rva(0x7554FC),
            table: rva(0x7592B6),
            run: false,
        },
    ]),
    frame_callback: FRAME_CALLBACK,
    later_frame_callback: None,
    before: &[],
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
