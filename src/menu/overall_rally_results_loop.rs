//! `menu::overall_rally_results_loop` (0x466B21): the loop of the fourth results screen after a
//! stage, the rally's standings so far for all classes.
//!
//! Named `screen_466b21_loop` until then.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own
//! (`SCREEN`): before the loop it sets the string of the menu's first item to 0x138 if
//! `dword_75AADC` is 1 and to 0x139 otherwise, as the stage results screen does;
//! its background comes from `menu::load_background`, after which a set carry flag is fatal
//! error 0x80712345; leaving is asked about, and a "no" sets the screen up again.
//! If `dword_75AB34` is 1 when the screen is left, nothing more is done: no question,
//! and the highlight is not saved.
//! Callers: `sub_4A3BA2` and the code at 0x4A9572.

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, Entry, FrameRoutine, List, Screen, Step};

// ---- game data ----

// ---- Rust version: `menu::screen_loop` with this ----

fn set_first_item(entry: &Entry) -> Result<(), Refused> {
    let menus = entry.menus;
    let title = if menus.screens.state.menus.which_title.get() == 1 {
        0x138
    } else {
        0x139
    };
    menus.tables.menus.overall_rally_results.items[0]
        .string
        .set(title);

    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: |m| m.tables.menus.overall_rally_results.id(),
    saved_item: |m| &m.screens.items.current[slot::OVERALL_RALLY_RESULTS], // dword_74778C
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |m| m.tables.draw_lists.overall_rally_results_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |m| m.tables.draw_lists.list_758e36.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |m| m.tables.draw_lists.overall_rally_results_overlay.id(),
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
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.overall_rally_results_picture
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
    leave_at_once: Some(|m| &m.screens.state.menus.standings_shown), // dword_75AB34
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
