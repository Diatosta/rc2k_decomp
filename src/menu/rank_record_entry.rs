//! `menu::rank_record_entry` (0x4A3D75): the current record's event's tenth record entry
//! moved up (`menu::rank_record_entry::rank_entries`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `menu::enter_record_entry`. Refuses a record past the table or no events table,
//! before anything. Self-test: the original and the Rust version on the game's event, its
//! entries' times by lot; put back.
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::rank_record_entry", rva(0x4A3D75), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const EXTRA: [usize; 0] = [];
const CASES: u32 = 4;

/// The current record's event's ten record entries (0x9C each from +0x57C - 9 * 0x9C, its
/// +0x14 the time): the tenth moved up past each entry whose time is not below its own
/// (signed), each of +0xC, +0x14, +4, +8, +0x1C, +0x10, +0x18, +0x40, +0x4C .. +0x58 and the
/// name's 0x1F bytes at +0x20 swapped (the name through `g_keptPlayerName`).
pub fn rank_entries(event: u32) {
    let mut esi = event + 0x57C;
    for _ in 0..9 {
        let edi = esi - 0x9C;
        if get(esi + 0x14) as i32 <= get(edi + 0x14) as i32 {
            for f in [
                0xC, 0x14, 4, 8, 0x1C, 0x10, 0x18, 0x40, 0x4C, 0x50, 0x54, 0x58,
            ] {
                let a = get(edi + f);
                put(edi + f, get(esi + f));
                put(esi + f, a);
            }
            let kept = crate::framework::game_addr(KEPT_NAME_RVA) as *mut u8;
            unsafe {
                std::ptr::copy((edi + 0x20) as *const u8, kept, 0x1F);
                std::ptr::copy((esi + 0x20) as *const u8, (edi + 0x20) as *mut u8, 0x1F);
                std::ptr::copy(kept as *const u8, (esi + 0x20) as *mut u8, 0x1F);
            }
        }
        esi -= 0x9C;
    }
}

fn test_entry(_case: u32) -> Regs {
    Regs::SENTINEL
}

fn prepare(case: u32) {
    if let Ok(e) = current_event() {
        for k in 0..10u32 {
            let t = [k * 100, 1000 - k * 100, (k * 37) % 5 * 10, 500][case as usize];
            put(e + 0x57C - 0x9C * (9 - k) + 0x14, t);
            put(e + 0x57C - 0x9C * (9 - k) + 0x20, 0x41 + k);
        }
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn rank_record_entry(entry: &Regs) -> Result<Regs, Refused> {
    rank_entries(current_event()?);
    Ok(*entry)
}

fn regions() -> Vec<Region> {
    let e = current_event().unwrap_or(0);
    vec![(e, 0x9C * 10), (game_addr(KEPT_NAME_RVA) as u32, 0x1F)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { rank_record_entry(r) }?))
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
    let refused = unsafe { rank_record_entry(&test_entry(0)) }.is_err();
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
