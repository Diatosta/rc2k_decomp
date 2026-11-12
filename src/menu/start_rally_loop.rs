//! `menu::start_rally_loop` (0x475C12): the mode menu, titled "COMEÇAR RALI".
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `sral.png`; leaving is not asked about; the
//! next frame begins with the steps after the first frame; it has steps of its own first on
//! leaving; the quit switch is not looked at; the steps that are more than a call or a store
//! are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "COMEÇAR RALI", not yet a visit.
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, Flow, Frames, List, Screen, Step, carousel, prepare_scene, screen_entry,
};

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    screens.flow.turning.set(0x1u32);
    screens.state.carousel.menu.set(0x1u32);
    carousel(None)?;
    let item = screens.items.current[slot::START_RALLY].get();
    screens.loop_flags.item.set(item);
    screens.state.carousel.front.set(item.wrapping_sub(1));
    entry.set_view(7, 10, 0x100);
    prepare_scene(entry.scene);
    Ok(())
}

fn after_first_frame(entry: &Entry) -> Result<Flow, Refused> {
    let tables = &entry.menus.tables;
    if tables.captions.left_by_key.is_set() {
        tables.captions.left_by_key.set(false);
        return Ok(Flow::SetUp);
    }
    Ok(Flow::On)
}

fn leaving(entry: &Entry) -> Result<(), Refused> {
    if entry.menus.tables.captions.left_by_key.is_set() {
        Callee::GoBack.run(entry)?;
    }
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.start_rally_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.start_rally_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.start_rally_overlay.id(),
            run: false,
        },
    ],
    set_up_first: &[Step::Do(set_up_first)],
    background: &[
        Step::Write(|m| &m.screens.event.clears_escape_flag, 0x1),
        Step::Call(Callee::LoadBackground(|m| {
            &m.tables.texts.start_rally_picture
        })),
    ],
    after_first_frame: &[Step::DoFlow(after_first_frame)],
    leaving: &[Step::Do(leaving)],
    frames: Frames {
        next_frame_repeats_first: true,
        no_quit_check: true,
        ..Frames::DEFAULT
    },
    ..Screen::new(
        |m| m.tables.menus.start_rally.id(),
        |m| &m.screens.items.current[slot::START_RALLY],
    )
};

/// Runs the screen (a refusal is the caller's).
pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, "menu::start_rally_loop", &SCREEN)
}
