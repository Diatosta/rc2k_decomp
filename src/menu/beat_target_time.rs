//! `menu::beat_target_time` (0x4A3C23): `dword_755918` 1 when the current player's
//! (`dword_75598C`) time (+0x88) is at most the current event's target (its +0x57C +
//! 0x14; signed), else 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A3BE0`. Refuses a record past the table, no events table, or a current
//! player past 3 or missing, before anything. Self-test: the original and the Rust version
//! on the game's tables, times by lot; put back.
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::beat_target_time", rva(0x4A3C23), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CURRENT_RVA: usize = rva(0x75598C); // dword_75598C
const FLAG_RVA: usize = rva(0x755918); // dword_755918
const EXTRA: [usize; 1] = [CURRENT_RVA];
const CASES: u32 = 4;

fn test_entry() -> Regs {
    Regs::SENTINEL
}

fn refusal_expected() -> bool {
    false
}

fn prepare(case: u32) {
    let _ = case;
    let n: u32 = unsafe { game_read(CURRENT_RVA) };
    if n < 4
        && player(n) != 0
        && let Ok(e) = current_event()
    {
        let target = get(e + 0x57C + 0x14);
        put(
            player(n) + 0x88,
            target.wrapping_add([0u32, 1, 0xFFFF_FFFF, 0x1000][case as usize]),
        );
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn beat_target_time(entry: &Regs) -> Result<Regs, Refused> {
    let event = current_event()?;
    let n: u32 = unsafe { game_read(CURRENT_RVA) };
    if n >= 4 || player(n) == 0 {
        return Err(BadInput("a current player past 3 or missing").into());
    }
    let target = get(event + 0x57C + 0x14) as i32;
    let beaten = (get(player(n) + 0x88) as i32) <= target;
    unsafe { game_write(FLAG_RVA, u32::from(beaten)) };
    Ok(*entry)
}

fn regions(r: &Regs) -> Vec<Region> {
    let _ = r;
    vec![at(FLAG_RVA)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { beat_target_time(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions(r);
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let before = test_entry();
    let regions = regions(&before);
    if regions.iter().any(|&(p, _)| p == 0) {
        return t.skip("the game's tables are not there");
    }
    let mut vars = regions.clone();
    vars.extend(EXTRA.map(at));
    let saved = capture(&vars);
    // The probe runs on the saved tables: it writes what the tests write.
    let refused = unsafe { beat_target_time(&before) }.is_err();
    restore(&vars, &saved);
    if refused && !refusal_expected() {
        return t.skip("the Rust version refuses the game's tables");
    }
    for case in 0..CASES {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            test_entry(),
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
