//! `menu::unused_section_times_loop_2` (0x467392; the same but for its scene list:
//! `_1`, 0x466E50): the ten best times of a section (`screens::draw_section_best_times`),
//! background `results1.png`. Nothing in RAL.EXE reaches either; reached here from the debug
//! UI's "Screens" tab (`debug::screens`), which says the section (`EVENT_STATE.section`).
use crate::menu::owner::menus;
use crate::menu::screen_loop::{Callee, FrameRoutine, List, Screen, Step};

/// `unk_747798`'s slot, the item kept between visits.
const SLOT: usize = (0x747798 - 0x747660) / 4;

// ---- Rust version: `menu::screen_loop` with this ----

pub const SCREEN: Screen = Screen {
    menu: || menus().tables.menus.menu_74fea8.id(),
    saved_item: || &menus().screens.items.current[SLOT],
    lists: &[
        List {
            pointer: || &menus().screens.flow.scene_list, // g_sceneList
            table: || menus().tables.draw_lists.list_7590ee.id(),
            run: false,
        },
        List {
            pointer: || &menus().screens.flow.draw_list, // g_drawList
            table: || menus().tables.draw_lists.list_758e4e.id(),
            run: false,
        },
        List {
            pointer: || &menus().screens.flow.overlay_list, // g_overlayList
            table: || menus().tables.draw_lists.list_7592ae.id(),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: FrameRoutine::Menu,
    later_frame_callback: None,
    before: &[],
    set_up_first: &[],
    prepare: &[],
    after_lists: &[],
    background: &[Step::Call(Callee::LoadBackground(|| {
        &menus().tables.texts.results1_png_3
    }))],
    after_first_frame: &[],
    confirm: None,
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
