//! The stage details screen before a stage of a championship, by its title "ETAPA SEGUINTE" and
//! its draw list, which is that of `screens::draw_stage_info`. Screen-specific part of
//! `menu::screen_loop`: leaving is asked about; it has a step of its own once a frame.

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, Entry, Frames, List, Screen, Step, screen_entry};

fn before(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    screens.items.current[slot::NEXT_STAGE].set(screens.items.first[slot::NEXT_STAGE].get());
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |m| m.tables.draw_lists.next_stage_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |m| m.tables.draw_lists.next_stage_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |m| m.tables.draw_lists.next_stage_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::Do(before)],
    after_lists: &[Step::Write(|m| &m.play.champ_view.stage_mode, 0x3)],
    background: &[Step::Call(Callee::DrawStageMap)],
    after_first_frame: &[Step::Call(Callee::PlayStageSpeech)],
    after: &[Step::Call(Callee::StopStream)],
    asking: Some(Asking { ..Asking::DEFAULT }),
    frames: Frames {
        each_frame: &[Step::Call(Callee::StepCarStream)],
        ..Frames::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.next_stage.id(),
        |m| &m.screens.items.current[slot::NEXT_STAGE],
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, &SCREEN)
}
