//! `menu::overall_stage_results_loop` (0x46658F): the loop of the second results screen after a
//! stage, the stage's times for all classes.
//!
//! Named `screen_46658f_loop` until then.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`):
//! before the loop it sets the string of the menu's first item to 0x138 if `dword_75AADC` is 1 and
//! to 0x139 otherwise, as the stage results screen does; its background comes from
//! `menu::load_background`, after which a set carry flag is fatal error 0x80712345; leaving is
//! asked about, and a "no" sets the screen up again. One caller, `sub_4A1FE5`.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::menus_store::MENUS;
use crate::menu::screen_items_store::{SCREEN_ITEMS, slot};
use crate::menu::screen_loop::{Callee, Carried, FRAME_CALLBACK, List, Screen, Step};
use crate::menu::state_store::MENU_STATE;

// ---- game data ----

// ---- Rust version: `menu::screen_loop` with this ----

fn set_first_item(c: &mut Carried) -> Result<(), Refused> {
    c.arg = if MENU_STATE.menus.which_title.get() == 1 {
        0x138
    } else {
        0x139
    };
    MENUS.overall_stage_results.items[0].string.set(c.arg);

    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: || MENUS.overall_stage_results.addr(),
    saved_item: || &SCREEN_ITEMS.current[slot::OVERALL_STAGE_RESULTS], // dword_747784
    lists: &[
        List {
            pointer: || &SCREEN_FLOW.scene_list,
            table: || DRAW_LISTS.overall_stage_results_scene.addr(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.draw_list,
            table: || DRAW_LISTS.list_758e1e.addr(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.overlay_list,
            table: || DRAW_LISTS.overall_stage_results_overlay.addr(),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: FRAME_CALLBACK,
    later_frame_callback: None,
    before: &[Step::Do(set_first_item)],
    set_up_first: &[],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::ArgAddr(|| {
            crate::menu::texts_store::MENU_TEXTS
                .overall_stage_results_picture
                .addr()
        }),
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
