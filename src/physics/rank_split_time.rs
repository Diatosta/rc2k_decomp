//! `physics::rank_split_time` (0x436C4F, race mode 1): the car's place among the 90
//! rivals at this split. `physics::fill_rival_table`; the figures +0xA0C .. +0xA1C -1 and
//! the two name fields (`dword_6D1838` .., `dword_6D1844` ..) blanks; each rival's time
//! scaled to the split: `dword_601ED0[c]` x (`dword_601030[split]` x 1000 /
//! `dword_60102C[g_splitColourCount]`) / 1000 into `dword_6021A0[c]` (`mul` / `div`); the
//! place (+0xA0C) one past the first not below the car's time (90 at most);
//! `dword_602308` the time; but for first place the gap to the one ahead (x10) into
//! +0xA10, its car +0xA18 and name id +0xA20, the name (`strings::append_string_by_id`
//! into `text`) into `dword_6D1844` ..; but for last place the gap to the one behind into
//! +0xA14, its car +0xA1C, name id +0xA24 and name into `dword_6D1838` ...
//!
//! In:  nothing
//! Out: the registers as left (ebx the cursor after a name, ebp 1000); no flags read
//!
//! Caller: `physics::split_by_mode`. Refuses a split or a split count past 0x40, a divisor
//! of 0 or a quotient past a dword (the original's `div` would fault); what was written is
//! put back.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.
use crate::errors::{BadInput, Refused};
use crate::physics::fill_rival_table::fill_rival_table;
use crate::race::owner::Race;
use crate::session::Dword;
use crate::strings::append_string_by_id::append_string_by_id;
use crate::strings::owner::strings;
use rc2k_core::physics::splits::{RIVALS, gap, rank, scaled_time, split_scale};

/// A name into `text` (eight blanks first), its first eight bytes into `to`.
fn name(id: u32, to: &[Dword; 2]) -> Result<(), crate::errors::BadInput> {
    let strings = strings();
    let text = crate::pointers::GamePtr::to(&strings.text.buffer.text);
    strings.text.buffer.text.set_u32(0, 0x2020_2020);
    strings.text.buffer.text.set_u32(4, 0x2020_2020);
    let mut written = 0u32;
    let mut refused = None;
    append_string_by_id(id, &mut |bytes| {
        if let Err(e) = text.offset(written).write_bytes(bytes, "a split name") {
            refused.get_or_insert(e);
        }
        written += bytes.len() as u32;
    })?;
    to[0].set(strings.text.buffer.text.get_u32(0));
    to[1].set(strings.text.buffer.text.get_u32(4));
    Ok(())
}

/// The Rust version. The registers the original leaves are read by none of its callers.
// Converted: `rc2k_core::physics::splits::{split_scale, scaled_time, rank, gap}`.
pub fn rank_split_time(race: &Race) -> Result<(), Refused> {
    let shared = &race.shared;
    let (split, count) = (
        shared.car.splits_recorded.get(),
        race.physics.split_tables.split_colour_count.get(),
    );
    if split > 0x40 || count > 0x40 {
        return Err(BadInput("a split or a split count past 0x40").into());
    }
    fill_rival_table(race)?;
    for k in 0..5 {
        shared.car.split_gaps[k].set(u32::MAX);
    }
    for label in [
        &race.frame.panel_and_overlay.overlay.label_behind,
        &race.frame.panel_and_overlay.overlay.label_ahead,
    ] {
        label[0].set(0x2020_2020);
        label[1].set(0x2020_2020);
    }
    let base = race.physics.split_tables.checkpoints[1 + split as usize].get();
    let d = race.physics.split_tables.checkpoints[count as usize].get();
    let (scale, _) = split_scale(base, d)?;
    for c in 0..RIVALS {
        let (q, _) = scaled_time(race.physics.split_tables.rival_times[c].get(), scale)?;
        race.physics.split_tables.rival_scaled[c].set(q);
    }
    // (A loop whose work was cut out: it left only registers, read by no one.)
    let time = shared.car.stage_time.get();
    let (c, _) = rank(time, |c| race.physics.split_tables.rival_scaled[c].get());
    shared.car.split_gaps[0].set(c);
    race.physics.split_tables.car_time.set(time);
    let c = c as usize;
    if c != 1 {
        // The one ahead: rival c - 2 (its scaled time, car and name).
        shared.car.split_gaps[1].set(gap(
            race.physics.split_tables.rival_scaled[c - 2].get(),
            time,
        ));
        shared.car.split_gaps[3].set(race.physics.split_tables.rival_cars[c - 2].get());
        let id = race.physics.split_tables.rival_names[c - 2].get();
        shared.car.split_gaps[5].set(id);
        name(id, &race.frame.panel_and_overlay.overlay.label_ahead)?;
    }
    if c != RIVALS {
        // The one behind: rival c - 1.
        shared.car.split_gaps[2].set(gap(
            time,
            race.physics.split_tables.rival_scaled[c - 1].get(),
        ));
        shared.car.split_gaps[4].set(race.physics.split_tables.rival_cars[c - 1].get());
        let id = race.physics.split_tables.rival_names[c - 1].get();
        shared.car.split_gaps[6].set(id);
        name(id, &race.frame.panel_and_overlay.overlay.label_behind)?;
    }
    Ok(())
}
