//! The loop of the screen where a car setup is saved and its name typed. Screen-specific part of
//! `menu::screen_loop`: leaving is not asked about; it has a step of its own once a frame.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, Flow, Frames, List, Screen, ScreenVar, SetUp, Step, screen_entry,
};

fn prepare(set_up: &SetUp) -> Result<(), Refused> {
    set_up.set_view(0, 0, 0x100);
    set_up.prepare_scene();
    set_up.set_clip_sides(0, 800);
    Ok(())
}

fn each_frame(entry: &Entry) -> Result<Flow, Refused> {
    let screens = &entry.menus.screens;
    if screens.entry.read().typing {
        Callee::EditSetupName.run(entry)?;
        if screens.entry.read().typing {
            return Ok(Flow::NextFrame);
        }
        screens.draw_work.write().action_mode = 0x2u32;
        return Ok(Flow::Leave);
    }
    Ok(Flow::On)
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.save_setup_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.save_setup_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.save_setup_overlay.id(),
            run: false,
        },
    ],
    set_up_first: &[Step::Write(ScreenVar::LeftService, 0x0)],
    prepare: &[Step::DoSetUp(prepare)],
    after: &[Step::Write(ScreenVar::LeftService, 0x1)],
    frames: Frames {
        each_frame: &[Step::DoFlow(each_frame)],
        ..Frames::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.save_setup.id(),
        slot::SAVE_SETUP,
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, ScreenId::SaveSetup)
}
