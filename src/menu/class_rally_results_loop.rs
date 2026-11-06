//! `menu::class_rally_results_loop` (0x466858): the loop of the third results screen after a
//! stage, the rally's standings so far for the player's class.
//!
//! Named `screen_466858_loop`, then `rally_results_loop`.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`):
//! before the loop it sets the string of the menu's first item to 0x138 if `dword_75AADC` is 1 and
//! to 0x139 otherwise, as the stage results screen does; its background comes from
//! `menu::load_background`, after which a set carry flag is fatal error 0x80712345; leaving is
//! asked about, and a "no" sets the screen up again. One caller, `sub_4A3317`.

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, Entry, FrameRoutine, List, Screen, Step};

// ---- game data ----

// ---- Rust version: `menu::screen_loop` with this ----

fn set_first_item(_: &Entry) -> Result<(), Refused> {
    let title = if crate::menu::owner::menus()
        .screens
        .state
        .menus
        .which_title
        .get()
        == 1
    {
        0x138
    } else {
        0x139
    };
    crate::menu::owner::menus()
        .tables
        .menus
        .class_rally_results
        .items[0]
        .string
        .set(title);

    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: || {
        crate::menu::owner::menus()
            .tables
            .menus
            .class_rally_results
            .id()
    },
    saved_item: || &crate::menu::owner::menus().screens.items.current[slot::CLASS_RALLY_RESULTS], // dword_747788
    lists: &[
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.scene_list,
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .class_rally_results_scene
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
                    .list_758e2a
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
                    .class_rally_results_overlay
                    .id()
            },
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: FrameRoutine::Menu,
    later_frame_callback: None,
    before: &[Step::Do(set_first_item)],
    set_up_first: &[],
    prepare: &[],
    after_lists: &[],
    background: &[Step::Call(Callee::LoadBackground(|| {
        &crate::menu::owner::menus()
            .tables
            .texts
            .class_rally_results_picture
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
