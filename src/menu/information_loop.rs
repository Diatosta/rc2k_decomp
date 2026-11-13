//! The loop of the information screen. Screen-specific part of `menu::screen_loop`: background
//! `info.png`; leaving is not asked about.

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, List, Screen, Step, carousel, log_line, prepare_scene, screen_entry,
};

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    log_line(c"InfoScr: Start");
    entry.set_view(-156, 39, 0x140);
    prepare_scene(entry);
    entry.set_clip_sides(-40, 760);
    screens.flow.turning.set(0x1u32);
    screens.state.carousel.menu.set(0x9u32);
    carousel(entry, None)?;
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
            pointer: |m| &m.screens.flow.scene_list,
            table: |m| m.tables.draw_lists.information_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |m| m.tables.draw_lists.information_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
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

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, "menu::information_loop", &SCREEN)
}
