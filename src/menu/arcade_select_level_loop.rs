//! The mode menu's third entry, by its title "ESCOLHA NÍVEL". Screen-specific part of
//! `menu::screen_loop`: background `Selarc.png`; leaving is not asked about; the next frame begins
//! with the steps after the first frame.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, Flow, Frames, List, MenuSound, Screen, Step, carousel, prepare_scene,
    screen_entry,
};
use crate::menu::texts_store::texts;

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    entry.set_view(7, 10, 0x100);
    prepare_scene(entry);
    screens.flow.turning.set(0x1u32);
    screens.state.carousel.menu.set(0x5u32);
    carousel(entry, None)?;
    let item = screens.items.current[slot::ARCADE_SELECT_LEVEL].get();
    screens.loop_flags.item.set(item);
    screens.state.carousel.front.set(item.wrapping_sub(1));
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

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.arcade_select_level_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.arcade_select_level_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.arcade_select_level_overlay.id(),
            run: false,
        },
    ],
    before: &[
        Step::WriteFlag(|m| &m.tables.captions.left_by_key, false),
        Step::WriteFlag(|m| &m.screens.event.event_over, false),
    ],
    set_up_first: &[Step::Do(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|_| {
        &texts().arcade_select_level_picture
    }))],
    after_first_frame: &[Step::DoFlow(after_first_frame)],
    after: &[Step::Sound(MenuSound::None)],
    frames: Frames {
        next_frame_repeats_first: true,
        ..Frames::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.arcade_select_level.id(),
        |m| &m.screens.items.current[slot::ARCADE_SELECT_LEVEL],
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, &SCREEN)
}
