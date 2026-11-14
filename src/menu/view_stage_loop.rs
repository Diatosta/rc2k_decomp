//! The stage details opened from the service area of a championship, by its title "VER ETAPA"
//! and its draw list, which is that of `screens::draw_service_stage_info`. Screen-specific part
//! of `menu::screen_loop`: leaving is asked about; it has a step of its own once a frame.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, Entry, Frames, List, Screen, Step};

// `event.section` and `event.index` are saved when the screen is entered and put back when it
// ends, kept here across the loop's steps (by hand: the generated steps share no stack).
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
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.view_stage_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.view_stage_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.view_stage_overlay.id(),
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
        module_path!(),
        |m| m.tables.menus.view_stage.id(),
        |m| &m.screens.items.current[slot::VIEW_STAGE],
    )
};
