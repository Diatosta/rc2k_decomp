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
use crate::menu::car_numbers_store::CAR_NUMBERS;
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::state_store::MENU_STATE;
use crate::pointers::GamePtr;
use crate::session::Dword;

/// The four classes' first drivers (`g_drivers` + 90, + 140, + 170, + 0) and counts.
const TABLES: [(usize, &Dword); 4] = [
    (90, &MENU_STATE.menus.class_counts[1]),
    (140, &MENU_STATE.menus.class_counts[2]),
    (170, &MENU_STATE.menus.class_counts[3]),
    (0, &MENU_STATE.menus.class_counts[0]),
];
/// The Rust version (see the header): the registers it leaves.
pub unsafe fn shuffle_drivers() -> Result<(), Refused> {
    use crate::math::random_below::random_below;
    unsafe {
        if TABLES.iter().any(|&(_, c)| c.get() == 0) {
            return Err(BadInput("a driver table count of 0").into());
        }
        let [t1, t2] = [
            GamePtr::new(CAR_NUMBERS.shuffle_a.addr() as u32),
            GamePtr::new(CAR_NUMBERS.shuffle_b.addr() as u32),
        ];
        for &(table, count) in &TABLES {
            let first = GamePtr::new(
                &crate::menu::drivers_store::DRIVERS.drivers[table] as *const _ as usize as u32,
            );
            let n = count.get();
            for _ in 0..n {
                DRAW_WORK.figure.set(random_below(n)?);
                DRAW_WORK.hhmm.set(random_below(n)?);
                first.copy_to(t1, 0x94, "a driver")?;
                first.copy_to(t2, 0x94, "a driver")?;
                t1.copy_to(first, 0x94, "a driver")?;
                t2.copy_to(first, 0x94, "a driver")?;
            }
        }
    }
    Ok(())
}
