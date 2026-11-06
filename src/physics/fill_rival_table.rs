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
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.
use crate::errors::Refused;
use crate::race::owner::race;
use crate::race::vars_store::RACE_VARS;
use rc2k_core::physics::splits::{RIVALS, default_rival};

/// The Rust version. The registers the original leaves are read by none of its callers.
// Converted: `rc2k_core::physics::splits::default_rival`.
pub fn fill_rival_table() -> Result<(), Refused> {
    let physics = &race().physics;
    for c in 0..RIVALS as u32 {
        let (time, car, name) = default_rival(c);
        physics.split_tables.rival_times[c as usize].set(time);
        physics.split_tables.rival_cars[c as usize].set(car);
        physics.split_tables.rival_names[c as usize].set(name);
    }
    // The standings summary (`g_rivals` 1 while it is there: `RACE_STATE.settings.rivals`).
    if RACE_VARS.rivals.get() == 0 {
        return Ok(());
    }
    let rows = crate::saves::best_times::SUMMARY.with(|t| t.rows)?;
    for (c, r) in rows.iter().take(RIVALS).enumerate() {
        physics.split_tables.rival_names[c].set(r.name_id);
        physics.split_tables.rival_cars[c].set(r.car);
        physics.split_tables.rival_times[c].set(r.time);
    }
    Ok(())
}
