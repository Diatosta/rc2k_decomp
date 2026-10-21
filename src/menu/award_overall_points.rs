//! `menu::award_overall_points` (0x4A4933): the stage's overall championship points. The
//! points start at 0x14 (`seconds`); each entry of standings table B whose +0x10 is not 0,
//! in the table's order, gets them into its +0x38 + 4 * `section` and they go down by one
//! (not below 0); a computer's driver (+4 0, `menu::find_cpu_team` by its +0) also into
//! its +0x5C + 4 * `section`, a player (+4 the player + 1) into its +0xB4 + 4 * `section`.
//! Then each entry's total of its six (+0x38 .., the negative ones left out) into its
//! +0x50 and its driver's +0x74 or player's +0xCC.
//!
//! In:  nothing
//! Out: edi the end entry, ecx 6, edx the last total, eax the last entry's driver number
//!      or player index, esi its driver or player (no `pusha`)
//!
//! Caller: `sub_4A0E60`. The search is its Rust version. Refuses no table, one without its
//! end entry, a section past 5, a player past 4 or missing, before anything; a driver not
//! found is the search's fatal error, before anything. Self-test: the original and the
//! Rust version on the game's tables, groups and sections by lot; put back.
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::award_overall_points",
    rva(0x4A4933),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const POINTS_RVA: usize = rva(0x7559AC); // seconds
const EXTRA: [usize; 1] = [SECTION_RVA];
const CASES: u32 = 4;

fn test_entry(_case: u32) -> Regs {
    Regs::SENTINEL
}

fn prepare(case: u32) {
    unsafe { game_write(SECTION_RVA, [0u32, 2, 5, 3][case as usize]) };
    if let Ok(all) = entries(unsafe { game_read(TABLE_B_RVA) }, B_ENTRY) {
        for (k, &e) in all.iter().enumerate() {
            put(e + 0x10, (k as u32 + case) % 3);
        }
    }
}

/// An entry's driver (with its number) or player (with its index).
unsafe fn owner(entry: &Regs, e: u32) -> Result<(u32, u32, bool), Refused> {
    match get(e + 4) {
        0 => {
            let g = unsafe {
                crate::menu::find_cpu_team::find_cpu_team(&Regs {
                    eax: get(e),
                    edi: e,
                    ..*entry
                })
            }?;
            Ok((g.esi, get(e), true))
        }
        n if n <= 4 && player(n - 1) != 0 => Ok((player(n - 1), n - 1, false)),
        _ => Err(BadInput("a player past 4 or missing").into()),
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn award_overall_points(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        let section: u32 = game_read(SECTION_RVA);
        let table: u32 = game_read(TABLE_B_RVA);
        if table == 0 || section > 5 {
            return Err(BadInput("no table, or a section past 5").into());
        }
        let all = entries(table, B_ENTRY)?;
        let mut owners = Vec::with_capacity(all.len());
        for &e in &all {
            owners.push(owner(entry, e)?);
        }
        // Converted: `rc2k_core::menu::points` (`Points`, `total`).
        use rc2k_core::menu::points::{Points, total};
        let at = 4 * section;
        let mut points = Points::default();
        game_write(POINTS_RVA, points.0);
        for (&e, &(o, _, cpu)) in all.iter().zip(&owners) {
            if get(e + 0x10) == 0 {
                continue;
            }
            let given = points.take();
            game_write(POINTS_RVA, points.0);
            put(e + 0x38 + at, given);
            put(o + if cpu { 0x5C } else { 0xB4 } + at, given);
        }
        for (&e, &(o, number, cpu)) in all.iter().zip(&owners) {
            let total = total(std::array::from_fn(|k| get(e + 0x38 + 4 * k as u32)));
            put(e + 0x50, total);
            put(o + if cpu { 0x74 } else { 0xCC }, total);
            g.eax = number;
            g.esi = o;
            g.edx = total;
        }
        g.ecx = 6;
        g.edi = table + B_ENTRY * all.len() as u32;
    }
    Ok(g)
}

fn regions() -> Vec<Region> {
    let table: u32 = unsafe { game_read(TABLE_B_RVA) };
    let Ok(all) = entries(table, B_ENTRY) else {
        return vec![(0, 0)];
    };
    let first = game_addr(DRIVERS_RVA) as u32;
    let drivers = entries(first, DRIVER).map(|d| d.len()).unwrap_or(0);
    let mut v = vec![
        (table, all.len() * B_ENTRY as usize),
        (first, drivers * DRIVER as usize),
        at(POINTS_RVA),
    ];
    v.extend(
        (0..4u32)
            .map(|n| (player(n), 0x100))
            .filter(|&(p, _)| p != 0),
    );
    v
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { award_overall_points(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions();
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    if regions.iter().any(|&(p, _)| p == 0) {
        return t.skip("the game's tables are not there");
    }
    let mut vars = regions.clone();
    vars.extend(EXTRA.map(at));
    let saved = capture(&vars);
    // The probe runs on the saved tables: it writes what the tests write.
    let refused = unsafe { award_overall_points(&test_entry(0)) }.is_err();
    restore(&vars, &saved);
    if refused {
        return t.skip("the Rust version refuses the game's tables");
    }
    for case in 0..CASES {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            test_entry(case),
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
