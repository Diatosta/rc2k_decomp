//! `menu::find_cpu_team` (0x4A49FF): the driver (`g_drivers`, 0x94 each) whose first dword
//! is eax (a reading: the computer team's number); none: the fatal error 0x8070A098
//! "CpuTeam Not Found (oh sh_t!)".
//!
//! In:  eax the number
//! Out: esi the driver; the rest kept
//!
//! Callers: `sub_4A47EA`, `sub_4A4933`. Refuses a table without its end entry, before anything;
//! raises the fatal error as the original.

use crate::errors::Refused;
use crate::menu::standings::*;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn find_cpu_team(team: u32) -> Result<u32, Refused> {
    let first = crate::menu::drivers_store::DRIVERS.first();
    let all = entries(first, DRIVER)?;
    // Converted: `rc2k_core::menu::standings::find_team`.
    let drivers = unsafe { table(first, DRIVER, all.len() as u32) };
    if let Some(n) = rc2k_core::menu::standings::find_team(&drivers, team) {
        return Ok(all[n]);
    }
    Err(crate::errors::FatalError {
        code: 0x8070A098,
        message: "CpuTeam Not Found (oh sh_t!)",
    }
    .into())
}
