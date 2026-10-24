//! `menu::load_replay_loop` (0x47F48B): the replay list "CARREGAR REPETIÇÃO"; returns the
//! replay chosen.
//!
//! In:  nothing
//! Out: eax = the result; every other register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `loadsv.png`; leaving is not asked about; its
//! own 3D list goes to `g_sceneList` before the first frame only; the Escape flag is looked at
//! at the top of every frame; it returns a value in eax; the steps that are more than a call or
//! a store are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "CARREGAR REPETIÇÃO", not yet a
//! visit.
//! Callers: `sub_46F786`.

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use crate::menu::screen_loop::{Callee, Carried, List, Screen, Step};

// ---- Rust version: `menu::screen_loop` with this ----

unsafe fn before(c: &mut Carried) -> Result<(), Refused> {
    unsafe {
        c.measure_x = game_read(rva(0x747824));
        game_write(rva(0x747688), c.measure_x);
        game_write(rva(0x755A3C), 0x0u32);
    }
    Ok(())
}

/// What the loop returns in eax, written by hand from the end of the listing (after the
/// `popa`): -1 if the highlight is on item 0x16 or `dword_758806` is 1, else the replay's
/// number, `dword_755248` * 20 + the highlighted item - 1.
unsafe fn result() -> u32 {
    unsafe {
        let item: u32 = game_read(rva(0x755525));
        if item == 0x16 || game_read::<u32>(rva(0x758806)) == 1 {
            return 0xFFFF_FFFF;
        }
        game_read::<u32>(rva(0x755248))
            .wrapping_mul(0x14)
            .wrapping_add(item)
            .wrapping_sub(1)
    }
}

pub const SCREEN: Screen = Screen {
    menu: rva(0x753CE4),
    saved_item: rva(0x747688),
    lists: &[
        List {
            pointer: rva(0x7554F4), // g_unreadList
            table: rva(0x758B72),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758BD6),
            run: false,
        },
        List {
            pointer: rva(0x755500), // dword_755500
            table: rva(0x759462),
            run: true,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x759152),
            run: false,
        },
    ],
    first_frame_lists: Some(&[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x758F2A),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758BD6),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x759152),
            run: false,
        },
    ]),
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[Step::Do(before)],
    set_up_first: &[],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::ArgAddr(rva(0x731EF5)),
        Step::CallOrFatal(Callee::LoadBackground, 0x80712345),
    ],
    after_first_frame: &[],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[],
    escape_leaves: true,
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
    result: Some(result),
    late_highlight: false,
    before_first_frame: &[],
    before_frame: &[],
    after_blink: &[],
    restart_action_check: false,
    always_asks: false,
};
