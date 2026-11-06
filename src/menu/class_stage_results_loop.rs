//! `menu::class_stage_results_loop` (0x4662C6): the loop of the first results screen after a
//! stage, the stage's times for the player's class.
//!
//! Named `screen_4662c6_loop`, then `stage_results_loop`.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`):
//! before the loop it sets the string of the menu's first item, the button at the bottom right, to
//! 0x138 ("voltar", back) if `dword_75AADC` is 1 and to 0x139 ("continuar") otherwise, as all four
//! results screens do; its background comes from `menu::load_background` (0x47EFFF), after which a
//! set carry flag is fatal error 0x80712345 (the flag is whatever the last blit in there left);
//! leaving is asked about, and a "no" sets the screen up again. One caller, `sub_4A1F7A`.

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
        .class_stage_results
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
            .class_stage_results
            .id()
    },
    saved_item: || &crate::menu::owner::menus().screens.items.current[slot::CLASS_STAGE_RESULTS], // dword_747780
    lists: &[
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.scene_list, // dword_755504
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .class_stage_results_scene
                    .id()
            }, // unk_7590D2
            run: false,
        },
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.draw_list,
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .list_758e12
                    .id()
            }, // off_758E12
            run: false,
        },
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.overlay_list, // dword_7554FC
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .class_stage_results_overlay
                    .id()
            }, // dword_759296
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
            .class_stage_results_picture
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
