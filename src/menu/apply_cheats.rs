//! `menu::apply_cheats` (0x480AC1): acts on the cheats that are switched on.
//!
//! In:  nothing
//! Out: every register preserved
//! The cheats are the bits of `g_cheats` (0x75870C); each that is set writes "Cheat n
//! (...)" to the game's log (`CHEATS` below, in this order):
//! - bits 0, 1, 13 .. 16 stay set and do nothing more here (unlock arcade, unlock cars, no
//!   damage, damage timer, 16, arcade timer): other code looks at them;
//! - bits 2 .. 9, 11 and 12 are cleared and each adds one secret car to the list of cars
//!   at `off_75A042`, unless it is there already (the ten functions `sub_470FE9` ..
//!   `sub_471332`, plain functions here). Bit 6, "Max Power", also adds its car to the
//!   list at `off_759D16`, but only when it was not in the first. The other nine have
//!   the same code for the second list, which is never reached;
//! - bit 10 is cleared and does nothing;
//! - bits 17 and 18 stay set and store 1 in `dword_75ACCC` (the A8 game; the reversed
//!   tracks).
//!
//! Callers: `system::load_game_data` with the bits 0x1FBC (when the setting
//! `allsecretcars` is 1) and `sub_480CBF`, which finds a cheat's name typed as the fourth
//! player's name.
//! Shadow mode: the original first, then the Rust version without the log lines on the
//! variables put back, compared. Self-test: a redirected copy whose log calls note the
//! text, on lists of the test's in the game's places.
//! Refuses a list without its end, or without room for a car more.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva,
};
use crate::install::stand_ins::{set_handler, stand_in};
use crate::menu::remove_secret_cars::{
    CHEATS_RVA, FIRST, List, SECOND, read_list, region, write_list,
};
use crate::menu::text_entry::log_l1;
use crate::redirect::RedirectedCopy;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, dispatch_replayed_state, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::apply_cheats", rva(0x480AC1), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const A8_RVA: usize = rva(0x75ACCC); // dword_75ACCC

/// What a cheat does besides being logged.
#[derive(Clone, Copy)]
enum Does {
    /// The bit stays set.
    Stays,
    /// The bit is cleared.
    Cleared,
    /// The bit is cleared and the car at this address added to the first list; `true`:
    /// and then to the second.
    Car(usize, bool),
    /// The bit stays set and `dword_75ACCC` becomes 1.
    Stores,
}

/// By bit: the log line's text and what the cheat does.
const CHEATS: [(usize, Does); 19] = [
    (0x4DE7A9, Does::Stays),                // "Cheat 1 (Unlock Arcade)"
    (0x4DE7C1, Does::Stays),                // "Cheat 2 (Unlock Cars)"
    (0x4DE7D7, Does::Car(0x757210, false)), // "Cheat 3 (Spud)"
    (0x4DE7E6, Does::Car(0x757300, false)), // "Cheat 4 (Sheep)"
    (0x4DE7F6, Does::Car(0x757288, false)), // "Cheat 5 (Cow)"
    (0x4DE804, Does::Car(0x757030, false)), // "Cheat 6 (MF Sports)"
    (0x4DE818, Does::Car(0x756838, true)),  // "Cheat 7 (Max Power)"
    (0x4DE82C, Does::Car(0x757468, false)), // "Cheat 8 (Local Shop)"
    (0x4DE841, Does::Car(0x757120, false)), // "Cheat 9 (Bolt's Pug)"
    (0x4DE856, Does::Car(0x757198, false)), // "Cheat 10 (Skip)"
    (0x4DE866, Does::Cleared),              // "Cheat 11"
    (0x4DE86F, Does::Car(0x7570A8, false)), // "Cheat 12 (MF Hatchback)"
    (0x4DE887, Does::Car(0x7573F0, false)), // "Cheat 13 (Sheep Wagon)"
    (0x4DE89E, Does::Stays),                // "Cheat 14 (No Damage)"
    (0x4DE8B3, Does::Stays),                // "Cheat 15 (Damage Timer)"
    (0x4DE8CB, Does::Stays),                // "Cheat 16"
    (0x4DE8D4, Does::Stays),                // "Cheat 17 (Arcade Timer)"
    (0x4DE8EC, Does::Stores),               // "Cheat 18 (Enable A8 Game)"
    (0x4DE906, Does::Stores),               // "Cheat 19 (Reverse Sim Tracks)"
];

// ---- Rust version ----

/// Adds the car to the list unless it is there; true if it was added.
fn add_car(list: &mut Vec<u32>, room: List, car: u32) -> Result<bool, BadInput> {
    if list.contains(&car) {
        return Ok(false);
    }
    // The car and the -1 after it.
    if list.len() + 2 > room.1 {
        return Err(BadInput("a list of cars without room for one more"));
    }
    list.push(car);
    Ok(true)
}

/// `log` gets the address of each line's text.
pub unsafe fn apply_cheats(log: &mut dyn FnMut(*const u8)) -> Result<(), BadInput> {
    unsafe {
        let mut cheats: u32 = game_read(CHEATS_RVA);
        let mut first = read_list(FIRST)?;
        let mut second = read_list(SECOND)?;
        let (mut first_changed, mut second_changed, mut stores) = (false, false, false);
        let mut lines = Vec::new();
        for (bit, (text, does)) in CHEATS.into_iter().enumerate() {
            if cheats & (1 << bit) == 0 {
                continue;
            }
            lines.push(game_addr(rva(text)) as *const u8);
            match does {
                Does::Stays => {}
                Does::Cleared => cheats &= !(1 << bit),
                Does::Car(car, both) => {
                    cheats &= !(1 << bit);
                    let car = game_addr(rva(car)) as u32;
                    if add_car(&mut first, FIRST, car)? {
                        first_changed = true;
                        if both && add_car(&mut second, SECOND, car)? {
                            second_changed = true;
                        }
                    }
                }
                Does::Stores => stores = true,
            }
        }
        // Nothing has been written or logged up to here.
        game_write(CHEATS_RVA, cheats);
        if first_changed {
            write_list(FIRST, &first);
        }
        if second_changed {
            write_list(SECOND, &second);
        }
        if stores {
            game_write(A8_RVA, 1u32);
        }
        for line in lines {
            log(line);
        }
        Ok(())
    }
}

// ---- hook ----

fn regions() -> [Region; 4] {
    [region(FIRST), region(SECOND), at(CHEATS_RVA), at(A8_RVA)]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { apply_cheats(&mut |line| log_l1(line, true)) }?;
    Ok(Left::nothing())
}

fn quiet(_: &Regs) -> Result<Left, Refused> {
    unsafe { apply_cheats(&mut |_| {}) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_replayed_state(&HOOK, r, call_original, &regions(), &rust, &quiet);
}

// ---- self-test ----

const LEN: usize = 0x1FE;
const ENTRY_BYTES: [u8; 8] = [0x60, 0x0F, 0xBA, 0x25, 0x0C, 0x87, 0x75, 0x00];
/// The nineteen log calls and the ten calls of the cars' functions.
const CALLS: [usize; 29] = [
    0x12, 0x29, 0x40, 0x46, 0x5C, 0x62, 0x78, 0x7E, 0x94, 0x9A, 0xB0, 0xB6, 0xCC, 0xD2, 0xE8, 0xEE,
    0x104, 0x10A, 0x120, 0x137, 0x13D, 0x153, 0x159, 0x16F, 0x186, 0x19D, 0x1B4, 0x1CB, 0x1EC,
];
const LOG_RVA: usize = rva(0x40142B); // logfile::log_write_l1_nl

/// The texts a run logged.
static LOGGED: Mutex<Vec<u32>> = Mutex::new(Vec::new());

fn note_line(_: u32, r: &mut Regs) {
    LOGGED.lock().unwrap().push(r.eax);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x480AC1)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| (callee == LOG_RVA).then(|| stand_in(0)),
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let regions = regions();
    let saved = capture(&regions);
    let car = |n: usize| match CHEATS[n].1 {
        Does::Car(car, _) => game_addr(rva(car)) as u32,
        _ => 0,
    };
    let (a, b) = (0x1111_0000u32, 0x2222_0000u32);
    // (what, the first list, the second)
    let lists: [(&str, Vec<u32>, Vec<u32>); 5] = [
        ("plain lists", vec![a, b], vec![b]),
        ("empty lists", vec![], vec![]),
        (
            "three of the cars there",
            vec![car(2), a, car(6), car(12)],
            vec![a],
        ),
        ("Max Power in the second only", vec![a], vec![b, car(6)]),
        (
            "lists that all the cars fill",
            (0..0x1D).map(|n| a + n).collect(),
            (0..0x2B).map(|n| b + n).collect(),
        ),
    ];
    let mut flags: Vec<u32> = (0..19).map(|bit| 1 << bit).collect();
    flags.extend([
        0,
        0x1FBC,
        0x7_FFFF,
        u32::MAX,
        0xFFF8_0000,
        0x5_5555,
        0x2_AAAA,
    ]);
    for (what, first, second) in &lists {
        for &cheats in &flags {
            let before = Regs::SENTINEL;
            let run = |rust: bool| unsafe {
                restore(&regions, &vec![0xAA; saved.len()]);
                write_list(FIRST, first);
                write_list(SECOND, second);
                game_write(CHEATS_RVA, cheats);
                LOGGED.lock().unwrap().clear();
                set_handler(note_line);
                let ended = if rust {
                    match apply_cheats(&mut |line| LOGGED.lock().unwrap().push(line as u32)) {
                        Ok(()) => 0,
                        Err(_) => 2,
                    }
                } else {
                    let mut ro = before;
                    call_game(&mut ro, copy.addr());
                    !ro.preserved_except(&before, 0) as u32
                };
                (ended, capture(&regions), LOGGED.lock().unwrap().clone())
            };
            let (theirs, mine) = (run(false), run(true));
            t.check(theirs == mine && theirs.0 == 0, || {
                format!(
                    "{what}, cheats {cheats:#x}: original ended {} logged {:x?}; rust ended {} logged {:x?}; first different byte left {:?}",
                    theirs.0,
                    theirs.2,
                    mine.0,
                    mine.2,
                    theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b),
                )
            });
        }
    }
    restore(&regions, &saved);
    t.finish()
}
