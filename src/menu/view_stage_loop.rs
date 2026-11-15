//! The stage details opened from the service area of a championship, by its title "VER ETAPA"
//! and its draw list, which is that of `screens::draw_service_stage_info`. Screen-specific part
//! of `menu::screen_loop`: leaving is asked about; it has a step of its own once a frame.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::owner::Screens;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, Entry, Frames, List, Screen, Step};

// `event.section` and `event.stage_in_section` are saved when the screen is entered and put back
// when it ends, kept in `Screens::stage_view_kept` across the loop's steps.
fn take_kept(screens: &Screens) -> u32 {
    let mut stack = screens
        .stage_view_kept
        .lock()
        .expect("the kept stack's lock");
    stack.pop().expect("a pop without its push")
}

fn before(screens: &Screens) -> Result<(), Refused> {
    let mut stack = screens
        .stage_view_kept
        .lock()
        .expect("the kept stack's lock");
    stack.push(screens.event.section.get());
    stack.push(screens.event.stage_in_section.get());
    drop(stack);
    screens
        .event
        .stage
        .first_on_page
        .set(screens.event.stage_in_section.get());

    Ok(())
}

fn after_lists(entry: &Entry) -> Result<(), Refused> {
    let menus = entry.menus;
    Callee::CountStagePages.run(entry)?;
    Callee::StyleChosenStageItem.run(entry)?;
    if menus.screens.event.stage.pages.get() == 0x0 {
        Callee::StyleStageItem.run(entry)?;
    }
    menus.play.champ_view.stage_mode.set(0x3u32);
    Ok(())
}

fn after(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    screens.event.stage_in_section.set(take_kept(screens));
    screens.event.section.set(take_kept(screens));
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
    before: &[Step::DoScreens(before)],
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
        slot::VIEW_STAGE,
    )
};
