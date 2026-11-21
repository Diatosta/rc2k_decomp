//! Race mode 1: the car's place among the 90 rivals at this split. `physics::fill_rival_table`; the
//! place figures are -1 and the two name fields (`label_behind`, `label_ahead`) blanks; each
//! rival's time is scaled to the split: `rival_times[c]` x (`SplitTables.checkpoints[1][split]` x
//! 1000 / `checkpoints[split_colour_count]`) / 1000 into `rival_scaled[c]`; the place is one past
//! the first not below the car's time (90 at most); `car_time` is the time; but for first place the
//! gap to the one ahead (x10), its car and name id go to their fields and the name
//! (`strings::append_string_by_id` into `text`) into `label_ahead`; but for last place the gap to
//! the one behind, its car, name id and name into `label_behind`.
//!
//! Refuses a split or a split count past 0x40, a divisor of 0 or a quotient past a dword; what was
//! written is put back.
use crate::errors::{BadInput, Refused};
use crate::memory::session::PlainDword;
use crate::physics::fill_rival_table::fill_rival_table;
use crate::physics::split_tables_store::SplitTables;
use crate::race::overlay_store::OverlayBlock;
use crate::race::owner_shared::RaceShared;
use crate::strings::append_string_by_id::append_string_by_id;
use rc2k_core::physics::splits::{RIVALS, gap, rank, scaled_time, split_scale};
use rc2k_core::saves::name_id::NameId;

/// A name into `text` (eight blanks first), its first eight bytes into `to`.
fn name(
    o: crate::owners::Owners<'static>,
    name: NameId,
    to: &[PlainDword; 2],
) -> Result<(), crate::errors::BadInput> {
    let strings = o.strings;
    let text = &strings.text.buffer.text;
    strings.text.buffer.text.set_u32(0, 0x2020_2020);
    strings.text.buffer.text.set_u32(4, 0x2020_2020);
    let mut written = 0u32;
    let mut refused = None;
    append_string_by_id(
        crate::strings::append_string_by_id::StringOwners::of(o),
        name.to_file(),
        &mut |bytes| {
            if let Err(e) = text.write_at(written as usize, bytes, "a split time's name") {
                refused.get_or_insert(e);
            }
            written += bytes.len() as u32;
        },
    )?;
    to[0].set(strings.text.buffer.text.get_u32(0));
    to[1].set(strings.text.buffer.text.get_u32(4));
    Ok(())
}

// Core: `rc2k_core::physics::splits::{split_scale, scaled_time, rank, gap}`.
pub fn rank_split_time(
    o: crate::owners::Owners<'static>,
    split_tables: &mut SplitTables,
    overlay: &OverlayBlock,
    shared: &RaceShared,
) -> Result<(), Refused> {
    let (split, count) = (
        shared.car().splits_recorded.get(),
        split_tables.split_colour_count,
    );
    if split > 0x40 || count > 0x40 {
        return Err(BadInput("a split or a split count past 0x40").into());
    }
    fill_rival_table(split_tables, shared.vars.get(), &o.saves.best_tables)?;
    for k in 0..5 {
        shared.car().split_gaps[k].set(u32::MAX);
    }
    for label in [&overlay.label_behind, &overlay.label_ahead] {
        label[0].set(0x2020_2020);
        label[1].set(0x2020_2020);
    }
    let base = split_tables.checkpoints[1 + split as usize];
    let d = split_tables.checkpoints[count as usize];
    let (scale, _) = split_scale(base, d)?;
    for c in 0..RIVALS {
        let (q, _) = scaled_time(split_tables.rival_times[c], scale)?;
        split_tables.rival_scaled[c] = q;
    }
    // (A loop whose work was cut out: it left only registers, read by no one.)
    let time = shared.car().stage_time.get();
    let (c, _) = rank(time, |c| split_tables.rival_scaled[c]);
    shared.car().split_gaps[0].set(c);
    split_tables.car_time = time;
    let c = c as usize;
    if c != 1 {
        // The one ahead: rival c - 2 (its scaled time, car and name).
        shared.car().split_gaps[1].set(gap(split_tables.rival_scaled[c - 2], time));
        shared.car().split_gaps[3].set(split_tables.rival_cars[c - 2]);
        let id = split_tables.rival_names[c - 2];
        shared.car().split_gaps[5].set(id.to_file());
        name(o, id, &overlay.label_ahead)?;
    }
    if c != RIVALS {
        // The one behind: rival c - 1.
        shared.car().split_gaps[2].set(gap(time, split_tables.rival_scaled[c - 1]));
        shared.car().split_gaps[4].set(split_tables.rival_cars[c - 1]);
        let id = split_tables.rival_names[c - 1];
        shared.car().split_gaps[6].set(id.to_file());
        name(o, id, &overlay.label_behind)?;
    }
    Ok(())
}
