//! The network rally menu, by its title "RALI EM REDE". Screen-specific part of
//! `menu::screen_loop`: background `nral.png`; leaving is not asked about; the quit switch is not
//! looked at.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, Frames, List, MenuSound, Screen, Step, carousel, screen_entry,
};
use crate::menu::texts_store::texts;

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    screens.flow.turning.set(0x1u32);
    screens.state.carousel.menu.set(0x6u32);
    carousel(entry, None)?;
    let item = screens.items.current[slot::NETWORK_RALLY].get();
    screens.loop_flags.item.set(item);
    screens.state.carousel.front.set(item.wrapping_sub(1));

    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.network_rally_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.network_rally_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.network_rally_overlay.id(),
            run: false,
        },
    ],
    set_up_first: &[Step::Do(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|_| {
        &texts().network_rally_picture
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
        Step::WriteFlag(|m| &m.tables.captions.left_by_key, false),
        Step::WriteFlag(|m| &m.screens.loop_flags.ask_leave, false),
        Step::WriteFlag(|m| &m.screens.draw_work.escape_pressed, false),
        Step::Sound(MenuSound::None),
        Step::WriteFlag(|m| &m.screens.loop_flags.leave_screen, false),
    ],
    frames: Frames {
        no_quit_check: true,
        ..Frames::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.network_rally.id(),
        |m| &m.screens.items.current[slot::NETWORK_RALLY],
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, &SCREEN)
}
