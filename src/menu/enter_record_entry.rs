//! `menu::enter_record_entry` (0x4A3C7F): the current player (`dword_75598C`) entered as
//! the current record's event's tenth record entry (its +0x57C): +0x14 the time (+0x88),
//! +0x1C its +0x90, +8 its +4, +0xC its name pointer (+0x18), +0x10 its +8, +0x4C its
//! +0xD0, +0x50 `g_cheats`, +0x54 / +0x58 the sealed time (`menu::sealed_player_time` with
//! the event's number), +4 the player + 1, +0x40 1, +0x20 the name's 0x1F bytes; the
//! event's number into `dword_7559EC` first; then moved up (`menu::rank_record_entry`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `menu::record_entries_pass`. The steps are their Rust versions. Refuses a record
//! past the table, no events table, a current player past 3 or missing, or no name, before
//! anything. Self-test: the original and the Rust version on the game's tables, times by
//! lot; put back.
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::enter_record_entry", rva(0x4A3C7F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CURRENT_RVA: usize = rva(0x75598C); // dword_75598C
const SLOT_RVA: usize = rva(0x7559EC); // dword_7559EC
const HHMM_RVA: usize = rva(0x7559F0); // hhmm
const CHEATS_RVA: usize = rva(0x75870C); // g_cheats
const EXTRA: [usize; 2] = [CURRENT_RVA, CHEATS_RVA];
const CASES: u32 = 4;

fn test_entry(_case: u32) -> Regs {
    Regs::SENTINEL
}

fn prepare(case: u32) {
    let n: u32 = unsafe { game_read(CURRENT_RVA) };
    if n < 4 && player(n) != 0 {
        put(
            player(n) + 0x88,
            [0u32, 0x5000, 0x7FFF_FFFF, 0x300][case as usize],
        );
    }
    if let Ok(e) = current_event() {
        for k in 0..10u32 {
            put(e + 0x57C - 0x9C * (9 - k) + 0x14, k * 0x400 + case);
        }
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn enter_record_entry(entry: &Regs) -> Result<Regs, Refused> {
    let event = current_event()?;
    let n: u32 = unsafe { game_read(CURRENT_RVA) };
    if n >= 4 || player(n) == 0 || get(player(n) + 0x18) == 0 {
        return Err(BadInput("a current player past 3, missing or without a name").into());
    }
    let p = player(n);
    let slot = (event - unsafe { game_read::<u32>(EVENTS_RVA) }) / 0x618;
    unsafe { game_write(SLOT_RVA, slot) };
    let cheats = unsafe { game_read(CHEATS_RVA) };
    let sealed = unsafe {
        crate::menu::sealed_player_time::sealed_player_time(&Regs {
            eax: slot,
            ..*entry
        })
    }?;
    // Converted: `rc2k_core::menu::bests::enter_tenth`, then `rank_entries`.
    use rc2k_core::menu::bests;
    let name_at = get(p + 0x18);
    let best = bests::NewBest {
        key: get(p + 0x88),
        player: n + 1,
        nationality: get(p + 4),
        name_at,
        car: get(p + 8),
        group: get(p + 0x90),
        name: bytes(name_at),
        entered: Some(get(p + 0xD0)),
        seal: Some(bests::Seal {
            cheats,
            sealed: [sealed.eax, sealed.ebx],
        }),
        line: None,
    };
    bests::enter_tenth(
        &mut crate::menu::rank_record_entry::event_list(event),
        &best,
    );
    crate::menu::rank_record_entry::rank_entries(event);
    Ok(*entry)
}

fn regions() -> Vec<Region> {
    let e = current_event().unwrap_or(0);
    let mut v = vec![
        (e, 0x9C * 10),
        (game_addr(KEPT_NAME_RVA) as u32, 0x1F),
        at(SLOT_RVA),
        at(HHMM_RVA),
        (game_addr(crate::race::save_seal::BLOCK_RVA) as u32, 8),
    ];
    v.extend(crate::race::save_seal::seal_regions());
    v
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { enter_record_entry(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions();
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
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
    // `prepare` writes the players' +0x88: put back too (left changed until 2026-10-18).
    vars.extend(
        (0..4u32)
            .map(player)
            .filter(|&p| p != 0)
            .map(|p| (p + 0x88, 4)),
    );
    let saved = capture(&vars);
    let refused = unsafe { enter_record_entry(&test_entry(0)) }.is_err();
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
