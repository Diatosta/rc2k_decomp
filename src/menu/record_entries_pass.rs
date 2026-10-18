//! `menu::record_entries_pass` (0x4A3BE0): the current event's record entries cleared
//! (`menu::clear_record_entries`), then each counted player (`dword_75598C` 0 ..
//! `dword_755984`, the original's `do .. while`) that beat the target time
//! (`menu::beat_target_time`, `dword_755918`) entered (`menu::enter_record_entry`),
//! `dword_755920` 1 once one is (0 first).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers. The steps are their Rust versions. A private copy (a call at
//! +1). Refuses what the steps refuse and a player count of 0 or past 4 or a player
//! missing, before anything. Self-test: the original and the Rust version on the game's
//! tables, times by lot; put back.
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::record_entries_pass", rva(0x4A3BE0), thunk, &ORIGINAL)
        .private_copy(0x43)
        .relocate_calls(&CALLS)
        .manual_patch()
        .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CURRENT_RVA: usize = rva(0x75598C); // dword_75598C
const SLOT_RVA: usize = rva(0x7559EC); // dword_7559EC
const HHMM_RVA: usize = rva(0x7559F0); // hhmm
const ENTERED_RVA: usize = rva(0x755920); // dword_755920
const BEATEN_RVA: usize = rva(0x755918); // dword_755918
const CHEATS_RVA: usize = rva(0x75870C); // g_cheats
const EXTRA: [usize; 2] = [PLAYER_COUNT_RVA, CHEATS_RVA];

/// Its calls (the first in the patched bytes).
const CALLS: [usize; 3] = [0x1, 0x1B, 0x33];
const CASES: u32 = 3;

fn test_entry(_case: u32) -> Regs {
    Regs::SENTINEL
}

fn prepare(case: u32) {
    unsafe { game_write(PLAYER_COUNT_RVA, 1 + case) };
    if let Ok(e) = current_event() {
        let target = get(e + 0x57C + 0x14);
        for n in 0..4u32 {
            if player(n) != 0 {
                let d = [0u32, 1, 0xFFFF_FFFF, 0x10][((n + case) % 4) as usize];
                put(player(n) + 0x88, target.wrapping_add(d));
            }
        }
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn record_entries_pass(entry: &Regs) -> Result<Regs, Refused> {
    let players = counted_players()?;
    current_event()?;
    if players.iter().any(|&p| get(p + 0x18) == 0) {
        return Err(BadInput("a player without a name").into());
    }
    unsafe {
        crate::menu::clear_record_entries::clear_record_entries(entry)?;
        game_write(ENTERED_RVA, 0u32);
        for n in 0..players.len() as u32 {
            game_write(CURRENT_RVA, n);
            let g = Regs { ecx: n, ..*entry };
            crate::menu::beat_target_time::beat_target_time(&g)?;
            if game_read::<u32>(BEATEN_RVA) == 1 {
                game_write(ENTERED_RVA, 1u32);
                crate::menu::enter_record_entry::enter_record_entry(&g)?;
            }
        }
    }
    Ok(*entry)
}

fn regions() -> Vec<Region> {
    let e = current_event().unwrap_or(0);
    let mut v = vec![
        (e, 0x618),
        (game_addr(KEPT_NAME_RVA) as u32, 0x1F),
        at(SLOT_RVA),
        at(HHMM_RVA),
        at(CURRENT_RVA),
        at(ENTERED_RVA),
        at(BEATEN_RVA),
        (game_addr(crate::race::save_seal::BLOCK_RVA) as u32, 8),
    ];
    v.extend(crate::race::save_seal::seal_regions());
    v.extend(
        (0..4u32)
            .map(|n| (player(n), 0x100))
            .filter(|&(p, _)| p != 0),
    );
    v
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { record_entries_pass(r) }?))
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
    let refused = unsafe { record_entries_pass(&test_entry(0)) }.is_err();
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
