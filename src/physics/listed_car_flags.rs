//! Three twins (IDA 2026-10-17): when the current car's entry in `dword_641680`
//! (indexed by `g_currentCar`) is not -1, a flag is set to 1:
//! `physics::set_flag_51659C` (0x45D151, `dword_51659C`), `physics::set_flag_5165A0`
//! (0x45D16D, `dword_5165A0`), `physics::set_flag_5165A4` (0x45D189, `dword_5165A4`).
//!
//! In:  nothing
//! Out: eax the entry; the rest kept; no flags read (callers go on with other
//!      instructions)
//!
//! Callers: `sub_436833`, `sub_436AFB`, `sub_436A4B` (under `sub_436790`).
//! Refuses (before anything) a current car past 7 (the table's size not worked out).
//! Self-test: a copy of each original (the current car, the table and the flag the
//! test's) against the core, cars 0 .. 7 with entries -1 and not.

use crate::errors::Refused;
use crate::framework::{EAX, RawHook, Regs, SelfTest, call_game, game_addr};
use crate::physics::cells::{get, put};
use crate::screens::harness::Left;
use crate::service::harness::capture;
use rc2k_core::physics::listed_cars::listed_entry;

pub const CURRENT_CAR: usize = 0x71B3F8; // g_currentCar
pub const TABLE: usize = 0x641680; // dword_641680

/// The Rust version: the current car at `current`, the table at `table`, the flag at
/// `flag`.
// Converted: `rc2k_core::physics::listed_cars::listed_entry`.
pub unsafe fn set_flag_at(current: u32, table: u32, flag: u32) -> Result<Left, Refused> {
    unsafe {
        let (entry, listed) = listed_entry(get(current), |car| get(table + 4 * car))?;
        if listed {
            put(flag, 1);
        }
        Ok(Left::nothing().with(EAX, entry))
    }
}

/// On the game's variables.
pub unsafe fn set_flag(flag_ida: usize) -> Result<Left, Refused> {
    let a = |x: usize| game_addr(crate::framework::rva(x)) as u32;
    unsafe { set_flag_at(a(CURRENT_CAR), a(TABLE), a(flag_ida)) }
}

/// A copy of one twin (at `address`, its flag `flag_ida`) on test cells.
pub fn twin_copy(
    hook: &RawHook,
    address: usize,
    flag_ida: usize,
    current: u32,
    table: u32,
    flag: u32,
) -> Result<crate::redirect::RedirectedCopy, String> {
    crate::audio::test_copy::make(
        hook,
        &crate::audio::test_copy::Copy {
            address,
            len: 0x1C,
            entry: &[0xA1, 0xF8, 0xB3, 0x71, 0x00],
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[(CURRENT_CAR, current), (TABLE, table), (flag_ida, flag)],
        3,
    )
}

/// The test of one twin at `address` whose flag is `flag_ida`.
pub fn test_twin(hook: &RawHook, address: usize, entry: &[u8], flag_ida: usize) -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(hook) else {
        return (0, 0);
    };
    // The current car, the flag, eight entries.
    let mut cells = vec![0u32; 10];
    let base = cells.as_mut_ptr() as u32;
    let (current, flag, table) = (base, base + 4, base + 8);
    let copy = match crate::audio::test_copy::make(
        hook,
        &crate::audio::test_copy::Copy {
            address,
            len: 0x1C,
            entry,
            calls: &[],
            jumps_out: &[],
        },
        &|_| None,
        &[(CURRENT_CAR, current), (TABLE, table), (flag_ida, flag)],
        3,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let all = [(base, 40)];
    for car in 0..8u32 {
        for minus in [false, true] {
            let fill = || unsafe {
                for k in 0..10u32 {
                    ((base + 4 * k) as *mut u32).write(0xA5A5_0000 | k);
                }
                (current as *mut u32).write(car);
                if minus {
                    ((table + 4 * car) as *mut u32).write(u32::MAX);
                }
            };
            fill();
            let mut ro = Regs::SENTINEL;
            unsafe { call_game(&mut ro, copy.addr()) };
            let theirs = capture(&all);
            fill();
            let left = unsafe { set_flag_at(current, table, flag) };
            let mine = capture(&all);
            let regs_ok = left.is_ok_and(|l| l.matches(&Regs::SENTINEL, &ro));
            t.check(regs_ok && theirs == mine, || {
                format!("car {car}, entry -1 {minus}: regs_ok={regs_ok}; the memory differs")
            });
        }
    }
    unsafe { (current as *mut u32).write(8) };
    let refused = unsafe { set_flag_at(current, table, flag) }.is_err();
    t.check(refused, || "car 8 not refused".into());
    drop((copy, cells));
    t.finish()
}
