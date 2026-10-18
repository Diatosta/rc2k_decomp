//! `menu::keep_record_best` (0x4A3E8B): the current record (`g_recordTable`, `section`,
//! `index`) takes each counted player's (`dword_755984`) score (+0x94) while it beats the
//! record's (+0x30, signed), with the player's +4 / +8 (+0x74 / +0x78) and name (0x20
//! bytes to +0x54); it stops at the first that does not. `dword_75598C` the player looked
//! at last.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers. `records::find_record`'s Rust version. Refuses a record
//! past the table, a player count of 0 or past 4, a missing player or name, before
//! anything. Self-test: the original and the Rust version on the game's tables, scores by
//! lot; put back.
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::keep_record_best", rva(0x4A3E8B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CURRENT_RVA: usize = rva(0x75598C); // dword_75598C

/// The current record (`g_recordTable`, `section`, `index`), refused past the table.
fn current_record() -> Result<u32, BadInput> {
    let table = game_addr(RECORDS_RVA) as u32;
    let section: u32 = unsafe { game_read(SECTION_RVA) };
    let index: u32 = unsafe { game_read(INDEX_RVA) };
    if section >= 64
        || index >= 64
        || !crate::records::find_record::has_section(table, section, 600)
    {
        return Err(BadInput("a record past the table"));
    }
    Ok(crate::records::find_record::find_record_in(
        table,
        index as i32,
        section,
    ))
}
const EXTRA: [usize; 0] = [];
const CASES: u32 = 3;

fn prepare(case: u32) {
    let _ = case;
    for n in 0..4u32 {
        if player(n) != 0 {
            put(
                player(n) + 0x94,
                [500u32, 0, 0xFFFF_FFF0][case as usize] + 100 * n,
            );
        }
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn keep_record_best(entry: &Regs) -> Result<Regs, Refused> {
    let players = counted_players()?;
    if players.iter().any(|&p| get(p + 0x18) == 0) {
        return Err(BadInput("a player without a name").into());
    }
    let record = current_record()?;
    for (n, &p) in players.iter().enumerate() {
        unsafe { game_write(CURRENT_RVA, n as u32) };
        if get(record + 0x30) as i32 >= get(p + 0x94) as i32 {
            break;
        }
        put(record + 0x30, get(p + 0x94));
        put(record + 0x74, get(p + 4));
        put(record + 0x78, get(p + 8));
        unsafe { std::ptr::copy(get(p + 0x18) as *const u8, (record + 0x54) as *mut u8, 0x20) };
    }
    Ok(*entry)
}

fn regions(r: &Regs) -> Vec<Region> {
    let _ = r;
    match current_record() {
        Ok(rec) => vec![(rec, 0x80), at(CURRENT_RVA)],
        Err(_) => vec![(0, 0)],
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { keep_record_best(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions(r);
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let before = Regs::SENTINEL;
    let regions = regions(&before);
    if regions.iter().any(|&(p, _)| p == 0) {
        return t.skip("the game's tables are not there");
    }
    let mut vars = regions.clone();
    vars.extend(EXTRA.map(at));
    let saved = capture(&vars);
    // The probe runs on the saved tables: it writes what the tests write.
    let refused = unsafe { keep_record_best(&before) }.is_err();
    restore(&vars, &saved);
    if refused {
        return t.skip("the Rust version refuses the game's tables");
    }
    for case in 0..CASES {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            before,
            &|| {
                restore(&vars, &saved);
                prepare(case);
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
