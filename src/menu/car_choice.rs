//! The car choice's own steps (IDA, 2026-10-15): the current player's car found in the
//! carousel's entries, and the carousel turned to the next or the previous car.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, game_write, rva};
use crate::menu::player::{PLAYERS, player_at};
use crate::menu::step_carousel::{ANGLE_RVA, COUNT_RVA, DIRECTION_RVA, FRONT_RVA};
use crate::menu::take_shown_car::{CURRENT_PLAYER_RVA, SHOWN_CAR_RVA};
use crate::scene::carousel_pictures::ENTRIES_RVA;
use crate::service::harness::{Region, at};

pub const LAST_ENTRY_RVA: usize = rva(0x75649C); // dword_75649C: the entries' last index
pub const CHOSEN_RVA: usize = rva(0x755561); // dword_755561: set to 0 by the turns
const MOST_ENTRIES: u32 = 0x100;

/// What the four write.
pub fn regions() -> [Region; 5] {
    [
        at(FRONT_RVA),
        at(SHOWN_CAR_RVA),
        at(DIRECTION_RVA),
        at(ANGLE_RVA),
        at(CHOSEN_RVA),
    ]
}

/// `menu::find_player_car`: the front of the carousel and the shown car the index of the
/// entry whose car (+0) is the current player's `car_number`, 0 when none is (the first
/// entry is looked at before the end mark -1). Refuses a current player past four, no
/// entries, or entries without their end within 0x100.
pub unsafe fn find_player_car() -> Result<(), Refused> {
    unsafe {
        let current: u32 = game_read(CURRENT_PLAYER_RVA);
        let p = player_at(current);
        if current >= PLAYERS || p == 0 {
            return Err(BadInput("a current player past the four").into());
        }
        let car = (*(p as *const crate::menu::player::Player)).car_number;
        let list: u32 = game_read(ENTRIES_RVA);
        let entry = |n: u32| ((list + 4 * n) as *const u32).read_unaligned();
        if list == 0 || entry(0) == u32::MAX {
            return Err(BadInput("no carousel entries").into());
        }
        let mut n = 0;
        let found = loop {
            if (entry(n) as *const u32).read_unaligned() == car {
                break n;
            }
            if entry(n + 1) == u32::MAX {
                break 0;
            }
            n += 1;
            if n >= MOST_ENTRIES {
                return Err(BadInput("carousel entries without their end").into());
            }
        };
        game_write(FRONT_RVA, found);
        game_write(SHOWN_CAR_RVA, found);
    }
    Ok(())
}

/// `menu::next_choice_car`: unless the carousel has one car, a turn already going the other way
/// is not counted: with the direction 1 the front moves on (round the count); then the
/// direction 1, the angle 0, the shown car on (after the last entry to 0), and
/// `dword_755561` 0.
pub unsafe fn next_choice_car() {
    unsafe {
        let count: i32 = game_read(COUNT_RVA);
        if count != 1 && game_read::<i32>(DIRECTION_RVA) == 1 {
            let front = game_read::<i32>(FRONT_RVA).wrapping_add(1);
            game_write(FRONT_RVA, if front < count { front } else { 0 });
        }
        game_write(DIRECTION_RVA, 1u32);
        game_write(ANGLE_RVA, 0u32);
        let shown: u32 = game_read(SHOWN_CAR_RVA);
        let last: u32 = game_read(LAST_ENTRY_RVA);
        game_write(
            SHOWN_CAR_RVA,
            if shown == last {
                0
            } else {
                shown.wrapping_add(1)
            },
        );
        game_write(CHOSEN_RVA, 0u32);
    }
}

/// `menu::previous_choice_car`: unless the carousel has one car or the direction is 1, the front
/// moves back (below 0 to the count's last); then the direction and the angle -1, the
/// shown car back (from 0 to the last entry), and `dword_755561` 0.
pub unsafe fn previous_choice_car() {
    unsafe {
        let count: i32 = game_read(COUNT_RVA);
        if count != 1 && game_read::<i32>(DIRECTION_RVA) != 1 {
            let front = game_read::<i32>(FRONT_RVA).wrapping_sub(1);
            game_write(
                FRONT_RVA,
                if front < 0 {
                    count.wrapping_sub(1)
                } else {
                    front
                },
            );
        }
        game_write(DIRECTION_RVA, u32::MAX);
        game_write(ANGLE_RVA, u32::MAX);
        let shown: u32 = game_read(SHOWN_CAR_RVA);
        let last: u32 = game_read(LAST_ENTRY_RVA);
        game_write(
            SHOWN_CAR_RVA,
            if shown == 0 {
                last
            } else {
                shown.wrapping_sub(1)
            },
        );
        game_write(CHOSEN_RVA, 0u32);
    }
}
