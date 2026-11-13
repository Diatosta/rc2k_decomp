//! The loop of the screen where a car setup is saved and its name typed. Screen-specific part of
//! `menu::screen_loop`: leaving is not asked about; it has a step of its own once a frame.

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, Flow, Frames, List, Screen, Step, prepare_scene, screen_entry,
};

fn prepare(entry: &Entry) -> Result<(), Refused> {
    entry.set_view(0, 0, 0x100);
    prepare_scene(entry);
    entry.set_clip_sides(0, 800);
    Ok(())
}

fn each_frame(entry: &Entry) -> Result<Flow, Refused> {
    let screens = &entry.menus.screens;
    if screens.option_work.typing.is_set() {
        Callee::EditSetupName.run(entry)?;
        if screens.option_work.typing.is_set() {
            return Ok(Flow::NextFrame);
        }
        screens.draw_work.action_mode.set(0x2u32);
        return Ok(Flow::Leave);
    }
    Ok(Flow::On)
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |m| m.tables.draw_lists.save_setup_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |m| m.tables.draw_lists.save_setup_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |m| m.tables.draw_lists.save_setup_overlay.id(),
            run: false,
        },
    ],
    set_up_first: &[Step::Write(|m| &m.screens.flow.left_service, 0x0)],
    prepare: &[Step::Do(prepare)],
    after: &[Step::Write(|m| &m.screens.flow.left_service, 0x1)],
    frames: Frames {
        each_frame: &[Step::DoFlow(each_frame)],
        ..Frames::DEFAULT
    },
    ..Screen::new(
        |m| m.tables.menus.save_setup.id(),
        |m| &m.screens.items.current[slot::SAVE_SETUP],
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, "menu::save_setup_loop", &SCREEN)
}
