//! `menu::copy_standing_entry` (0x46613F): a standings entry (edi) filled from the player
//! record esi: +4 ebx + 1, +0x14 / +0x1C / +0x18 the record's +0x88 / +0x90 / +0x8C, +0xC
//! its name pointer (+0x18) and +0x20 the name's 0x20 bytes, +8 / +0x10 its +4 / +8, +0x4C
//! its +0xD0.
//!
//! In:  edi the entry, esi the record, ebx its index
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_4A2B75` and two of the game's. Refuses a null entry, record or name,
//! before anything. Self-test: the original and the Rust version on an entry and a record
//! of the test's.
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::copy_standing_entry", rva(0x46613F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// An entry, a record and a name of the test's.
struct Test {
    entry: [u32; 0x20],
    record: [u32; 0x40],
    name: [u8; 0x20],
}

static mut TEST: Test = Test {
    entry: [0xAAAA_AAAA; 0x20],
    record: [0; 0x40],
    name: *b"A test driver's name, 32 bytes!!",
};

/// The registers pointing at the test's entry and record.
unsafe fn test_regs() -> Regs {
    {
        let p = &raw const TEST;
        unsafe {
            Regs {
                edi: (&raw const (*p).entry) as u32,
                esi: (&raw const (*p).record) as u32,
                ebx: 2,
                ..Regs::SENTINEL
            }
        }
    }
}
const EXTRA: [usize; 0] = [];
const CASES: u32 = 2;

fn test_entry() -> Regs {
    unsafe { test_regs() }
}

fn refusal_expected() -> bool {
    false
}

fn prepare(case: u32) {
    let _ = case;
    unsafe {
        let p = &raw mut TEST;
        for k in 0..0x40usize {
            (*p).record[k] = (k as u32 * 0x1111 + case) & 0xFFFF;
        }
        (*p).record[0x18 / 4] = (&raw const (*p).name) as u32;
        (*p).entry = [0xAAAA_AAAA; 0x20];
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn copy_standing_entry(entry: &Regs) -> Result<Regs, Refused> {
    let (e, p) = (entry.edi, entry.esi);
    if e == 0 || p == 0 || get(p + 0x18) == 0 {
        return Err(BadInput("a null entry, record or name").into());
    }
    // Converted: `rc2k_core::menu::standings::fill_a`.
    use rc2k_core::menu::standings::{PlayerA, fill_a};
    let name_at = get(p + 0x18);
    let source = PlayerA {
        index: entry.ebx,
        time: get(p + 0x88),
        group: get(p + 0x90),
        total: get(p + 0x8C),
        name_at,
        name: bytes(name_at),
        nationality: get(p + 4),
        car: get(p + 8),
        entered: get(p + 0xD0),
    };
    // The entry's bytes as far as it is written (to +0x50).
    fill_a(&mut unsafe { table(e, 0x50, 1) }, 0, &source);
    Ok(*entry)
}

fn regions(r: &Regs) -> Vec<Region> {
    let _ = r;
    vec![(r.edi, 0x78)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { copy_standing_entry(r) }?))
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
    let refused = unsafe { copy_standing_entry(&before) }.is_err();
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
