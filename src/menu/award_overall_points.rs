//! `menu::award_overall_points` (0x4A4933): the stage's overall championship points. The
//! points start at 0x14 (`seconds`); each entry of standings table B whose +0x10 is not 0,
//! in the table's order, gets them into its +0x38 + 4 * `section` and they go down by one
//! (not below 0); a computer's driver (+4 0, `menu::find_cpu_team` by its +0) also into
//! its +0x5C + 4 * `section`, a player (+4 the player + 1) into its +0xB4 + 4 * `section`.
//! Then each entry's total of its six (+0x38 .., the negative ones left out) into its
//! +0x50 and its driver's +0x74 or player's +0xCC.
//!
//! In:  nothing
//! Out: edi the end entry, ecx 6, edx the last total, eax the last entry's driver number
//! or player index, esi its driver or player (no `pusha`)
//!
//! Caller: `sub_4A0E60`. The search is its Rust version. Refuses no table, one without its end
//! entry, a section past 5, a player past 4 or missing, before anything; a driver not found is the
//! search's fatal error, before anything.
use crate::errors::{BadInput, Refused};
use crate::memory::blocks::Place;
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::event_flow::Fault;
use crate::menu::standings::*;

/// An entry's driver (with its number) or player (with its index).
fn owner(e: Place) -> Result<(u32, u32, bool), Refused> {
    match get_at(e.add(4))? {
        0 => {
            let driver = crate::menu::find_cpu_team::find_cpu_team(get_at(e)?)?;
            Ok((driver, get_at(e)?, true))
        }
        n if n <= 4 && player(n - 1) != 0 => Ok((player(n - 1), n - 1, false)),
        _ => Err(BadInput("a player past 4 or missing").into()),
    }
}

/// What `award_overall_points` leaves in the registers, which the championship's later
/// calls may read (esi reaches `scene::set_up_carousel` in the original): the last entry's
/// (number, owner's record, total) when there was an entry (eax, esi, edx; else they are
/// kept), ecx 6, edi the end of the table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Awarded {
    pub last: Option<(u32, u32, u32)>,
}

impl Awarded {
    /// The carried values as the original leaves them (eax, ecx, edx, esi; edi, the end
    /// of the table, is `self.end`).
    pub fn carry_on(&self, c: &mut crate::menu::screen_loop::Carried) {
        if let Some((number, owner, total)) = self.last {
            c.arg = number;
            c.cars = owner;
            c.clock = total;
        }
        c.measure_x = 6;
    }
}

/// The Rust version (see the header); what it leaves in the registers as `Awarded`.
pub fn award_overall_points() -> Result<Awarded, Refused> {
    let mut last = None;
    {
        let section: u32 = SECTION_RVA.get();
        let table = TABLE_B_RVA.get();
        let (Some(table), true) = (table, section <= 5) else {
            return Err(BadInput("no table, or a section past 5").into());
        };
        let table = Place::Owned(table);
        let all = entries(table, B_ENTRY)?;
        let mut owners = Vec::with_capacity(all.len());
        for &e in &all {
            owners.push(owner(e)?);
        }
        // Converted: `rc2k_core::menu::points` (`Points`, `total`).
        use rc2k_core::menu::points::{Points, total};
        let at = 4 * section;
        let mut points = Points::default();
        DRAW_WORK.seconds.set(points.0);
        for (&e, &(o, _, cpu)) in all.iter().zip(&owners) {
            if get_at(e.add(0x10))? == 0 {
                continue;
            }
            let given = points.take();
            DRAW_WORK.seconds.set(points.0);
            e.add(0x38 + at).write(given, "a standings entry")?;
            put(o + if cpu { 0x5C } else { 0xB4 } + at, given)?;
        }
        for (&e, &(o, number, cpu)) in all.iter().zip(&owners) {
            let fault = Fault::default();
            let total = total(std::array::from_fn(|k| {
                fault.get_at(e.add(0x38 + 4 * k as u32))
            }));
            fault.check()?;
            e.add(0x50).write(total, "a standings entry")?;
            put(o + if cpu { 0x74 } else { 0xCC }, total)?;
            last = Some((number, o, total));
        }
        Ok(Awarded { last })
    }
}
