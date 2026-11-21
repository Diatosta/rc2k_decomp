//! `menu::find_cpu_team`: the driver (`standings::walked`'s walk) whose first dword is the
//! given one (a reading: the computer team's number); none: the fatal error 0x8070A098
//! "CpuTeam Not Found (oh sh_t!)".
//!
//! Refuses a table without its end entry, before anything; raises the fatal error.

use crate::errors::Refused;
use crate::menu::owner::Menus;

/// The driver's number among the walked ones.
pub(super) fn find_cpu_team(menus: &Menus, team: u32) -> Result<usize, Refused> {
    let count = crate::menu::standings::walked(&menus.cars)?;
    menus.cars.drivers.read().drivers[..count]
        .iter()
        .position(|d| d.name_entry == team)
        .ok_or(
            crate::errors::FatalError {
                code: 0x8070A098,
                message: "CpuTeam Not Found (oh sh_t!)",
            }
            .into(),
        )
}
