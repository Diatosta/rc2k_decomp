//! `race::read_replay_header` (0x460428): "READREPLAYHEADER" logged; the replay's header
//! (`dword_719B14` ..) into the race's variables: the arcade's noise seed, start place
//! and cars, the stage, the game mode, the players, the cars and their kinds (into the
//! car blocks too), the two game-over flags (`MOVES`); then twelve of them logged, each
//! as its name and its value (`NAMES`).
//!
//! In:  nothing
//! Out: every register kept (the logs in `pusha` / `popa`, the copies by `push` / `pop`)
//!
//! Callers: `race::viewer_load_stage`, 0x44401B (the replay run of `sub_443FxF`). Its log
//! calls through `part`: 1 `logfile::log_write_l1_nl`, 2 `logfile::log_write_l1`, 3
//! `logfile::log_int_l1_nl` (eax the text or value); in the game their Rust versions.
//! Shadow mode runs the original alone (it logs). Self-test: a copy whose calls go to stand-ins, against the Rust version; the header filled
//! by the test, what it writes saved and put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::flow::a;
use crate::race::viewer::{Part, Saved, dispatch_parts, dword, get, put, run_both, test_part};
use crate::service::harness::Region;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::read_replay_header", rva(0x460428), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// (to, from), in the original's order.
const MOVES: [(usize, usize); 17] = [
    (0x60519C, 0x719B48),
    (0x6051BC, 0x719B40),
    (0x6051B4, 0x719B44),
    (0x623850, 0x719B14), // g_stageNumber
    (0x62389C, 0x719B18),
    (0x63DB94, 0x719B1C),
    (0x63DB90, 0x719B20), // g_raceCars
    (0x70F410, 0x719B24), // g_carBlockStore.kind
    (0x623858, 0x719B24), // g_raceCarKinds
    (0x7103F0, 0x719B28),
    (0x62385C, 0x719B28),
    (0x7113D0, 0x719B2C),
    (0x623860, 0x719B2C),
    (0x7123B0, 0x719B30),
    (0x623864, 0x719B30),
    (0x70F49C, 0x719B4C), // g_carBlockStore.out
    (0x71047C, 0x719B50),
];
/// (the name's text, the value logged).
const NAMES: [(usize, usize); 12] = [
    (0x4DDDFB, 0x719B40), // "[replayheader.arcstartpos]"
    (0x4DDE16, 0x719B44), // "[replayheader.numarccars]"
    (0x4DDE30, 0x719B14), // "[replayheader.track]"
    (0x4DDE45, 0x719B18), // "[replayheader.gmode]"
    (0x4DDE5A, 0x719B1C), // "[replayheader.numpl]"
    (0x4DDE6F, 0x719B20), // "[replayheader.numcars]"
    (0x4DDE86, 0x719B24), // "[replayheader.car1]"
    (0x4DDE9A, 0x719B28), // "[replayheader.car2]"
    (0x4DDEAE, 0x719B2C), // "[replayheader.car3]"
    (0x4DDEC2, 0x719B30), // "[replayheader.car4]"
    (0x4DDED6, 0x719B4C), // "[replayheader.gameoverflag1]"
    (0x4DDEF3, 0x719B50), // "[replayheader.gameoverflag2]"
];
const TITLE: usize = 0x4DDDEA; // "READREPLAYHEADER"

/// The Rust version: the registers it leaves (the entry's).
pub fn read_replay_header(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let call = |part: Part, place: u32, eax: u32| -> Result<(), Refused> {
        let mut r = Regs { eax, ..*entry };
        part(place, &mut r)
    };
    call(part, 1, a(TITLE))?;
    for (to, from) in MOVES {
        put(to, get(from));
    }
    for (text, value) in NAMES {
        call(part, 2, a(text))?;
        call(part, 3, get(value))?;
    }
    Ok(*entry)
}

/// In the game: the logs' Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        match place {
            1 | 2 => crate::logfile::log_write::log_write(r.eax as *const u8, place == 1, 1),
            _ => crate::logfile::log_int::log_int(r.eax as i32, true, 1),
        }
    }
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        read_replay_header(r, &mut game_part)
    });
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let calls: Vec<usize> = std::iter::once(0x7)
        .chain((0..12).flat_map(|n| [0xE0 + 0x19 * n, 0xEC + 0x19 * n]))
        .collect();
    // pusha; lea eax, aReadreplayhead
    let copy = match crate::race::viewer::copy(
        0x460428,
        0x206,
        &[0x60, 0x8D, 0x05, 0xEA, 0xDD, 0x4D, 0x00],
        &calls,
        &[(0x40142B, 1), (0x401427, 2), (0x401476, 3)],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    crate::race::viewer::quiet();
    let regions: Vec<Region> = MOVES.iter().map(|&(to, _)| dword(to)).collect();
    let mut all = regions.clone();
    all.push(crate::race::viewer::region(0x719B14, 0x40));
    let saved = Saved::new(&all);
    run_both(
        &mut t,
        "a header of the test's",
        copy.addr(),
        Regs::SENTINEL,
        &regions,
        0,
        &|| {
            crate::race::viewer::fill(0x719B14, 0x40, 21);
            for &(to, _) in &MOVES {
                put(to, 0xAAAA_AAAA);
            }
        },
        &|r| read_replay_header(r, &mut test_part),
    );
    drop(saved);
    t.finish()
}
