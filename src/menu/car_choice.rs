//! The car choice's own steps (IDA, 2026-10-15): the current player's car found in the
//! carousel's entries, and the carousel turned to the next or the previous car.

use crate::errors::{BadInput, Refused};
use crate::menu::player::player;

const MOST_ENTRIES: u32 = 0x100;

/// `menu::find_player_car`: the front of the carousel and the shown car the index of the
/// entry whose car (+0) is the current player's `car_number`, 0 when none is (the first
/// entry is looked at before the end mark -1). Refuses a current player past four, no
/// entries, or entries without their end within 0x100.
pub fn find_player_car() -> Result<(), Refused> {
    let current: u32 = crate::menu::owner::menus()
        .screens
        .event
        .current_player
        .get();
    let car = player(current)
        .ok_or(BadInput("a current player past the four"))?
        .get()
        .car_number;
    let Some(list) = crate::menu::owner::menus().screens.state.carousel.list()? else {
        return Err(BadInput("no carousel entries").into());
    };
    let entries = crate::menu::owner::menus().tables.lists.cars(list);
    let entry = |n: u32| entries.offset(4 * n).read::<u32>("a carousel entry");
    if entry(0)? == u32::MAX {
        return Err(BadInput("no carousel entries").into());
    }
    // Converted: `rc2k_core::menu::cars::find_car`, over the entries' cars.
    let mut cars = Vec::new();
    let mut n = 0;
    loop {
        cars.push(crate::menu::cars_store::description(entry(n)?)?.get().car);
        if entry(n + 1)? == u32::MAX {
            break;
        }
        n += 1;
        if n >= MOST_ENTRIES {
            return Err(BadInput("carousel entries without their end").into());
        }
    }
    let found = rc2k_core::menu::cars::find_car(&cars, car);
    crate::menu::owner::menus()
        .screens
        .state
        .carousel
        .front
        .set(found);
    crate::menu::owner::menus()
        .screens
        .event
        .shown_car
        .set(found);

    Ok(())
}

/// `menu::next_choice_car`: unless the carousel has one car, a turn already going the other way
/// is not counted: with the direction 1 the front moves on (round the count); then the
/// direction 1, the angle 0, the shown car on (after the last entry to 0), and
/// `dword_755561` 0.
/// The carousel turned (`menu::next_choice_car` / `previous_choice_car`). Converted:
/// `rc2k_core::menu::cars::turn_carousel`.
fn turn(on: bool) {
    use rc2k_core::menu::cars::{Carousel, turn_carousel};
    use rc2k_core::menu::navigation::Turn;

    let c = turn_carousel(
        Carousel {
            turn: Turn {
                count: crate::menu::owner::menus()
                    .screens
                    .state
                    .carousel
                    .count
                    .get() as i32,
                value: crate::menu::owner::menus()
                    .screens
                    .state
                    .carousel
                    .front
                    .get() as i32,
                way: crate::menu::owner::menus()
                    .screens
                    .state
                    .carousel
                    .direction
                    .get(),
                step: crate::menu::owner::menus()
                    .screens
                    .state
                    .carousel
                    .angle
                    .get(),
            },
            shown: crate::menu::owner::menus().screens.event.shown_car.get(),
            last: crate::menu::owner::menus().cars.table.last_car.get(),
        },
        on,
    );
    crate::menu::owner::menus()
        .screens
        .state
        .carousel
        .front
        .set(c.turn.value as u32);
    crate::menu::owner::menus()
        .screens
        .state
        .carousel
        .direction
        .set(c.turn.way);
    crate::menu::owner::menus()
        .screens
        .state
        .carousel
        .angle
        .set(c.turn.step);
    crate::menu::owner::menus()
        .screens
        .event
        .shown_car
        .set(c.shown);
    crate::menu::owner::menus()
        .screens
        .loop_flags
        .back
        .set(0u32);
}

pub fn next_choice_car() {
    turn(true);
}

pub fn previous_choice_car() {
    turn(false);
}
