//! `menu::arcade_review_options_loop` (0x488958): the menu after the results screens, by its
//! title "OPÇÕES DE REVISÃO" (a reading)
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`), as
//! extracted from the disassembly: background `roptions.png`; leaving is asked about; also when
//! `dword_755569` is 1; the steps that are more than a call or a store are the functions below,
//! generated from the listing. Named by its address until then. visit.
//! Callers: `sub_487E2C`.

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use crate::menu::screen_loop::{Callee, Carried, List, Screen, Step, carousel, prepare_scene};

// ---- Rust version: `menu::screen_loop` with this ----

unsafe fn set_up_first(c: &mut Carried) -> Result<(), Refused> {
    unsafe {
        game_write(rva(0x755569), 0x0u32);
        game_write(rva(0x623FB0), 0xFFFFFF64u32);
        game_write(rva(0x623FB4), 0x27u32);
        game_write(rva(0x624080), 0x140u32);
        prepare_scene(c);
        game_write(rva(0x7554EC), 0x1u32);
        game_write(rva(0x75AA74), 0xFu32);
        carousel(c)?;
        c.arg = game_read(rva(0x74779C));
        game_write(rva(0x755525), c.arg);
        c.arg = c.arg.wrapping_sub(1);
        game_write(rva(0x75AA7C), c.arg);
        game_write(rva(0x7554EC), 0x0u32);
    }
    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: rva(0x750EF8),
    saved_item: rva(0x7477C4),
    lists: &[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x758FA6),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758EC6),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x7592F6),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[
        Step::Write(rva(0x75555D), 0x0),
        Step::Write(rva(0x755980), 0x0),
    ],
    set_up_first: &[Step::Do(set_up_first)],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::ArgAddr(rva(0x732957)),
        Step::CallOrFatal(Callee::LoadBackground, 0x80712345),
    ],
    after_first_frame: &[],
    confirm: Some(&[]),
    ask_also_if: Some(rva(0x755569)),
    confirmed: &[],
    unasked_skips_save: false,
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
