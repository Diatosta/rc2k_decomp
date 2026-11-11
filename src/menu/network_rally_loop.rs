//! `menu::network_rally_loop` (0x475F1A): the network rally menu, by its title "RALI EM REDE"
//! (named from its title string and callers; the original's name is lost)
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `nral.png`; leaving is not asked about; the
//! quit switch is not looked at; the steps that are more than a call or a store are the
//! functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "RALI EM REDE", not yet a visit.
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, Frames, List, MenuSound, Screen, Step, carousel, screen_entry,
};

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    screens.flow.turning.set(0x1u32);
    screens.state.carousel.menu.set(0x6u32);
    carousel(None)?;
    let item = screens.items.current[slot::NETWORK_RALLY].get();
    screens.loop_flags.item.set(item);
    screens.state.carousel.front.set(item.wrapping_sub(1));

    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.network_rally_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.network_rally_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.network_rally_overlay.id(),
            run: false,
        },
    ],
    set_up_first: &[Step::Do(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.network_rally_picture
    }))],
    after_first_frame: &[
        Step::WriteScene(
            |_| &crate::scene::polygon_work::front_end().window.view_offset[0],
            0x7,
        ),
        Step::WriteScene(
            |_| &crate::scene::polygon_work::front_end().window.view_offset[1],
            0xA,
        ),
        Step::WriteScene(
            |_| &crate::scene::polygon_work::front_end().window.view_zoom,
            0x100,
        ),
        Step::Call(Callee::PrepareScene),
    ],
    after: &[
        Step::Write(|m| &m.tables.captions.left_by_key, 0x0),
        Step::Write(|m| &m.screens.loop_flags.ask_leave, 0x0),
        Step::Write(|m| &m.screens.draw_work.escape_pressed, 0x0),
        Step::Sound(MenuSound::None),
        Step::Write(|m| &m.screens.loop_flags.leave_screen, 0x0),
    ],
    frames: Frames {
        no_quit_check: true,
        ..Frames::DEFAULT
    },
    ..Screen::new(
        |m| m.tables.menus.network_rally.id(),
        |m| &m.screens.items.current[slot::NETWORK_RALLY],
    )
};

/// Runs the screen (a refusal is the caller's).
pub fn run() -> Result<(), crate::errors::Refused> {
    screen_entry("menu::network_rally_loop", &SCREEN)
}
