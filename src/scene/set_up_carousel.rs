//! `scene::set_up_carousel` (0x4904DD): the carousel of the menu `g_carouselMenu`: its
//! entries' kinds, the eight textures of its menu, its size, place and spacing.
//!
//! Takes: a list of cars (menus 0xD and 0xE only)
//! Out: by the menu (below); everything else preserved; for a menu outside 0 .. 0x14
//!      nothing at all
//!
//! The dispatcher calls one function a menu, 21 of them, each with this one caller and
//! all inlined here. Seventeen are only stores (`scene/carousel_set_ups.rs`, generated
//! by `tools/carousel_set_ups` from the listing): `dword_75AA8C` = 1, the eight
//! numbers at `dword_75A622` = -1 (`rep stosd`: eax = -1, ecx = 0, edi = their end)
//! and then that menu's numbers, the object's size `dword_635038`, the entries' kinds
//! from `dword_75A674` on, `g_carouselCount`, the front, angle and direction 0, and
//! the spacing and place (`dword_75A664` .. `dword_75A670`).
//! - 0xB (0x48FED9) the cars of the list `off_759DCA`, 0xD (0x4A08AB) those
//!   of the list in esi: the list's address to `dword_75A65C` (here its `CarList`); for each entry until
//!   -1, the car's number (its first dword) picks a kind from `dword_75A1FE`, which
//!   goes to the next entry's kind; the count to `g_carouselCount`, one less to
//!   `dword_75649C`; then the size (0xF, 0xC), the front, angle and direction 0, the
//!   spacing and place of the cars' carousels, `dword_63503C` = 0x10FF (the angle
//!   about x) and `dword_635020` = 0xFFFE6000 (the height). Out: eax, ebx, edi as the
//!   last entry left them, ecx = the count less one, esi = the list's -1 (here: no list);
//! - 0xE (0x4A094F) as 0xD, but first the list is counted from its second entry
//!   (its first is not looked at) and a list of one car has that car copied over its
//!   -1 (the original's way of showing one car twice: the list is changed, and is then
//!   read on to its next -1);
//! - 0xC (0x48FF83) the six fixed kinds 0x195 .. 0x1BD with `off_759D16` as the
//!   list (esi = it).
//!
//! 26 callers (the screen loops' set-up). Refuses, for the cars' carousels, no list, a list
//! without its -1 within 0x100 entries, an entry that is not there, and a car's number of 0x400
//! or more. The registers above are the original's, for reference; a call by address goes
//! through `routes.rs`.

use crate::errors::{BadInput, Refused};
use crate::menu::lists_store::CarList;
use crate::scene::carousel_set_ups::PLAIN;
use crate::session::{Dword, write_all, write_prefix};

// ---- game data ----
const MENUS: u32 = 0x15;
use crate::menu::owner::{Menus, Tables, menus};
use crate::scene::owner::scene;
use rc2k_core::scene::carousel;

/// One step of a carousel's set-up.
#[derive(Clone, Copy)]
pub enum Op {
    /// A dword to one of the physics block's variables.
    Set(&'static Dword, u32),
    /// A dword of the race block's, found at run time (those statics do not deref in a `const`).
    Run(fn() -> &'static Dword, u32),
    /// The eight numbers at `dword_75A622` = -1.
    Fill,
}

/// The cars' carousels' stores after their entries, `dword_635038` apart.
const CARS_TAIL: [Op; 9] = [
    Op::Set(&crate::menu::owner::MENUS.screens.state.carousel.front, 0),
    Op::Set(&crate::menu::owner::MENUS.screens.state.carousel.angle, 0),
    Op::Set(
        &crate::menu::owner::MENUS.screens.state.carousel.direction,
        0,
    ),
    Op::Set(
        &crate::menu::owner::MENUS.screens.state.carousel.spacing,
        0x36BC0,
    ),
    Op::Set(
        &crate::menu::owner::MENUS.screens.state.carousel.y,
        0xFFFF_4000,
    ),
    Op::Set(&crate::menu::owner::MENUS.screens.state.carousel.z, 0x5F000),
    Op::Set(
        &crate::menu::owner::MENUS.screens.state.carousel.forward,
        0x20000,
    ),
    Op::Run(|| &scene().drawing.object_work.angles[0], 0x10FF),
    Op::Run(|| &scene().drawing.object_work.place[1], 0xFFFE_6000),
];

/// 0x48FF83's six kinds.
const FIXED_KINDS: [u32; 6] = [0x195, 0x199, 0x19F, 0x1A5, 0x1AD, 0x1BD];

// ---- Rust version ----

/// A store of the tables (never a `Fill`).
fn run_op(op: Op) {
    match op {
        Op::Set(cell, value) => cell.set(value),
        Op::Run(cell, value) => cell().set(value),
        Op::Fill => unreachable!("the cars' tail has no fill"),
    }
}

/// Runs `f` with a reader of the list's entries (by number) that answers the list's end
/// where an entry is past the lists, and refuses afterwards: the core's checks take
/// readers that cannot fail.
fn reading<R>(
    tables: &Tables,
    list: CarList,
    f: impl FnOnce(&dyn Fn(usize) -> u32) -> R,
) -> Result<R, BadInput> {
    let failed = std::cell::Cell::new(None);
    let read = |n: usize| match list_entry(tables, list, n) {
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

fn list_entry(tables: &Tables, list: CarList, n: usize) -> Result<u32, BadInput> {
    tables
        .lists
        .cars(list)
        .offset(4 * n as u32)
        .read("a list of cars")
}

/// The checks of a list of cars, before anything is written (see
/// `rc2k_core::scene::carousel::check_cars`).
fn check_cars(tables: &Tables, list: CarList, from_second: bool) -> Result<(), Refused> {
    reading(tables, list, |read| {
        let failed = std::cell::Cell::new(None);
        // A car's number: the first dword of the description its handle names.
        let number = |car: u32| match crate::menu::cars_store::description(car).map(|d| d.get().car)
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
fn cars(menus: &Menus, list: CarList, size: u32) -> Result<(), BadInput> {
    {
        menus.screens.state.carousel.set_up.set(1);
        menus.screens.state.carousel.set_list(list);
        let mut n = 0;
        loop {
            let car = list_entry(&menus.tables, list, n)?;
            if car == u32::MAX {
                break;
            }
            let first = crate::menu::cars_store::description(car)?.get().car;
            // `car_kinds` (dword_75A1FE) has 35 entries; the original allows numbers up to
            // 0x400 and reads on past the table (fixed: refused).
            let kind = menus
                .tables
                .lists
                .car_kinds
                .get(first as usize)
                .ok_or(BadInput("a car past the 35 car kinds"))?
                .get();
            menus.screens.state.carousel.kinds[n].set(kind);
            n += 1;
        }
        menus.screens.state.carousel.count.set(n as u32);
        let last = (n as u32).wrapping_sub(1);
        menus.cars.table.last_car.set(last);
        scene().drawing.object_work.size.set_bits(size);
        for op in CARS_TAIL {
            run_op(op);
        }
    }
    Ok(())
}

/// The Rust version: `list` is the list of cars the original reads in esi (menus 0xD and
/// 0xE; the other menus' own). What the original leaves in the registers (above) is read by
/// none of its callers.
// Converted: `rc2k_core::scene::carousel::{check_cars, list_end}`
pub fn set_up_carousel(
    list: Option<CarList>,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let menus = menus();
    let given = || list.ok_or(BadInput("no list of cars"));
    {
        let menu: u32 = menus.screens.state.carousel.menu.get();
        if menu >= MENUS {
            return Ok(());
        }
        match menu {
            0xB => {
                check_cars(&menus.tables, CarList::First, false)?;
                cars(menus, CarList::First, 0xF)?;
            }
            0xC => {
                menus.screens.state.carousel.set_up.set(1);
                menus.screens.state.carousel.set_list(CarList::A8);
                scene.drawing.object_work.size.set_bits(0xCu32);
                write_prefix(&menus.screens.state.carousel.kinds, &FIXED_KINDS);
                menus.screens.state.carousel.count.set(6u32);
                menus.cars.table.last_car.set(5u32);
                for op in CARS_TAIL {
                    run_op(op);
                }
            }
            0xD => {
                let list = given()?;
                check_cars(&menus.tables, list, false)?;
                cars(menus, list, 0xC)?;
            }
            0xE => {
                let list = given()?;
                check_cars(&menus.tables, list, true)?;
                menus.screens.state.carousel.set_up.set(1);
                let count = reading(&menus.tables, list, |read| {
                    carousel::list_end(read, 1).unwrap_or(0)
                })?;
                if count == 1 {
                    let first = list_entry(&menus.tables, list, 0)?;
                    menus
                        .tables
                        .lists
                        .cars(list)
                        .offset(4)
                        .write(first, "a list of cars")?;
                }
                cars(menus, list, 0xC)?;
            }
            _ => {
                let ops = PLAIN[menu as usize].ok_or(BadInput("a menu without its list"))?;
                for op in ops {
                    match *op {
                        Op::Set(cell, value) => cell.set(value),
                        Op::Run(cell, value) => cell().set(value),
                        Op::Fill => write_all(&menus.tables.lists.carousel_textures, [u32::MAX; 8]),
                    }
                }
            }
        }
        Ok(())
    }
}
