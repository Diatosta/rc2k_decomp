//! The car choice's own steps: the current player's car found in the
//! carousel's entries, and the carousel turned to the next or the previous car.

use crate::errors::{BadInput, Refused};
use crate::menu::lists_store::CarListParts;
use crate::menu::owner::Menus;
use crate::menu::player::player;

const MOST_ENTRIES: u32 = 0x100;

/// `menu::find_player_car`: the front of the carousel and the shown car the index of the
/// entry whose car (+0) is the current player's `car_number`, 0 when none is (the first
/// entry is looked at before the end mark -1). Refuses a current player past four, no
/// entries, or entries without their end within 0x100.
pub fn find_player_car(menus: &Menus) -> Result<(), Refused> {
    let current: u32 = menus.screens.event.read().current_player;
    let car = player(menus, current)
        .ok_or(BadInput("a current player past the four"))?
        .get()
        .car_number;
    let Some(list) = menus.screens.carousel.list()? else {
        return Err(BadInput("no carousel entries").into());
    };
    let entry = |n: u32| menus.lists.car_entry(CarListParts::of(menus), list, n);
    if entry(0)? == u32::MAX {
        return Err(BadInput("no carousel entries").into());
    }
    // The logic is in `rc2k_core::menu::cars::find_car`, over the entries' cars.
    let mut cars = Vec::new();
    let mut n = 0;
    loop {
        cars.push(crate::menu::cars_store::description(&menus.cars.table, entry(n)?)?.car);
        if entry(n + 1)? == u32::MAX {
            break;
        }
        n += 1;
        if n >= MOST_ENTRIES {
            return Err(BadInput("carousel entries without their end").into());
        }
    }
    let found = rc2k_core::menu::cars::find_car(&cars, car);
    menus.screens.carousel.write().front = found;
    menus.screens.event.write().shown.car = found;

    Ok(())
}

/// `menu::next_choice_car`: unless the carousel has one car, a turn already going the other way
/// is not counted: with the direction 1 the front moves on (round the count); then the
/// direction 1, the angle 0, the shown car on (after the last entry to 0), and
/// `LoopFlags.leave_screen` 0.
/// The carousel turned (`menu::next_choice_car` / `previous_choice_car`). The logic is in
/// `rc2k_core::menu::cars::turn_carousel`.
fn turn(menus: &Menus, on: bool) {
    use rc2k_core::menu::cars::{Carousel, turn_carousel};
    use rc2k_core::menu::navigation::Turn;

    let c = turn_carousel(
        Carousel {
            turn: Turn {
                count: menus.screens.carousel.read().count as i32,
                value: menus.screens.carousel.read().front as i32,
                way: menus.screens.carousel.read().direction,
                step: menus.screens.carousel.read().angle,
            },
            shown: menus.screens.event.read().shown.car,
            last: menus.cars.table.read().last_car,
        },
        on,
    );
    menus.screens.carousel.write().front = c.turn.value as u32;
    menus.screens.carousel.write().direction = c.turn.way;
    menus.screens.carousel.write().angle = c.turn.step;
    menus.screens.event.write().shown.car = c.shown;
    menus.screens.loop_flags.write().leave_screen = false;
}

pub fn next_choice_car(menus: &Menus) {
    turn(menus, true);
}

pub fn previous_choice_car(menus: &Menus) {
    turn(menus, false);
}
