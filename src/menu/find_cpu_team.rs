//! `menu::find_cpu_team` (0x4A49FF): the driver (`g_drivers`, 0x94 each) whose first dword
//! is eax (a reading: the computer team's number); none: the fatal error 0x8070A098
//! "CpuTeam Not Found (oh sh_t!)".
//!
//! In:  eax the number
//! Out: esi the driver; the rest kept
//!
//! Callers: `sub_4A47EA`, `sub_4A4933`. Refuses a table without its end entry, before
//! anything; raises the fatal error as the original. Self-test: the original and the Rust
//! version on the game's drivers, numbers there and (the fatal error checked in Rust
//! only) not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::standings::*;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn find_cpu_team(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    let first = crate::framework::game_addr(DRIVERS_RVA) as u32;
    let all = entries(first, DRIVER)?;
    // Converted: `rc2k_core::menu::standings::find_team`.
    let drivers = unsafe { table(first, DRIVER, all.len() as u32) };
    if let Some(n) = rc2k_core::menu::standings::find_team(&drivers, g.eax) {
        g.esi = all[n];
        return Ok(g);
    }
    Err(crate::errors::FatalError {
        code: 0x8070A098,
        message: "CpuTeam Not Found (oh sh_t!)",
    }
    .into())
}
