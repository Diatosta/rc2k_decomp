//! The championship's standings table (`sub_4A4E1E` -> `sub_4A256C`; stream A, part F1,
//! 2026-10-18): ten rows from position `dword_7556B8` of class `dword_7558F4` in the
//! championship history (`dword_7556F0`: 0x78-byte entries, -1 ends them; +4 the player's
//! own, +8 the car, +0xC the flag, +0x10 the class, +0x14 the name, +0x54 .. +0x68 six
//! rounds' points, below 0 none, +0x6C the total, +0x70 the position).

use crate::errors::{FatalError, Refused};
use crate::framework::{game_read, rva};
use crate::text::draw_text::Canvas;

fn k(a: usize) -> u32 {
    unsafe { game_read(rva(a)) }
}

unsafe fn get(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

const NOT_FOUND: FatalError = FatalError {
    code: 0x8070_A7FA,
    message: "Position in GB_ChampHST Not Found (see Log)",
};

/// `screens::find_history_entry` (0x4A256C): from entry edi on, the first of class ebx and
/// position eax (the entry edi points at itself is looked at before the end test); none
/// before an entry of -1 is fatal error 0x8070A7FA.
pub unsafe fn find_history_entry(entry: u32, class: u32, position: u32) -> Result<u32, Refused> {
    // Converted: `rc2k_core::screens::championship_table::find_history`.
    use rc2k_core::screens::championship_table::{HISTORY_ENTRY_LEN, find_history};
    let at = |n: u32| entry.wrapping_add(n.wrapping_mul(HISTORY_ENTRY_LEN));
    let found = find_history(
        |n| unsafe { get(at(n)) },
        |n| unsafe { (get(at(n) + 0x10), get(at(n) + 0x70)) },
        class,
        position,
    );
    found.map(at).ok_or(NOT_FOUND.into())
}

/// `screens::draw_championship_table` (0x4A4E1E): the heads (strings 0x484, 0x485, 0x4F9 at
/// y 0xDC), the six rounds' flags (sprites 0x90 .. 0x95 in event mode 0x13, else 0x8A .. 0x8F,
/// at y 0x8C), then the ten rows (`dword_755579` from `dword_7556B8` to `dword_7556BC` =
/// it + 10): position, name, the rounds' points ("-" alone for below 0), the total, the car's
/// and flag's sprites; the player's own (+4 above 0) in the highlight fonts; each text from
/// the last one's y, rows 0x19 apart from y 0xF0. The sprites' palettes ebx as the original
/// leaves it. (It also looks the section's first record up and never uses it: left out.)
/// Every register kept. Refuses (before anything) no history table; a position not in it is
/// the original's fatal error.
pub unsafe fn draw_championship_table(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    unsafe {
        let table = k(0x7556F0);
        if table == 0 {
            return Err(crate::errors::BadInput("no championship history").into());
        }
        // Converted: `rc2k_core::screens::championship_table`; the ten entries looked up
        // first (a position not in the history is the original's fatal error, raised
        // before the rows rather than after those before it).
        let (class, first) = (k(0x7558F4), k(0x7556B8));
        let mut rows = Vec::with_capacity(10);
        for n in 0..10u32 {
            let e = find_history_entry(table, class, first.wrapping_add(n))?;
            rows.push(rc2k_core::screens::championship_table::Row {
                mine: (get(e + 4) as i32) > 0,
                nation: get(e + 8),
                car: get(e + 0xC),
                name: e + 0x14,
                points: std::array::from_fn(|r| get(e + 0x54 + 4 * r as u32) as i32),
                total: get(e + 0x6C),
                position: get(e + 0x70),
            });
        }
        let rows: [_; 10] = rows.try_into().unwrap_or_else(|_| unreachable!());
        rc2k_core::screens::championship_table::championship_table(
            &mut crate::screens::game_screen::GameScreen::<
                rc2k_core::screens::championship_table::Font,
                rc2k_core::screens::championship_table::Work,
            >::new(canvas),
            first,
            k(0x7556D0) == 0x13,
            &rows,
        )
    }
}
