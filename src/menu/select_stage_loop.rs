//! The stage selection "SELECT STAGE" of the start menus under `menu::single_rally`.
//! Screen-specific part of `menu::screen_loop`: leaving is asked about; left without the
//! question, the highlight is not saved; the next frame begins with the steps after the first
//! frame; the question is not put while the lamps flag is 1.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::owner::Tables;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Asking, Callee, Flow, Frames, List, MenuSound, Screen, ScreenVar, Step,
};

fn after_first_frame(tables: &Tables) -> Result<Flow, Refused> {
    if tables.captions.read().left_by_key {
        tables.captions.write().left_by_key = false;
        return Ok(Flow::SetUp);
    }
    Ok(Flow::On)
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.select_stage_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.select_stage_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list,
            table: |_| DRAW_LISTS.select_stage_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.select_stage_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::Write(ScreenVar::YesNoAnswer, 0x0)],
    set_up_first: &[Step::Call(Callee::MarkStageChoice)],
    background: &[Step::Call(Callee::DrawStageMap)],
    after_first_frame: &[Step::DoTablesFlow(after_first_frame)],
    after: &[Step::Sound(MenuSound::None)],
    asking: Some(Asking {
        unless: Some(|owners| u32::from(owners.net.race_slot.is_on())),
        unasked_skips_save: true,
        ..Asking::DEFAULT
    }),
    frames: Frames {
        next_frame_repeats_first: true,
        ..Frames::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.select_stage.id(),
        slot::SELECT_STAGE,
    )
};
