//! `menu::championship_standings_loop` (0x4670DF): the loop of the championship table, the
//! points of every driver in each of the six rallies and in total.
//!
//! Screen: the owner's screenshots of 2026-10-11, from a visit the Rust version ran (hook.log):
//! title "CAMPEONATO GERAL", columns under the six rallies' logos and "total", rows 81 to
//! 90, buttons "81-90" and "continuar"; reached from the menu that follows the results
//! screens. In those screenshots the 3D objects of that menu (green arrows, a floppy disk)
//! are drawn over the table. That was a fault of the Rust version, not of the game: the
//! owner's screenshot of the same screen with the original running (shadow on, 2026-10-11)
//! has no such objects. This loop puts its own empty 3D list in `g_sceneList` only before
//! the first frame after set-up (and in the unread `dword_7554F4` at the other two places);
//! the first Rust version left that out, so the list of the menu before stayed. Fixed with
//! `Screen::first_frame_lists`; the owner's screenshot of the fixed build (Rust version
//! running, 2026-10-11) shows the table without them. Named `screen_4670df_loop` until then.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own
//! (`SCREEN`): nothing happens before the loop (its set-up starts at the function's second byte);
//! its background comes from `menu::load_background`, after which a set carry flag is fatal
//! error 0x80712345; leaving is asked about, and a "no" sets the screen up again.
//! If `dword_75AB34` is 1 when the screen is left, nothing more is done: no question,
//! and the highlight is not saved.
//! One caller, `sub_4A5A47`. Manual patch with a private copy (0x2B3 bytes; nine calls and
//! the `jb` out to the fatal-error stub relocated), because the set-up label is inside the
//! bytes a patch overwrites. Ungated thunk; shadow mode runs the original alone; each real
//! call writes a line to hook.log. Self-test: `screen_loop::check_screen`.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::class_stage_results_loop::LOAD_BACKGROUND;
use crate::menu::run_draw_list::DRAW_LIST_RVA;
use crate::menu::screen_loop::{
    FRAME_CALLBACK, List, Original, Screen, Step, check_screen, dispatch_screen, screen_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::championship_standings_loop",
    rva(0x4670DF),
    thunk,
    &ORIGINAL,
)
.private_copy(0x2B3)
.relocate_calls(&[
    0x8D, 0x92, 0x9E, 0xA3, 0xF2, 0xF7, 0x13F, 0x19D, 0x23F, 0x26D,
])
.manual_patch()
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

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
    also_reads: &[],
    also_writes: &[],
};

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_screen(&HOOK, r, call_original, &SCREEN);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    screen_entry("menu::championship_standings_loop", r, &SCREEN)
}

// ---- self-test ----

const THE_ORIGINAL: Original = Original {
    function: rva(0x4670DF),
    len: 0x2B3,
    entry_bytes: &[0x60, 0xC7, 0x05, 0x06, 0x88, 0x75, 0x00, 0x00],
    calls: &[0x8D, 0x9E, 0xA3, 0xF2, 0xF7, 0x13F, 0x19D, 0x23F, 0x26D],
};

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Err(why) = check_screen(&mut t, &SCREEN, &THE_ORIGINAL) {
        return t.skip(why);
    }
    t.finish()
}
