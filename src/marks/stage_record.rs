//! `marks::stage_record` (0x4D2DE0): `marks::run_frame`'s pick of the record of
//! `dword_64165C` among `off_606320`'s 0x34 (0x5C bytes each; by their count and
//! `sub_4616AA`'s clamp to 0x33, the stages: a reading).
//!
//! In:  nothing
//! Out: eax the number, esi its record
//!
//! One caller, `marks::run_frame`. Refuses a number past the table (unsigned). Self-test:
//! the original and the Rust version on numbers 0, 1, 0x1F and 0x33 and the game's own;
//! put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::stage_record", rva(0x4D2DE0), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const NUMBER_RVA: usize = rva(0x64165C); // dword_64165C
const RECORDS_RVA: usize = rva(0x606320); // off_606320: 0x34 pointers
use rc2k_core::marks::race::STAGE_RECORDS as RECORDS;

/// The Rust version (see the header).
// Converted: `rc2k_core::marks::race::stage_record`
pub unsafe fn stage_record(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        g.eax = game_read(NUMBER_RVA);
        let n = rc2k_core::marks::race::stage_record(g.eax)?;
        g.esi = game_read(RECORDS_RVA + 4 * n);
    }
    Ok(g)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { stage_record(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
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
    let regions: [Region; 1] = [at(NUMBER_RVA)];
    let saved = capture(&regions);
    let own: u32 = unsafe { game_read(NUMBER_RVA) };
    for n in [0u32, 1, 0x1F, 0x33, own] {
        if n >= RECORDS {
            continue;
        }
        check_state(
            &mut t,
            &format!("number {n:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe { game_write(NUMBER_RVA, n) },
            &regions,
            &rust,
        );
    }
    unsafe { game_write(NUMBER_RVA, RECORDS) };
    let refused = unsafe { stage_record(&Regs::SENTINEL) }.is_err();
    t.check(refused, || "number 0x34 not refused".into());
    restore(&regions, &saved);
    t.finish()
}
