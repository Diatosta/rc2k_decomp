//! `scene::set_up_carousel`: the carousel of the current menu: its entries' kinds, the
//! eight textures of its menu, its size, place and spacing. Takes a list of cars (menus 0xD
//! and 0xE only).
//!
//! 21 menus have their own set-up, all here. Seventeen are only stores
//! (`scene/carousel_set_ups.rs`): the carousel is switched on, the eight textures are set to
//! -1 and then that menu's numbers, the object's size, the entries' kinds, the entry count, the
//! front, angle and direction 0, and the spacing and place.
//! - 0xB (the list of all cars) and 0xD (a list given by the caller): the list is kept as the
//!   carousel's list; for each entry until -1, the car's number (its first dword) picks a kind
//!   from `car_kinds`, which goes to the next entry's kind; the count goes to the carousel's
//!   count (and one less to the cars' last-index variable); then the size (0xF, 0xC), the
//!   front, angle and direction 0, the spacing and place of the cars' carousels, the angle
//!   about x (0x10FF) and the height (0xFFFE6000);
//! - 0xE as 0xD, but first the list is counted from its second entry (its first is not looked
//!   at) and a list of one car has that car copied over its -1 (this shows one car twice: the
//!   list is changed, and is then read on to its next -1);
//! - 0xC the six fixed kinds 0x195 .. 0x1BD with the fixed list of cars.
//!
//! 26 callers (the screen loops' set-up). Refuses, for the cars' carousels, no list, a list
//! without its -1 within 0x100 entries, an entry that is not there, and a car's number of 0x400
//! or more.

use crate::errors::{BadInput, Refused};
use crate::memory::session::DwordLike;
use crate::memory::session::{write_all, write_prefix};
use crate::menu::lists_store::{CarList, CarListParts};
use crate::scene::carousel_set_ups::PLAIN;

// ---- game data ----
const MENUS: u32 = 0x16;
use crate::menu::carousel_store::Carousel;
use rc2k_core::scene::carousel;

/// A dword of the menus' carousel or of its list of textures.
#[derive(Clone, Copy)]
pub enum Field {
    SetUp,
    /// `Carousel::textures[k]`.
    Texture(usize),
    /// `carousel.kinds[k]`.
    Kind(usize),
    Count,
    Front,
    Angle,
    Direction,
    Spacing,
    Y,
    Z,
    Forward,
}

impl Field {
    pub fn cell(self, carousel: &Carousel) -> &dyn DwordLike {
        match self {
            Field::SetUp => &carousel.set_up,
            Field::Texture(k) => &carousel.textures[k],
            Field::Kind(k) => &carousel.kinds[k],
            Field::Count => &carousel.count,
            Field::Front => &carousel.front,
            Field::Angle => &carousel.angle,
            Field::Direction => &carousel.direction,
            Field::Spacing => &carousel.spacing,
            Field::Y => &carousel.y,
            Field::Z => &carousel.z,
            Field::Forward => &carousel.forward,
        }
    }
}

/// One step of a carousel's set-up.
#[derive(Clone, Copy)]
pub enum Op {
    /// A dword of the menus' carousel.
    Set(Field, u32),
    /// A dword of the race block's, found at run time (those statics do not deref in a `const`).
    Run(
        fn(&crate::scene::workers::DrawingScratch) -> &dyn DwordLike,
        u32,
    ),
    /// The eight textures of the menu set to -1.
    Fill,
}

/// The cars' carousels' stores after their entries.
const CARS_TAIL: [Op; 9] = [
    Op::Set(Field::Front, 0),
    Op::Set(Field::Angle, 0),
    Op::Set(Field::Direction, 0),
    Op::Set(Field::Spacing, 0x36BC0),
    Op::Set(Field::Y, 0xFFFF_4000),
    Op::Set(Field::Z, 0x5F000),
    Op::Set(Field::Forward, 0x20000),
    Op::Run(|scratch| &scratch.front_end().object.angles[0], 0x10FF),
    Op::Run(|scratch| &scratch.front_end().object.place[1], 0xFFFE_6000),
];

/// The six kinds of menu 0xC.
const FIXED_KINDS: [u32; 6] = [0x195, 0x199, 0x19F, 0x1A5, 0x1AD, 0x1BD];

/// A store of the tables (never a `Fill`).
fn run_op(carousel: &Carousel, scratch: &crate::scene::workers::DrawingScratch, op: Op) {
    match op {
        Op::Set(field, value) => field.cell(carousel).set(value),
        Op::Run(cell, value) => cell(scratch).set(value),
        Op::Fill => unreachable!("the cars' tail has no fill"),
    }
}

/// Runs `f` with a reader of the list's entries (by number) that answers the list's end
/// where an entry is past the lists, and refuses afterwards: the core's checks take
/// readers that cannot fail.
fn reading<R>(
    parts: CarListParts,
    list: CarList,
    f: impl FnOnce(&dyn Fn(usize) -> u32) -> R,
) -> Result<R, BadInput> {
    let failed = std::cell::Cell::new(None);
    let read = |n: usize| match list_entry(parts, list, n) {
        Ok(word) => word,
        Err(e) => {
            failed.set(Some(e));
            u32::MAX
        }
    };
    let result = f(&read);
    match failed.take() {
        Some(e) => Err(e),
        None => Ok(result),
    }
}

fn list_entry(parts: CarListParts, list: CarList, n: usize) -> Result<u32, BadInput> {
    parts.lists.car_entry(parts, list, n as u32)
}

/// The checks of a list of cars, before anything is written (see
/// `rc2k_core::scene::carousel::check_cars`).
fn check_cars(parts: CarListParts, list: CarList, from_second: bool) -> Result<(), Refused> {
    reading(parts, list, |read| {
        let failed = std::cell::Cell::new(None);
        // A car's number: the first dword of the description its handle names.
        let number = |car: u32| match crate::menu::cars_store::description(&parts.cars.table, car)
            .map(|d| d.get().car)
        {
            Ok(n) => n,
            Err(e) => {
                failed.set(Some(e));
                u32::MAX
            }
        };
        let checked = carousel::check_cars(read, number, from_second);
        failed.take().map_or(Ok(checked), Err)
    })???;
    Ok(())
}

/// The cars' carousels, from the list; `size` the object's size.
fn cars(
    parts: CarListParts,
    scratch: &crate::scene::workers::DrawingScratch,
    list: CarList,
    size: u32,
) -> Result<(), BadInput> {
    {
        parts.carousel.set_up.set(1);
        parts.carousel.set_list(list);
        let mut n = 0;
        loop {
            let car = list_entry(parts, list, n)?;
            if car == u32::MAX {
                break;
            }
            let first = crate::menu::cars_store::description(&parts.cars.table, car)?
                .get()
                .car;
            // `car_kinds` has 35 entries; a number past them is refused (fixed: "Four tables
            // read past their end" in `wiki/Fixed-bugs.md`).
            let kind = parts
                .lists
                .car_kinds
                .get(first as usize)
                .ok_or(BadInput("a car past the 35 car kinds"))?
                .get();
            parts.carousel.kinds[n].set(kind);
            n += 1;
        }
        parts.carousel.count.set(n as u32);
        let last = (n as u32).wrapping_sub(1);
        parts.cars.table.last_car.set(last);
        scratch.front_end().object.size.set(size);
        for op in CARS_TAIL {
            run_op(parts.carousel, scratch, op);
        }
    }
    Ok(())
}

/// `list` is the list of cars for menus 0xD and 0xE (the other menus use their own).
// Logic: `rc2k_core::scene::carousel::{check_cars, list_end}`
pub fn set_up_carousel(
    parts: CarListParts,
    scratch: &crate::scene::workers::DrawingScratch,
    list: Option<CarList>,
) -> Result<(), Refused> {
    let given = || list.ok_or(BadInput("no list of cars"));
    {
        let menu: u32 = parts.carousel.menu.get();
        if menu >= MENUS {
            return Ok(());
        }
        match menu {
            0xB => {
                check_cars(parts, CarList::First, false)?;
                cars(parts, scratch, CarList::First, 0xF)?;
            }
            0xC => {
                parts.carousel.set_up.set(1);
                parts.carousel.set_list(CarList::A8);
                scratch.front_end().object.size.set(0xCu32);
                write_prefix(&parts.carousel.kinds, &FIXED_KINDS);
                parts.carousel.count.set(6u32);
                parts.cars.table.last_car.set(5u32);
                for op in CARS_TAIL {
                    run_op(parts.carousel, scratch, op);
                }
            }
            0xD => {
                let list = given()?;
                check_cars(parts, list, false)?;
                cars(parts, scratch, list, 0xC)?;
            }
            0xE => {
                let list = given()?;
                check_cars(parts, list, true)?;
                parts.carousel.set_up.set(1);
                let count = reading(parts, list, |read| carousel::list_end(read, 1).unwrap_or(0))?;
                if count == 1 {
                    let first = list_entry(parts, list, 0)?;
                    parts.lists.set_car_entry(parts, list, 1, first)?;
                }
                cars(parts, scratch, list, 0xC)?;
            }
            _ => {
                let ops = PLAIN[menu as usize].ok_or(BadInput("a menu without its list"))?;
                for op in ops {
                    match *op {
                        Op::Set(field, value) => field.cell(parts.carousel).set(value),
                        Op::Run(cell, value) => cell(scratch).set(value),
                        Op::Fill => write_all(&parts.carousel.textures, [u32::MAX; 8]),
                    }
                }
                // The web build's main menu has no network entry in its carousel.
                #[cfg(target_os = "emscripten")]
                if menu == 0 {
                    let kinds = &parts.carousel.kinds;
                    kinds[2].set(kinds[3].get());
                    kinds[3].set(kinds[4].get());
                    parts.carousel.count.set(4u32);
                }
            }
        }
        Ok(())
    }
}
