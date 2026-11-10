//! `menu::welcome_loop` (0x4721F0): the screen titled "BEM-VINDO" that the championship's event
//! loops show.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: leaving is asked about; something runs before the
//! question is put.
//! The name is a reading from the title string of its menu, "BEM-VINDO", not yet a visit.
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, FrameRoutine, List, Screen, Step};

pub const SCREEN: Screen = Screen {
    menu: |m| m.tables.menus.welcome.id(),
    saved_item: |m| &m.screens.items.current[slot::WELCOME],
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.welcome_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.welcome_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.welcome_overlay.id(),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: FrameRoutine::Menu,
    later_frame_callback: None,
    before: &[Step::Write(|m| &m.screens.draw_work.escaped, 0x0)],
    set_up_first: &[Step::Write(|m| &m.tables.pages.yes_no_answer, 0x0)],
    prepare: &[],
    after_lists: &[],
    background: &[Step::Call(Callee::LoadWelcomePicture)],
    after_first_frame: &[Step::Call(Callee::PlayRallySpeech)],
    confirm: Some(&[]),
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[Step::Call(Callee::StopStream)],
    escape_leaves: false,
    each_frame: &[],
    short_action_check: false,
    leave_at_once: None,
    next_frame_repeats_first: false,
    action_saved_item: None,
    action_menu: None,
    no_quit_check: false,
    before_asking: &[Step::Call(Callee::StopStream)],
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
