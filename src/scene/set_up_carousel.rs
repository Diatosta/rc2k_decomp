//! `scene::set_up_carousel` (0x4904DD): the carousel of the menu `g_carouselMenu`: its
//! entries' kinds, the eight textures of its menu, its size, place and spacing.
//!
//! In:  esi = a list of cars (menus 0xD and 0xE only)
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
//! - 0xB (`sub_48FED9`) the cars of the list `off_759DCA`, 0xD (`sub_4A08AB`) those
//!   of the list in esi: the list's address to `dword_75A65C` (here its `CarList`); for each entry until
//!   -1, the car's number (its first dword) picks a kind from `dword_75A1FE`, which
//!   goes to the next entry's kind; the count to `g_carouselCount`, one less to
//!   `dword_75649C`; then the size (0xF, 0xC), the front, angle and direction 0, the
//!   spacing and place of the cars' carousels, `dword_63503C` = 0x10FF (the angle
//!   about x) and `dword_635020` = 0xFFFE6000 (the height). Out: eax, ebx, edi as the
//!   last entry left them, ecx = the count less one, esi = the list's -1 (here: no list);
//! - 0xE (`sub_4A094F`) as 0xD, but first the list is counted from its second entry
//!   (its first is not looked at) and a list of one car has that car copied over its
//!   -1 (the original's way of showing one car twice: the list is changed, and is then
//!   read on to its next -1);
//! - 0xC (`sub_48FF83`) the six fixed kinds 0x195 .. 0x1BD with `off_759D16` as the
//!   list (esi = it).
//!
//! 26 callers (the screen loops' set-up). Refuses, for the cars' carousels, no list, a list
//! without its -1 within 0x100 entries, an entry that is not there, and a car's number of 0x400
//! or more. The registers above are the original's, for reference; a call by address goes
//! through `routes.rs`.

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::menu::cars_store::CAR_TABLE;
use crate::menu::lists_store::{CarList, MENU_LISTS};
use crate::menu::state_store::MENU_STATE;
use crate::scene::carousel_set_ups::PLAIN;
use crate::scene::object_store::OBJECT_WORK;
use crate::session::{Dword, write_all, write_prefix};

// ---- game data ----
const MENUS: u32 = 0x15;
const CAR_KINDS_IDA: usize = 0x75A1FE; // dword_75A1FE: `MENU_LISTS.car_kinds`
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
    Op::Set(&MENU_STATE.carousel.front, 0),
    Op::Set(&MENU_STATE.carousel.angle, 0),
    Op::Set(&MENU_STATE.carousel.direction, 0),
    Op::Set(&MENU_STATE.carousel.spacing, 0x36BC0),
    Op::Set(&MENU_STATE.carousel.y, 0xFFFF_4000),
    Op::Set(&MENU_STATE.carousel.z, 0x5F000),
    Op::Set(&MENU_STATE.carousel.forward, 0x20000),
    Op::Run(|| &OBJECT_WORK.angles[0], 0x10FF),
    Op::Run(|| &OBJECT_WORK.place[1], 0xFFFE_6000),
];

/// `sub_48FF83`'s six kinds.
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
fn reading<R>(list: CarList, f: impl FnOnce(&dyn Fn(usize) -> u32) -> R) -> Result<R, BadInput> {
    let failed = std::cell::Cell::new(None);
    let read = |n: usize| match list_entry(list, n) {
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

fn list_entry(list: CarList, n: usize) -> Result<u32, BadInput> {
    MENU_LISTS
        .cars(list)
        .offset(4 * n as u32)
        .read("a list of cars")
}

/// The checks of a list of cars, before anything is written (see
/// `rc2k_core::scene::carousel::check_cars`).
fn check_cars(list: CarList, from_second: bool) -> Result<(), Refused> {
    reading(list, |read| {
        let failed = std::cell::Cell::new(None);
        // A car's number: the first dword of the description its handle names.
        let number = |car: u32| match crate::menu::cars_store::description_ptr(car)
            .and_then(|d| d.read::<u32>("a car of the carousel"))
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

/// The registers the original leaves, where it writes them (the others are as they came).
/// Read through the menus' carried registers: eax and ebx (the last car and its kind), ecx
/// (the number of entries less one, or 0), esi (the list of cars after); edi (the last
/// car) is read by none (`lq.py -s 4904DD`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Left {
    pub eax: Option<u32>,
    pub ebx: Option<u32>,
    pub ecx: Option<u32>,
    /// esi where it is written: the fixed kinds' list (menu 0xC), or no list (the cars'
    /// carousels leave the address of their list's end mark, which no one reads as a list).
    pub esi: Option<Option<CarList>>,
    pub edi: Option<u32>,
}

impl Left {
    /// Puts the registers it writes over the given ones (eax, ebx, ecx, edi; esi `cars`).
    pub fn over(self, regs: [&mut u32; 4], cars: &mut Option<CarList>) {
        if let Some(list) = self.esi {
            *cars = list;
        }
        let [eax, ebx, ecx, edi] = regs;
        for (to, from) in [
            (eax, self.eax),
            (ebx, self.ebx),
            (ecx, self.ecx),
            (edi, self.edi),
        ] {
            if let Some(v) = from {
                *to = v;
            }
        }
    }
}

/// The cars' carousels, from the list; `size` the object's size.
fn cars(out: &mut Left, list: CarList, size: u32) -> Result<(), BadInput> {
    {
        MENU_STATE.carousel.set_up.set(1);
        MENU_STATE.carousel.set_list(list);
        let mut n = 0;
        loop {
            let car = list_entry(list, n)?;
            if car == u32::MAX {
                break;
            }
            out.edi = Some(car);
            let first: u32 =
                crate::menu::cars_store::description_ptr(car)?.read("a car of the carousel")?;
            out.eax = Some(first);
            // `car_kinds` has 35 entries; the original allows numbers up to 0x400 and reads
            // on past the table, into what the image has after it (other statics now:
            // `MOVED` resolves each address to its home).
            let kind = crate::store::game_read::<u32>(rva(CAR_KINDS_IDA + 4 * first as usize));
            out.ebx = Some(kind);
            MENU_STATE.carousel.kinds[n].set(kind);
            n += 1;
        }
        out.esi = Some(None);
        MENU_STATE.carousel.count.set(n as u32);
        let last = (n as u32).wrapping_sub(1);
        out.ecx = Some(last);
        CAR_TABLE.last_car.set(last);
        OBJECT_WORK.size.set_bits(size);
        for op in CARS_TAIL {
            run_op(op);
        }
    }
    Ok(())
}

/// The Rust version: `list` is the list of cars the original reads in esi (menus 0xD and
/// 0xE; the other menus' own); returns the registers it leaves.
// Converted: `rc2k_core::scene::carousel::{check_cars, list_end}`
pub fn set_up_carousel(list: Option<CarList>) -> Result<Left, Refused> {
    let given = || list.ok_or(BadInput("no list of cars"));
    let mut out = Left::default();
    {
        let menu: u32 = MENU_STATE.carousel.menu.get();
        if menu >= MENUS {
            return Ok(out);
        }
        match menu {
            0xB => {
                check_cars(CarList::First, false)?;
                cars(&mut out, CarList::First, 0xF)?;
            }
            0xC => {
                MENU_STATE.carousel.set_up.set(1);
                out.esi = Some(Some(CarList::A8));
                MENU_STATE.carousel.set_list(CarList::A8);
                OBJECT_WORK.size.set_bits(0xCu32);
                write_prefix(&MENU_STATE.carousel.kinds, &FIXED_KINDS);
                MENU_STATE.carousel.count.set(6u32);
                CAR_TABLE.last_car.set(5u32);
                for op in CARS_TAIL {
                    run_op(op);
                }
            }
            0xD => {
                let list = given()?;
                check_cars(list, false)?;
                cars(&mut out, list, 0xC)?;
            }
            0xE => {
                let list = given()?;
                check_cars(list, true)?;
                MENU_STATE.carousel.set_up.set(1);
                let count = reading(list, |read| carousel::list_end(read, 1).unwrap_or(0))?;
                if count == 1 {
                    let first = list_entry(list, 0)?;
                    out.eax = Some(first);
                    MENU_LISTS
                        .cars(list)
                        .offset(4)
                        .write(first, "a list of cars")?;
                }
                cars(&mut out, list, 0xC)?;
            }
            _ => {
                let ops = PLAIN[menu as usize].ok_or(BadInput("a menu without its list"))?;
                for op in ops {
                    match *op {
                        Op::Set(cell, value) => cell.set(value),
                        Op::Run(cell, value) => cell().set(value),
                        Op::Fill => write_all(&MENU_LISTS.carousel_textures, [u32::MAX; 8]),
                    }
                }
                out.eax = Some(u32::MAX);
                out.ecx = Some(0);
                out.edi = Some(MENU_LISTS.carousel_textures[0].addr() + 0x20);
            }
        }
        Ok(out)
    }
}
