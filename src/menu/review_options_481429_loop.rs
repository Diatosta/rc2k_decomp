//! `menu::review_options_481429_loop` (0x481429): the menu after the results screens, by its
//! title "OPÇÕES DE REVISÃO" (a reading)
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `roptions.png`; leaving is asked about; also
//! when `dword_755569` is 1; the Escape flag is looked at at the top of every frame; around an
//! action the highlight is kept in `dword_7477A4`; the action run is that of the item of
//! another menu, 0x754C88 (the original's slip); the steps that are more than a call or a store
//! are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "OPÇÕES DE REVISÃO", not yet a
//! visit.
//! Callers: `sub_4A0E60`, `sub_4AA4D7`.

use crate::errors::Refused;
use crate::framework::rva;
use crate::menu::captions_store::CAPTIONS;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::menu::menus_store::MENUS;
use crate::menu::screen_items_store::{SCREEN_ITEMS, slot};
use crate::menu::screen_loop::{Callee, Carried, List, Screen, Step, carousel, prepare_scene};
use crate::menu::state_store::MENU_STATE;
use crate::scene::draw_store::DRAW;

// ---- Rust version: `menu::screen_loop` with this ----

unsafe fn set_up_first(c: &mut Carried) -> Result<(), Refused> {
    unsafe {
        Callee::NoOp.run(c)?;
        LOOP_FLAGS.ended_a.set(0x0u32);
        SCREEN_FLOW.turning.set(0x1u32);
        MENU_STATE.carousel.menu.set(0x14u32);
        carousel(c)?;
        c.arg = SCREEN_ITEMS.current[slot::REVIEW_OPTIONS_481429].get();
        LOOP_FLAGS.item.set(c.arg);
        c.arg = c.arg.wrapping_sub(1);
        MENU_STATE.carousel.front.set(c.arg);
        SCREEN_FLOW.turning.set(0x0u32);
        DRAW.view_offset[0].set(0xFFFFFF64u32);
        DRAW.view_offset[1].set(0x27u32);
        DRAW.view_zoom.set(0x140u32);
        prepare_scene(c);
    }
    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: || MENUS.review_options_481429.addr() as usize,
    saved_item: || &SCREEN_ITEMS.current[slot::REVIEW_OPTIONS_481429],
    lists: &[
        List {
            pointer: || &SCREEN_FLOW.scene_list, // g_sceneList
            table: || DRAW_LISTS.review_options_481429_scene.addr() as usize,
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.draw_list, // g_drawList
            table: || DRAW_LISTS.review_options_481429_draw.addr() as usize,
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.overlay_list, // g_overlayList
            table: || DRAW_LISTS.review_options_481429_overlay.addr() as usize,
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[Step::Write(|| &DRAW_WORK.escaped, 0x0)],
    set_up_first: &[Step::Do(set_up_first)],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::ArgAddr(|| {
            (crate::menu::texts_store::MENU_TEXTS
                .review_options_481429_picture
                .addr()) as usize
        }),
        Step::CallOrFatal(Callee::LoadBackground, 0x80712345),
    ],
    after_first_frame: &[],
    confirm: Some(&[]),
    ask_also_if: Some(|| &LOOP_FLAGS.ended_a),
    confirmed: &[
        Step::Write(|| &LOOP_FLAGS.ended_a, 0x0),
        Step::Write(|| &CAPTIONS.screen_left, 0x0),
    ],
    unasked_skips_save: false,
    after: &[Step::Call(Callee::NoOp)],
    escape_leaves: true,
    each_frame: &[],
    short_action_check: false,
    leave_at_once: None,
    next_frame_repeats_first: false,
    action_saved_item: Some(|| &SCREEN_ITEMS.current[slot::REVIEW_OPTIONS_48108E]),
    action_menu: Some(|| MENUS.review_options_48108e.addr() as usize),
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
