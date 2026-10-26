//! The car's parts repaired whole (IDA, 2026-10-15): for each of five parts a run of the
//! current player's car's dwords (as `service::apply_repairs` zeroes them) and its damage
//! dword zeroed, and `service::repair_car`, which does all of them and more. Converted:
//! `rc2k_core::service::car`.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, SelfTest};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, restore};
use crate::service::state::current_player;
use rc2k_core::menu::player::Player;

pub use rc2k_core::service::car::Part;

pub fn player() -> Result<u32, BadInput> {
    current_player().ok_or(BadInput(
        "no current player object: the original writes through null",
    ))
}

/// The current player's record changed by `f` (read, changed, written back whole).
pub fn with_player(f: impl FnOnce(&mut Player)) -> Result<(), BadInput> {
    let at = player()? as *mut Player;
    unsafe {
        let mut p = at.read_unaligned();
        f(&mut p);
        at.write_unaligned(p);
    }
    Ok(())
}

/// A part's run (offset, dwords) and damage dword, for the tests.
pub fn runs_of(part: Part) -> [(u32, u32); 2] {
    match part {
        Part::Body => [(0x1A4, 6), (0xD8, 1)],
        Part::Gearbox => [(0x17C, 10), (0x104, 1)],
        Part::Brakes => [(0x1EC, 3), (0x100, 1)],
        Part::Lights => [(0x1BC, 7), (0xE8, 1)],
        Part::Suspension => [(0x1D8, 5), (0xF8, 1)],
    }
}

pub unsafe fn repair_part(part: Part) -> Result<(), BadInput> {
    // Converted: `rc2k_core::service::car` (`Player::repair`).
    with_player(|p| p.repair(part))
}

/// The other damage dwords `repair_car` zeroes itself.
const CAR_DAMAGE: [u32; 8] = [0xD4, 0xDC, 0xE0, 0xE4, 0xEC, 0xF0, 0xF4, 0xFC];
/// Set to -1 (`Player::flags_278`), and eight dwords at +0x24C (`field_24c`) zeroed.
const FLAGS: u32 = 0x278;
const FIELD_24C: u32 = 0x24C;

/// `service::repair_car`: the current car as new: the damage dwords, the body, the tyres'
/// wear, the gearbox, the brakes, the lights, the suspension, the tyres' wear again, the
/// flags -1 and +0x24C's eight dwords.
pub unsafe fn repair_car() -> Result<(), BadInput> {
    // Converted: `rc2k_core::service::car` (`Player::repair_car`).
    with_player(Player::repair_car)
}

/// The current car, whole.
pub fn regions() -> Vec<Region> {
    current_player()
        .map(|p| vec![(p, std::mem::size_of::<crate::menu::player::Player>())])
        .unwrap_or_default()
}

/// The dwords `repair_car` writes, for its test: (offset, dwords).
pub fn car_runs() -> Vec<(u32, u32)> {
    let mut runs: Vec<(u32, u32)> = CAR_DAMAGE.iter().map(|&d| (d, 1)).collect();
    for part in [
        Part::Body,
        Part::Gearbox,
        Part::Brakes,
        Part::Lights,
        Part::Suspension,
    ] {
        runs.extend(runs_of(part));
    }
    use crate::service::reset_tyre_wear::{TYRE_DAMAGE_OFFSET, WEAR_DWORDS, WEAR_OFFSET};
    runs.extend([
        (WEAR_OFFSET, WEAR_DWORDS),
        (TYRE_DAMAGE_OFFSET, 1),
        (FLAGS, 1),
        (FIELD_24C, 8),
    ]);
    runs
}

/// A repair's self-test: the current car, only the dwords in `runs` given test values (the
/// game may be reading the rest), the whole car compared, put back afterwards.
pub fn check_repair(
    t: &mut SelfTest,
    call_original: unsafe extern "C" fn(*mut Regs),
    rust: &dyn Fn(&Regs) -> Result<Left, Refused>,
    runs: &[(u32, u32)],
) -> bool {
    let regions = regions();
    if regions.is_empty() {
        return false;
    }
    let saved = capture(&regions);
    let player = regions[0].0;
    for fill in [0x1111_1111u32, 0xEEEE_EEEE] {
        check_state(
            t,
            &format!("fill={fill:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                for &(offset, dwords) in runs {
                    for i in 0..dwords {
                        ((player + offset + 4 * i) as *mut u32).write_unaligned(fill);
                    }
                }
            },
            &regions,
            rust,
        );
    }
    restore(&regions, &saved);
    true
}
