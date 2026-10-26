//! `menu::award_stage_points` (0x4A47EA): the stage's championship points. For each class
//! (`dword_7558F4`, 0 .. `dword_7558F8`) the points start at 0x14 (`seconds`); each entry
//! of standings table B in that class, in the table's order, is logged ("Status of
//! driver" and its +0x74), gets the points into its +0x54 + 4 * `section` and the points
//! go down by one (not below 0). A computer's driver (+4 0, `menu::find_cpu_team` by its
//! +0) keeps them in its +0x78 + 4 * `section` when its +0x24 is 1, a player (+4 the
//! player + 1) in its +0x98 + 4 * `section` when its +0xD0 is 1; otherwise both are 0.
//! Then each entry's total of its six stage points (+0x54 .., the negative ones left out)
//! into its +0x6C and its driver's +0x90 or player's +0xB0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers. The search is its Rust version; it logs through `logfile::log_write`
//! and `log_int` (level 1). Refuses no table, one without its end entry, a class count of 0 or past
//! 0x100, a section past 5, a player past 4 or missing, before anything; a driver not found is the
//! search's fatal error, before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::event_flow::Fault;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::standings::*;

const STATUS_TEXT_RVA: &std::ffi::CStr = c"Status of driver"; // aStatusOfDriver
/// The driver (computer, by `find_cpu_team`) or player of an entry, and the field its
/// stage points go to, and whether it keeps them.
enum Owner {
    Driver(u32),
    Player(u32),
}

fn owner(e: u32) -> Result<Owner, Refused> {
    match get(e + 4)? {
        0 => {
            let driver = crate::menu::find_cpu_team::find_cpu_team(get(e)?)?;
            Ok(Owner::Driver(driver))
        }
        n if n <= 4 && player(n - 1) != 0 => Ok(Owner::Player(player(n - 1))),
        _ => Err(BadInput("a player past 4 or missing").into()),
    }
}

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn award_stage_points() -> Result<(), Refused> {
    unsafe {
        let classes: u32 = EVENT_STATE.classes.get();
        let section: u32 = SECTION_RVA.get();
        let table: u32 = TABLE_B_RVA.get();
        if table == 0 || classes == 0 || classes > 0x100 || section > 5 {
            return Err(BadInput(
                "no table, a class count of 0 or past 0x100, or a section past 5",
            )
            .into());
        }
        let all = entries(table, B_ENTRY)?;
        let mut owners = Vec::with_capacity(all.len());
        for &e in &all {
            owners.push(owner(e)?);
        }
        // Converted: `rc2k_core::menu::points` (`Points`, `kept`, `total`).
        use rc2k_core::menu::points::{Points, kept, total};
        let at = 4 * section;
        for class in 0..classes {
            let mut points = Points::default();
            DRAW_WORK.seconds.set(points.0);
            for (&e, o) in all.iter().zip(&owners) {
                EVENT_STATE.class.set(class);
                if get(e + 0x10)? != class {
                    continue;
                }
                crate::logfile::log_write::log_write(
                    STATUS_TEXT_RVA.as_ptr() as *const u8,
                    false,
                    1,
                );
                crate::logfile::log_int::log_int(get(e + 0x74)? as i32, true, 1);
                let given = points.take();
                DRAW_WORK.seconds.set(points.0);
                let (keeps, field) = match *o {
                    Owner::Driver(d) => (get(d + 0x24)? == 1, d + 0x78 + at),
                    Owner::Player(p) => (get(p + 0xD0)? == 1, p + 0x98 + at),
                };
                put(e + 0x54 + at, kept(given, keeps))?;
                put(field, kept(given, keeps))?;
            }
        }
        for (&e, o) in all.iter().zip(&owners) {
            let fault = Fault::default();
            let total = total(std::array::from_fn(|k| fault.get(e + 0x54 + 4 * k as u32)));
            fault.check()?;
            put(e + 0x6C, total)?;
            match *o {
                Owner::Driver(d) => put(d + 0x90, total)?,
                Owner::Player(p) => put(p + 0xB0, total)?,
            }
        }
    }
    Ok(())
}
