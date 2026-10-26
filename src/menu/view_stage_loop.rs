//! `menu::view_stage_loop` (0x47FD37): the stage details opened from the service area of a
//! championship, by its title "VER ETAPA" and its draw list, which is that of
//! `screens::draw_service_stage_info` (a reading)
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: leaving is asked about; it has a step of its own once a
//! frame; the steps that are more than a call or a store are the functions below, generated
//! from the listing.
//! The name is a reading from the title string of its menu, "VER ETAPA", not yet a visit.
//! Callers: `sub_47037E`.

use crate::errors::Refused;
use crate::framework::rva;
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::menus_store::MENUS;
use crate::menu::screen_items_store::{SCREEN_ITEMS, slot};
use crate::menu::screen_loop::{Callee, Carried, List, Screen, Step};

// ---- Rust version: `menu::screen_loop` with this ----

// The original pushes `dword_75595C` and `dword_755958` at its entry and pops them at its
// end, with the whole screen in between: the two are kept here across the loop's steps
// (written by hand; the generated functions each have a stack of their own).
thread_local! {
    static KEPT: std::cell::RefCell<Vec<u32>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn take_kept() -> u32 {
    KEPT.with_borrow_mut(|stack| stack.pop().expect("a pop without its push"))
}

fn before(c: &mut Carried) -> Result<(), Refused> {
    KEPT.with_borrow_mut(|stack| stack.push(EVENT_STATE.section.get()));
    KEPT.with_borrow_mut(|stack| stack.push(EVENT_STATE.index.get()));
    c.arg = EVENT_STATE.section.get();
    EVENT_STATE.dword_755964.set(c.arg);
    c.arg = EVENT_STATE.index.get();
    EVENT_STATE.first_page_stage.set(c.arg);

    Ok(())
}

unsafe fn after_lists(c: &mut Carried) -> Result<(), Refused> {
    unsafe {
        Callee::CountStagePages.run(c)?;
        Callee::StyleChosenStageItem.run(c)?;
        if EVENT_STATE.stage_pages.get() == 0x0 {
            Callee::StyleStageItem.run(c)?;
        }
        CHAMP_VIEW.stage_mode.set(0x3u32);
    }
    Ok(())
}

unsafe fn after(c: &mut Carried) -> Result<(), Refused> {
    unsafe {
        EVENT_STATE.index.set(take_kept());
        EVENT_STATE.section.set(take_kept());
        Callee::StopStream.run(c)?;
    }
    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: || MENUS.view_stage.addr(),
    saved_item: || &SCREEN_ITEMS.current[slot::VIEW_STAGE],
    lists: &[
        List {
            pointer: || &SCREEN_FLOW.scene_list, // g_sceneList
            table: || DRAW_LISTS.view_stage_scene.addr(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.draw_list, // g_drawList
            table: || DRAW_LISTS.view_stage_draw.addr(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.overlay_list, // g_overlayList
            table: || DRAW_LISTS.view_stage_overlay.addr(),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[Step::Do(before)],
    set_up_first: &[],
    prepare: &[],
    after_lists: &[Step::Do(after_lists)],
    background: &[Step::Call(Callee::DrawStageMap)],
    after_first_frame: &[Step::Call(Callee::PlayStageSpeech)],
    confirm: Some(&[]),
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[Step::Do(after)],
    escape_leaves: false,
    each_frame: &[Step::Call(Callee::StepCarStream)],
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
