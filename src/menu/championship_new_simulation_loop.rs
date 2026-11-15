//! The start screen of the normal championship, by its title "NOVA SIMULAÇÃO" and its place in the
//! mode menu. Screen-specific part of `menu::screen_loop`: background `sral.png`; leaving is not
//! asked about; the next frame begins with the steps after the first frame.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::owner::Tables;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, Flow, Frames, List, Screen, ScreenFlag, Step, carousel, prepare_scene,
    screen_entry,
};
use crate::menu::texts_store::texts;

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let menus = entry.menus;
    menus.tables.captions.left_by_key.set(false);
    menus.screens.flow.turning.set(0x1u32);
    menus.screens.carousel.menu.set(0x2u32);
    carousel(entry.menus, None)?;
    let item = menus.screens.items.current[slot::CHAMPIONSHIP_NEW_SIMULATION].get();
    menus.screens.loop_flags.item.set(item);
    menus.screens.carousel.front.set(item.wrapping_sub(1));
    entry.set_view(7, 10, 0x100);
    prepare_scene(&entry.owners);
    Ok(())
}

fn after_first_frame(tables: &Tables) -> Result<Flow, Refused> {
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
            table: |_| DRAW_LISTS.championship_new_simulation_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.championship_new_simulation_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.championship_new_simulation_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::WriteFlag(ScreenFlag::EventOver, false)],
    set_up_first: &[Step::Do(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|_| {
        &texts().championship_new_simulation_picture
    }))],
    after_first_frame: &[Step::DoTablesFlow(after_first_frame)],
    frames: Frames {
        next_frame_repeats_first: true,
        ..Frames::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.championship_new_simulation.id(),
        slot::CHAMPIONSHIP_NEW_SIMULATION,
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, ScreenId::ChampionshipNewSimulation)
}
