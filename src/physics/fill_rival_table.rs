//! `physics::fill_rival_table` (0x436BD6): the 90 rival times: for each c, `dword_601ED0[c]`
//! = 12000 + 1000 c, `dword_602038[c]` = c % 23 (`div` by ebx 23), `id[c]`
//! (`dword_601D68`) = 0x5A5 + c (a string id: the rival's name); then, when
//! `dword_6238BC` points at a table of 90 records of 0x50 bytes, each record's +0, +8 and
//! +0xC into `id`, `dword_602038` and `dword_601ED0` instead.
//!
//! In:  nothing
//! Out: eax, ebx 23, ecx 90, edx, esi (the table's end, or 0) as left; no flags read
//!
//! Callers: `physics::rank_split_time`, `sub_435F43`.
//! Self-test: `physics/splits.rs` (`test_splits`), with and without a table.
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::splits::{g, guarded, p, test_splits, written};
use crate::screens::harness::Left;
use crate::service::harness::dispatch_state;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::fill_rival_table", rva(0x436BD6), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The steps on `r`.
pub fn fill_rival_table_in(r: &mut Regs) -> Result<(), Refused> {
    for c in 0..0x5Au32 {
        r.ecx = c;
        r.eax = c.wrapping_mul(1000).wrapping_add(0x2EE0);
        p(0x601ED0 + 4 * c as usize, r.eax);
        r.ebx = 0x17;
        r.eax = c / 0x17;
        r.edx = c % 0x17;
        p(0x602038 + 4 * c as usize, r.edx);
        r.eax = c + 0x5A5;
        p(0x601D68 + 4 * c as usize, r.eax);
    }
    r.ecx = 0x5A;
    r.esi = g(0x6238BC);
    if r.esi == 0 {
        return Ok(());
    }
    for c in 0..0x5Ausize {
        let at = |o: u32| unsafe { ((r.esi + o) as *const u32).read_unaligned() };
        r.eax = at(0);
        p(0x601D68 + 4 * c, r.eax);
        r.eax = at(8);
        p(0x602038 + 4 * c, r.eax);
        r.eax = at(12);
        p(0x601ED0 + 4 * c, r.eax);
        r.esi = r.esi.wrapping_add(0x50);
    }
    r.ecx = 0x5A;
    Ok(())
}

/// The Rust version.
pub unsafe fn fill_rival_table(entry: &Regs) -> Result<Left, Refused> {
    guarded(entry, &fill_rival_table_in)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { fill_rival_table(r) }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &written(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    test_splits(&mut t, call_original, &rust, &|c| c.mode == 1);
    t.finish()
}
