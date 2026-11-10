//! `menu::find_cpu_team` (0x4A49FF): the driver (`g_drivers`, `standings::drivers`'s walk)
//! whose first dword is eax (a reading: the computer team's number); none: the fatal error 0x8070A098
//! "CpuTeam Not Found (oh sh_t!)".
//!
//! Refuses a table without its end entry, before anything;
//! raises the fatal error as the original.

use crate::errors::Refused;
use crate::menu::drivers_store::Driver;

/// The driver.
pub fn find_cpu_team(team: u32) -> Result<&'static Driver, Refused> {
    crate::menu::standings::drivers()?
        .iter()
        .find(|d| d.name_entry.get() == team)
        .ok_or(
            crate::errors::FatalError {
                code: 0x8070A098,
                message: "CpuTeam Not Found (oh sh_t!)",
            }
            .into(),
        )
}
