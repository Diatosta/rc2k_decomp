//! `menu::award_stage_points` (0x4A47EA): the stage's championship points. For each class
//! (`dword_7558F4`, 0 .. `dword_7558F8`) the points start at 0x14 (`seconds`); each entry
//! of standings table B in that class, in the table's order, is logged ("Status of
//! driver" and its +0x74), gets the points into its +0x54 + 4 * `section` and the points
//! go down by one (not below 0). A computer's driver (+4 0, `menu::find_cpu_team` by its
//! +0) keeps them in its +0x78 + 4 * `section` when its +0x24 is 1, a player (+4 the
//! player + 1) in its +0x98 + 4 * `section` when its +0xD0 is 1; otherwise both are 0.
//! Then each entry's total of its six stage points (+0x54 .., the negative ones left out)
//! into its +0x6C and its driver's +0x90 or player's +0xB0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers. The search is its Rust version; it logs through
//! `logfile::log_write` and `log_int` (level 1). Refuses no table, one without its end
//! entry, a class count of 0 or past 0x100, a section past 5, a player past 4 or missing,
//! before anything; a driver not found is the search's fatal error, before anything.
//! Shadow mode runs the original alone (it logs). Self-test (unpaused: it logs): the original and the Rust version on the game's tables,
//! groups and sections by lot; put back.
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::award_stage_points", rva(0x4A47EA), thunk, &ORIGINAL)
        .self_test(self_test)
        .self_test_unpaused();

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const POINTS_RVA: usize = rva(0x7559AC); // seconds
const CLASS_RVA: usize = rva(0x7558F4); // dword_7558F4
const CLASSES_RVA: usize = rva(0x7558F8); // dword_7558F8
const STATUS_TEXT_RVA: usize = rva(0x4DEC56); // aStatusOfDriver
const EXTRA: [usize; 2] = [CLASSES_RVA, SECTION_RVA];
const CASES: u32 = 4;

fn test_entry(_case: u32) -> Regs {
    Regs::SENTINEL
}

fn prepare(case: u32) {
    unsafe {
        game_write(CLASSES_RVA, 1 + case % 3);
        game_write(SECTION_RVA, [0u32, 2, 5, 3][case as usize]);
    }
    if let Ok(all) = entries(unsafe { game_read(TABLE_B_RVA) }, B_ENTRY) {
        for (k, &e) in all.iter().enumerate() {
            put(e + 0x10, (k as u32 + case) % 3);
        }
    }
}

/// The driver (computer, by `find_cpu_team`) or player of an entry, and the field its
/// stage points go to, and whether it keeps them.
enum Owner {
    Driver(u32),
    Player(u32),
}

unsafe fn owner(entry: &Regs, e: u32) -> Result<Owner, Refused> {
    match get(e + 4) {
        0 => {
            let g = unsafe {
                crate::menu::find_cpu_team::find_cpu_team(&Regs {
                    eax: get(e),
                    edi: e,
                    ..*entry
                })
            }?;
            Ok(Owner::Driver(g.esi))
        }
        n if n <= 4 && player(n - 1) != 0 => Ok(Owner::Player(player(n - 1))),
        _ => Err(BadInput("a player past 4 or missing").into()),
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn award_stage_points(entry: &Regs) -> Result<Regs, Refused> {
    unsafe {
        let classes: u32 = game_read(CLASSES_RVA);
        let section: u32 = game_read(SECTION_RVA);
        let table: u32 = game_read(TABLE_B_RVA);
        if table == 0 || classes == 0 || classes > 0x100 || section > 5 {
            return Err(BadInput(
                "no table, a class count of 0 or past 0x100, or a section past 5",
            )
            .into());
        }
        let all = entries(table, B_ENTRY)?;
        let mut owners = Vec::with_capacity(all.len());
        for &e in &all {
            owners.push(owner(entry, e)?);
        }
        let at = 4 * section;
        for class in 0..classes {
            game_write(POINTS_RVA, 0x14u32);
            for (&e, o) in all.iter().zip(&owners) {
                game_write(CLASS_RVA, class);
                if get(e + 0x10) != class {
                    continue;
                }
                crate::logfile::log_write::log_write(
                    game_addr(STATUS_TEXT_RVA) as *const u8,
                    false,
                    1,
                );
                crate::logfile::log_int::log_int(get(e + 0x74) as i32, true, 1);
                let points: u32 = game_read(POINTS_RVA);
                put(e + 0x54 + at, points);
                if (points as i32) > 0 {
                    game_write(POINTS_RVA, points - 1);
                }
                let (keeps, field) = match *o {
                    Owner::Driver(d) => (get(d + 0x24) == 1, d + 0x78 + at),
                    Owner::Player(p) => (get(p + 0xD0) == 1, p + 0x98 + at),
                };
                if keeps {
                    put(field, get(e + 0x54 + at));
                } else {
                    put(e + 0x54 + at, 0);
                    put(field, 0);
                }
            }
        }
        for (&e, o) in all.iter().zip(&owners) {
            let total = (0..6)
                .map(|k| get(e + 0x54 + 4 * k) as i32)
                .filter(|&v| v >= 0)
                .fold(0u32, |s, v| s.wrapping_add(v as u32));
            put(e + 0x6C, total);
            match *o {
                Owner::Driver(d) => put(d + 0x90, total),
                Owner::Player(p) => put(p + 0xB0, total),
            }
        }
    }
    Ok(*entry)
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
        at(CLASS_RVA),
    ];
    v.extend(
        (0..4u32)
            .map(|n| (player(n), 0x100))
            .filter(|&(p, _)| p != 0),
    );
    v
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { award_stage_points(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    // It logs: in shadow mode the original alone (the self-test compares the two).
    if crate::framework::shadow() {
        HOOK.count_call();
        unsafe { call_original(r) };
        return;
    }
    let regions = regions();
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // Unpaused: frames drawn meanwhile may see the tables this test changes.
    let _disturbing = crate::screens::harness::Disturbing::begin();
    let regions = regions();
    if regions.iter().any(|&(p, _)| p == 0) {
        return t.skip("the game's tables are not there");
    }
    let mut vars = regions.clone();
    vars.extend(EXTRA.map(at));
    let saved = capture(&vars);
    // The probe runs on the saved tables: it writes what the tests write.
    let refused = unsafe { award_stage_points(&test_entry(0)) }.is_err();
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
