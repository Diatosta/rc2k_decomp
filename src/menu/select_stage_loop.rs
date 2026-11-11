//! `menu::select_stage_loop` (0x482099): the stage selection "ESCOLHA ETAPA" of the start menus
//! under `menu::single_rally`.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: leaving is asked about; left without the question, the
//! highlight is not saved; the next frame begins with the steps after the first frame; the
//! question is not put while `dword_52A910` is 1; the steps that are more than a call or a
//! store are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "ESCOLHA ETAPA", not yet a visit.
//! Rust callers run its `entry` (`routes.rs`).

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Asking, Callee, Entry, Flow, Frames, List, MenuSound, Screen, Step,
};
use crate::net::owner::net;

fn after_first_frame(entry: &Entry) -> Result<Flow, Refused> {
    let tables = &entry.menus.tables;
    if tables.captions.left_by_key.get() == 0x1 {
        tables.captions.left_by_key.set(0x0u32);
        return Ok(Flow::SetUp);
    }
    Ok(Flow::On)
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.select_stage_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.select_stage_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list, // dword_755500
            table: |m| m.tables.draw_lists.select_stage_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.select_stage_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::Write(|m| &m.tables.pages.yes_no_answer, 0x0)],
    set_up_first: &[Step::Call(Callee::MarkStageChoice)],
    background: &[Step::Call(Callee::DrawStageMap)],
    after_first_frame: &[Step::DoFlow(after_first_frame)],
    after: &[Step::Sound(MenuSound::None)],
    asking: Some(Asking {
        unless: Some(|_| net().vars.session.get()),
        unasked_skips_save: true,
        ..Asking::DEFAULT
    }),
    frames: Frames {
        next_frame_repeats_first: true,
        ..Frames::DEFAULT
    },
    ..Screen::new(
        |m| m.tables.menus.select_stage.id(),
        |m| &m.screens.items.current[slot::SELECT_STAGE],
    )
};
