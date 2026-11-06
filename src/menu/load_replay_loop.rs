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
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, Entry, FrameRoutine, List, Screen, Step};

// ---- Rust version: `menu::screen_loop` with this ----

fn before(_: &Entry) -> Result<(), Refused> {
    crate::menu::owner::menus().screens.items.current[slot::LOAD_REPLAY]
        .set(crate::menu::owner::menus().screens.items.first[slot::LOAD_REPLAY].get());
    crate::menu::owner::menus()
        .screens
        .draw_work
        .escaped
        .set(0x0u32);
    Ok(())
}

/// What the loop returns in eax, written by hand from the end of the listing (after the
/// `popa`): -1 if the highlight is on item 0x16 or `dword_758806` is 1, else the replay's
/// number, `dword_755248` * 20 + the highlighted item - 1.
fn result() -> u32 {
    let item: u32 = crate::menu::owner::menus().screens.loop_flags.item.get();
    if item == 0x16
        || crate::menu::owner::menus()
            .tables
            .captions
            .screen_left
            .get()
            == 1
    {
        return 0xFFFF_FFFF;
    }
    crate::menu::owner::menus()
        .tables
        .pages
        .replay_page
        .get()
        .wrapping_mul(0x14)
        .wrapping_add(item)
        .wrapping_sub(1)
}

pub const SCREEN: Screen = Screen {
    menu: || crate::menu::owner::menus().tables.menus.load_replay.id(),
    saved_item: || &crate::menu::owner::menus().screens.items.current[slot::LOAD_REPLAY],
    lists: &[
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.unread_list, // g_unreadList
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .unread_758b72
                    .id()
            },
            run: false,
        },
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.draw_list, // g_drawList
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .draw_758bd6
                    .id()
            },
            run: false,
        },
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.setup_list, // dword_755500
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .load_replay_setup
                    .id()
            },
            run: true,
        },
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.overlay_list, // g_overlayList
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .overlay_759152
                    .id()
            },
            run: false,
        },
    ],
    first_frame_lists: Some(&[
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.scene_list, // g_sceneList
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .load_replay_scene
                    .id()
            },
            run: false,
        },
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.draw_list, // g_drawList
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .draw_758bd6
                    .id()
            },
            run: false,
        },
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.overlay_list, // g_overlayList
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .overlay_759152
                    .id()
            },
            run: false,
        },
    ]),
    frame_callback: FrameRoutine::Menu,
    later_frame_callback: None,
    before: &[Step::Do(before)],
    set_up_first: &[],
    prepare: &[],
    after_lists: &[],
    background: &[Step::Call(Callee::LoadBackground(|| {
        &crate::menu::owner::menus().tables.texts.load_replay_picture
    }))],
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
