//! `menu::shuffle_drivers` (0x470AB9): meant to shuffle four driver tables
//! (`unk_73EA7C`, `unk_740764`, `unk_7418BC`, `g_drivers`; 0x94 an entry; counts
//! `dword_75AB04`, `dword_75AB08`, `dword_75AB0C`, `dword_75AB00`), but it never adds the
//! two random entries' offsets: for each table, count times, two random numbers under the
//! count (`dword_7559EC`, `hhmm`) and the first entry swapped with itself through
//! `unk_755398` / `unk_75542C` (both left holding it). Section 16.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_473126` and one of the game's. The random numbers are `math::random_below`'s.
//! Refuses a count of 0 (the original's `div` traps, its loop runs on), before anything.
use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_write, rva};
use crate::menu::standings::*;
use crate::menu::state_store::MENU_STATE;
use crate::session::Dword;

const TABLES: [(usize, &Dword); 4] = [
    (rva(0x73EA7C), &MENU_STATE.menus.class_counts[1]),
    (rva(0x740764), &MENU_STATE.menus.class_counts[2]),
    (rva(0x7418BC), &MENU_STATE.menus.class_counts[3]),
    (DRIVERS_RVA, &MENU_STATE.menus.class_counts[0]),
];
const TEMP_RVA: [usize; 2] = [rva(0x755398), rva(0x75542C)]; // unk_755398, unk_75542C
const PICKS_RVA: [usize; 2] = [rva(0x7559EC), rva(0x7559F0)]; // dword_7559EC, hhmm
/// The Rust version (see the header): the registers it leaves.
pub unsafe fn shuffle_drivers() -> Result<(), Refused> {
    use crate::math::random_below::random_below;
    unsafe {
        if TABLES.iter().any(|&(_, c)| c.get() == 0) {
            return Err(BadInput("a driver table count of 0").into());
        }
        let [t1, t2] = TEMP_RVA.map(|r| game_addr(r) as *mut u8);
        for &(table, count) in &TABLES {
            let first = game_addr(table) as *mut u8;
            let n = count.get();
            for _ in 0..n {
                game_write(PICKS_RVA[0], random_below(n)?);
                game_write(PICKS_RVA[1], random_below(n)?);
                std::ptr::copy(first, t1, 0x94);
                std::ptr::copy(first, t2, 0x94);
                std::ptr::copy(t1, first, 0x94);
                std::ptr::copy(t2, first, 0x94);
            }
        }
    }
    Ok(())
}
