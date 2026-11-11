//! `menu::information_loop` (0x48E169): the loop of the information screen.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `info.png`; leaving is not asked about; the
//! steps that are more than a call or a store are the functions below, generated from the
//! listing.
//! The name is a reading from the title string of its menu, "INFORMAÇÃO", not yet a visit.
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, List, Screen, Step, carousel, log_line, prepare_scene, screen_entry,
};

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    log_line(c"InfoScr: Start");
    crate::scene::polygon_work::front_end().window.view_offset[0].set(0xFFFFFF64u32);
    crate::scene::polygon_work::front_end().window.view_offset[1].set(0x27u32);
    crate::scene::polygon_work::front_end()
        .window
        .view_zoom
        .set(0x140u32);
    prepare_scene(entry.scene);
    crate::scene::polygon_work::front_end().window.clip_edges[2].set(0xFFFFFFD8u32);
    crate::scene::polygon_work::front_end().window.clip_edges[3].set(0x2F8u32);
    crate::scene::polygon_work::front_end().window.clip_screen[2]
        .set(crate::scene::polygon_work::front_end().window.clip_edges[2].bits() as i32 as f32);
    crate::scene::polygon_work::front_end().window.clip_screen[3]
        .set(crate::scene::polygon_work::front_end().window.clip_edges[3].bits() as i32 as f32);
    screens.flow.turning.set(0x1u32);
    screens.state.carousel.menu.set(0x9u32);
    carousel(None)?;
    let item = screens.items.current[slot::INFORMATION].get();
    screens.loop_flags.item.set(item);
    screens.state.carousel.front.set(item.wrapping_sub(1));
    Ok(())
}

fn after(_: &Entry) -> Result<(), Refused> {
    log_line(c"InfoScr: End");
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.information_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.information_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.information_overlay.id(),
            run: false,
        },
    ],
    set_up_first: &[Step::Do(set_up_first)],
    background: &[
        Step::Write(|m| &m.screens.event.clears_escape_flag, 0x1),
        Step::Call(Callee::LoadBackground(|m| {
            &m.tables.texts.information_picture
        })),
    ],
    after: &[Step::Do(after)],
    ..Screen::new(
        |m| m.tables.menus.information.id(),
        |m| &m.screens.items.current[slot::INFORMATION],
    )
};

/// Runs the screen (a refusal is the caller's).
pub fn run() -> Result<(), crate::errors::Refused> {
    screen_entry("menu::information_loop", &SCREEN)
}
