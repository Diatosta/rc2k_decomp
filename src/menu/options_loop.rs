//! The loop of the options menu. Screen-specific part of `menu::screen_loop`: background
//! `opts.png`; leaving is not asked about; after an action it is left or set up again, nothing
//! else.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    ActionCheck, Callee, Entry, Keys, List, Screen, ScreenVar, Step, carousel, prepare_scene,
    screen_entry,
};

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let menus = entry.menus;
    crate::strings::units::set_units(
        entry.owners.strings,
        menus.play.slots.records[0].get().metric_units,
    );
    entry.set_view(-156, 39, 0x140);
    prepare_scene(&entry.owners);
    menus.screens.flow.turning.set(0x1u32);
    menus.screens.carousel.menu.set(0x7u32);
    carousel(entry, None)?;
    let item = menus.screens.items.current[slot::OPTIONS].get();
    menus.screens.loop_flags.item.set(item);
    menus.screens.carousel.front.set(item.wrapping_sub(1));
    menus.screens.flow.turning.set(0x0u32);
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.options_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.options_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.options_overlay.id(),
            run: false,
        },
    ],
    set_up_first: &[Step::Do(set_up_first)],
    background: &[
        Step::Write(ScreenVar::ClearsEscapeFlag, 0x1),
        Step::Call(Callee::LoadBackground(|m| &m.texts.options_picture)),
    ],
    keys: Keys {
        action_check: ActionCheck::Short,
        ..Keys::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.options.id(),
        slot::OPTIONS,
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, ScreenId::Options)
}
