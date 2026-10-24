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
//!   of the list in esi: the list's address to `dword_75A65C`; for each entry until
//!   -1, the car's number (its first dword) picks a kind from `dword_75A1FE`, which
//!   goes to the next entry's kind; the count to `g_carouselCount`, one less to
//!   `dword_75649C`; then the size (0xF, 0xC), the front, angle and direction 0, the
//!   spacing and place of the cars' carousels, `dword_63503C` = 0x10FF (the angle
//!   about x) and `dword_635020` = 0xFFFE6000 (the height). Out: eax, ebx, edi as the
//!   last entry left them, ecx = the count less one, esi = the list's -1;
//! - 0xE (`sub_4A094F`) as 0xD, but first the list is counted from its second entry
//!   (its first is not looked at) and a list of one car has that car copied over its
//!   -1 (the original's way of showing one car twice: the list is changed, and is then
//!   read on to its next -1);
//! - 0xC (`sub_48FF83`) the six fixed kinds 0x195 .. 0x1BD with `off_759D16` as the
//!   list (esi = it).
//!
//! 26 callers (the screen loops' set-up). Refuses, for the cars' carousels, no list, a list
//! without its -1 within 0x100 entries, an entry that is not there, and a car's number of 0x400
//! or more. Hook removed 2026-10-22 (direct calls): Rust callers run its `entry`
//! (`calls::call`, `routes.rs`); the registers above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::scene::carousel_set_ups::PLAIN;

// ---- game data ----

const MENU_RVA: usize = rva(0x75AA74); // g_carouselMenu
const MENUS: u32 = 0x15;
const TEXTURES_RVA: usize = rva(0x75A622); // dword_75A622, eight numbers
const LIST_RVA: usize = rva(0x75A65C); // dword_75A65C
const KINDS_RVA: usize = rva(0x75A674); // dword_75A674, the entries' kinds
const CAR_KINDS_RVA: usize = rva(0x75A1FE); // dword_75A1FE, a kind by car number
const COUNT_RVA: usize = rva(0x75AA78); // g_carouselCount
const LAST_RVA: usize = rva(0x75649C); // dword_75649C
const SIZE_RVA: usize = rva(0x635038); // dword_635038
const CARS_RVA: usize = rva(0x759DCA); // off_759DCA
const FIXED_CARS_RVA: usize = rva(0x759D16); // off_759D16
use rc2k_core::scene::carousel;

/// One step of a carousel's set-up.
#[derive(Clone, Copy, Debug)]
pub enum Op {
    /// A dword to the address (as IDA shows it).
    Store(usize, u32),
    /// The eight numbers at `dword_75A622` = -1.
    Fill,
}

/// The cars' carousels' stores after their entries, `dword_635038` apart.
const CARS_TAIL: [(usize, u32); 9] = [
    (0x75AA7C, 0),
    (0x75AA80, 0),
    (0x75A660, 0),
    (0x75A664, 0x36BC0),
    (0x75A668, 0xFFFF_4000),
    (0x75A66C, 0x5F000),
    (0x75A670, 0x20000),
    (0x63503C, 0x10FF),
    (0x635020, 0xFFFE_6000),
];

/// `sub_48FF83`'s six kinds.
const FIXED_KINDS: [u32; 6] = [0x195, 0x199, 0x19F, 0x1A5, 0x1AD, 0x1BD];

// ---- Rust version ----

unsafe fn store(address: usize, value: u32) {
    unsafe { game_write(rva(address), value) }
}

unsafe fn list_entry(list: u32, n: usize) -> u32 {
    unsafe { ((list as usize + n * 4) as *const u32).read_unaligned() }
}

/// The checks of a list of cars, before anything is written (see
/// `rc2k_core::scene::carousel::check_cars`).
unsafe fn check_cars(list: u32, from_second: bool) -> Result<(), Refused> {
    unsafe {
        Ok(carousel::check_cars(
            list,
            |n| list_entry(list, n),
            |car| (car as *const u32).read_unaligned(),
            from_second,
        )?)
    }
}

/// The registers the original leaves, where it writes them (the others are as they came).
/// Read through the menus' carried registers: eax and ebx (the last car and its kind), ecx
/// (the number of entries less one, or 0), esi (the end of the list); edi (the last car)
/// is read by none (`lq.py -s 4904DD`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Left {
    pub eax: Option<u32>,
    pub ebx: Option<u32>,
    pub ecx: Option<u32>,
    pub esi: Option<u32>,
    pub edi: Option<u32>,
}

impl Left {
    /// Puts the registers it writes over the given ones (eax, ebx, ecx, esi, edi).
    pub fn over(self, regs: [&mut u32; 5]) {
        let [eax, ebx, ecx, esi, edi] = regs;
        for (to, from) in [
            (eax, self.eax),
            (ebx, self.ebx),
            (ecx, self.ecx),
            (esi, self.esi),
            (edi, self.edi),
        ] {
            if let Some(v) = from {
                *to = v;
            }
        }
    }
}

/// The cars' carousels, from the list; `size` the object's size.
unsafe fn cars(out: &mut Left, list: u32, size: u32) {
    unsafe {
        store(0x75AA8C, 1);
        game_write(LIST_RVA, list);
        let mut n = 0;
        let mut esi = list;
        loop {
            let car = (esi as *const u32).read_unaligned();
            if car == u32::MAX {
                break;
            }
            out.edi = Some(car);
            let first = (car as *const u32).read_unaligned();
            out.eax = Some(first);
            let kind = game_read(CAR_KINDS_RVA + first as usize * 4);
            out.ebx = Some(kind);
            game_write(KINDS_RVA + n * 4, kind);
            n += 1;
            esi = esi.wrapping_add(4);
        }
        out.esi = Some(esi);
        game_write(COUNT_RVA, n as u32);
        let last = (n as u32).wrapping_sub(1);
        out.ecx = Some(last);
        game_write(LAST_RVA, last);
        game_write(SIZE_RVA, size);
        for (address, value) in CARS_TAIL {
            store(address, value);
        }
    }
}

/// The Rust version: `list` is the list of cars the original reads in esi (menus 0xD and
/// 0xE; the other menus' own); returns the registers it leaves.
// Converted: `rc2k_core::scene::carousel::{check_cars, list_end}`
pub unsafe fn set_up_carousel(list: u32) -> Result<Left, Refused> {
    let mut out = Left::default();
    unsafe {
        let menu: u32 = game_read(MENU_RVA);
        if menu >= MENUS {
            return Ok(out);
        }
        match menu {
            0xB => {
                let cars_list = game_addr(CARS_RVA) as u32;
                check_cars(cars_list, false)?;
                cars(&mut out, cars_list, 0xF);
            }
            0xC => {
                store(0x75AA8C, 1);
                let fixed = game_addr(FIXED_CARS_RVA) as u32;
                out.esi = Some(fixed);
                game_write(LIST_RVA, fixed);
                game_write(SIZE_RVA, 0xCu32);
                game_write(KINDS_RVA, FIXED_KINDS);
                game_write(COUNT_RVA, 6u32);
                game_write(LAST_RVA, 5u32);
                for (address, value) in CARS_TAIL {
                    store(address, value);
                }
            }
            0xD => {
                check_cars(list, false)?;
                cars(&mut out, list, 0xC);
            }
            0xE => {
                check_cars(list, true)?;
                store(0x75AA8C, 1);
                let count = carousel::list_end(|n| list_entry(list, n), 1).unwrap_or(0);
                if count == 1 {
                    let first = list_entry(list, 0);
                    out.eax = Some(first);
                    ((list as usize + 4) as *mut u32).write_unaligned(first);
                }
                cars(&mut out, list, 0xC);
            }
            _ => {
                let ops = PLAIN[menu as usize].ok_or(BadInput("a menu without its list"))?;
                for op in ops {
                    match *op {
                        Op::Store(address, value) => store(address, value),
                        Op::Fill => game_write(TEXTURES_RVA, [u32::MAX; 8]),
                    }
                }
                out.eax = Some(u32::MAX);
                out.ecx = Some(0);
                out.edi = Some((game_addr(TEXTURES_RVA) + 0x20) as u32);
            }
        }
        Ok(out)
    }
}
