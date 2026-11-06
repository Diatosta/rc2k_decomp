//! `menu::rally_stages_loop` (0x472596): the loop of the screen that shows a rally's stages,
//! on the map (`ukmap.png`), before the service area in a championship.
//!
//! The owner: "the screen before the service area in championship that shows that rally's stages".
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own
//! (`SCREEN`): before the loop it clears the Escape flag `dword_755A3C`;
//! its background comes from `menu::load_background`, after which a set carry flag is fatal
//! error 0x80712345; leaving is asked about, and a "no" sets the screen up again.
//! Callers: the event loops `sub_4A0E60` and `sub_4AA4D7`.
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, FrameRoutine, List, Screen, Step};

// ---- game data ----

// ---- Rust version: `menu::screen_loop` with this ----

pub const SCREEN: Screen = Screen {
    menu: || crate::menu::owner::menus().tables.menus.rally_stages.id(),
    saved_item: || &crate::menu::owner::menus().screens.items.current[slot::RALLY_STAGES], // dword_747770
    lists: &[
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.scene_list,
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .rally_stages_scene
                    .id()
            },
            run: false,
        },
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.draw_list,
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .list_758d56
                    .id()
            },
            run: false,
        },
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.overlay_list,
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .rally_stages_overlay
                    .id()
            },
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: FrameRoutine::Menu,
    later_frame_callback: None,
    before: &[Step::Write(
        || &crate::menu::owner::menus().screens.draw_work.escaped,
        0,
    )],
    set_up_first: &[],
    prepare: &[],
    after_lists: &[],
    background: &[Step::Call(Callee::LoadBackground(|| {
        &crate::menu::owner::menus()
            .tables
            .texts
            .rally_stages_picture
    }))],
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
};
