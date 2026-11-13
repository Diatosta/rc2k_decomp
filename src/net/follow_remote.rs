//! `net::follow_remote` (0x464883): a remote car's step in a network race: while it has
//! steps of a glide left (the current car's block +0xFCC, set to 32 by
//! `net::apply_message`), one less, and a 32nd of the way (+0xFC0 .. +0xFC8) added to its
//! place (+0x134 .. +0x13C).
//!
//!
//! See `net/mod.rs`. One caller, `net::race_step`. Calls nothing.

use crate::net::apply_message::{glide_steps, place, three, wanted};
use crate::net::store::NetShared;
use crate::race::owner::Race;
use rc2k_core::net::race;
pub fn follow_remote(race: &Race, vars: &NetShared) -> Result<(), crate::errors::BadInput> {
    let car = race.shared.car();
    // Converted: `rc2k_core::net::race::glide` (the block read only with the race on).
    if vars.session.get() == 0 {
        return Ok(());
    }
    let steps = glide_steps(car).get();
    if steps == 0 {
        return Ok(());
    }
    let step = three(wanted(car));
    if let Some((left, new_place)) = race::glide(true, steps, three(place(car)), step) {
        glide_steps(car).set(left);
        for (cell, v) in place(car).iter().zip(new_place) {
            cell.set(v);
        }
    }
    Ok(())
}
