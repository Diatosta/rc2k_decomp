//! `menu::view_stage_loop` (0x47FD37): the stage details opened from the service area of a
//! championship, by its title "VER ETAPA" and its draw list, which is that of
//! `screens::draw_service_stage_info` (named from its title string and callers; the original's name is lost)
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: leaving is asked about; it has a step of its own once a
//! frame; the steps that are more than a call or a store are the functions below, generated
//! from the listing.
//! The name is a reading from the title string of its menu, "VER ETAPA", not yet a visit.

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, Entry, Frames, List, Screen, Step};

// The original pushes `dword_75595C` and `dword_755958` at its entry and pops them at its
// end, with the whole screen in between: the two are kept here across the loop's steps
// (written by hand; the generated functions each have a stack of their own).
thread_local! {
    static KEPT: std::cell::RefCell<Vec<u32>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn take_kept() -> u32 {
    KEPT.with_borrow_mut(|stack| stack.pop().expect("a pop without its push"))
}

fn before(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    KEPT.with_borrow_mut(|stack| {
        stack.push(screens.event.section.get());
    });
    KEPT.with_borrow_mut(|stack| stack.push(screens.event.index.get()));
    screens
        .event
        .viewed_section
        .set(screens.event.section.get());
    screens
        .event
        .first_page_stage
        .set(screens.event.index.get());

    Ok(())
}

fn after_lists(entry: &Entry) -> Result<(), Refused> {
    let menus = entry.menus;
    Callee::CountStagePages.run(entry)?;
    Callee::StyleChosenStageItem.run(entry)?;
    if menus.screens.event.stage_pages.get() == 0x0 {
        Callee::StyleStageItem.run(entry)?;
    }
    menus.play.champ_view.stage_mode.set(0x3u32);
    Ok(())
}

fn after(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    screens.event.index.set(take_kept());
    screens.event.section.set(take_kept());
    Callee::StopStream.run(entry)?;
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.view_stage_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.view_stage_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.view_stage_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::Do(before)],
    after_lists: &[Step::Do(after_lists)],
    background: &[Step::Call(Callee::DrawStageMap)],
    after_first_frame: &[Step::Call(Callee::PlayStageSpeech)],
    after: &[Step::Do(after)],
    asking: Some(Asking { ..Asking::DEFAULT }),
    frames: Frames {
        each_frame: &[Step::Call(Callee::StepCarStream)],
        ..Frames::DEFAULT
    },
    ..Screen::new(
        |m| m.tables.menus.view_stage.id(),
        |m| &m.screens.items.current[slot::VIEW_STAGE],
    )
};
