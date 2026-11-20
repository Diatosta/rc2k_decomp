//! The screen titled "CAR SPECIFICATIONS". Screen-specific part of `menu::screen_loop`:
//! leaving is not asked about; it has a step of its own once a frame, and one after the loop
//! (the shown car kept for the cheats).

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::owner::Menus;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, Frames, List, Screen, SetUp, Step, screen_entry};

fn before(menus: &Menus) -> Result<(), Refused> {
    menus.screens.event.write().shown.car = menus.play.cheats.read().kept_car;
    menus.screens.event.write().shown.car_page = 0x0u32;

    Ok(())
}

fn set_up_first(set_up: &SetUp) -> Result<(), Refused> {
    let screens = &set_up.menus.screens;
    set_up.set_view(-160, -100, 0x100);
    set_up.prepare_scene();
    screens.flow.write().turning = 0x1u32;
    screens.carousel.menu.set(0xBu32);
    set_up.carousel_with(None)?;
    let item = screens.items.current(slot::CAR_SPECIFICATION);
    screens.loop_flags.write().item = item;
    screens.carousel.front.set(item.wrapping_sub(1));
    screens.flow.write().turning = 0x0u32;
    screens.carousel.front.set(screens.event.read().shown.car);
    Ok(())
}

fn after(menus: &Menus) -> Result<(), Refused> {
    menus.play.cheats.write().kept_car = menus.screens.event.read().shown.car;

    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.car_specification_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.car_specification_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list,
            table: |_| DRAW_LISTS.car_specification_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.car_specification_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::DoMenus(before)],
    set_up_first: &[Step::DoSetUp(set_up_first)],
    background: &[
        Step::Call(Callee::LoadCarBackground),
        Step::Call(Callee::PlayCarStream),
    ],
    after: &[Step::DoMenus(after)],
    after_restored: &[Step::Call(Callee::StopStream)],
    frames: Frames {
        each_frame: &[Step::Call(Callee::StepCarStream)],
        ..Frames::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.car_specification.id(),
        slot::CAR_SPECIFICATION,
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, ScreenId::CarSpecification)
}
