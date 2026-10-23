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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::splits::{g, guarded, p};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::physics::splits::{RIVALS, default_rival};

/// The steps on `r`.
// Converted: `rc2k_core::physics::splits::default_rival`.
pub fn fill_rival_table_in(r: &mut Regs) -> Result<(), Refused> {
    for c in 0..RIVALS as u32 {
        r.ecx = c;
        let (time, car, name) = default_rival(c);
        p(0x601ED0 + 4 * c as usize, time);
        r.ebx = 0x17;
        r.eax = c / 0x17;
        r.edx = car;
        p(0x602038 + 4 * c as usize, car);
        r.eax = name;
        p(0x601D68 + 4 * c as usize, name);
    }
    r.ecx = 0x5A;
    r.esi = g(0x6238BC);
    if r.esi == 0 {
        return Ok(());
    }
    for c in 0..RIVALS {
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

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
