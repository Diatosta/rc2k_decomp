//! The car's parts repaired whole (IDA, 2026-10-15): for each of five parts a run of the
//! current player's car's dwords (as `service::apply_repairs` zeroes them) and its damage
//! dword zeroed, and `service::repair_car`, which does all of them and more.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, SelfTest};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, restore};
use crate::service::state::current_player;

/// A part: its run (offset, dwords) and its damage dword.
#[derive(Clone, Copy)]
pub struct Part {
    pub run: u32,
    pub dwords: u32,
    pub damage: u32,
}

pub const BODY: Part = Part {
    run: 0x1A4,
    dwords: 6,
    damage: 0xD8,
};
pub const GEARBOX: Part = Part {
    run: 0x17C,
    dwords: 10,
    damage: 0x104,
};
pub const BRAKES: Part = Part {
    run: 0x1EC,
    dwords: 3,
    damage: 0x100,
};
pub const LIGHTS: Part = Part {
    run: 0x1BC,
    dwords: 7,
    damage: 0xE8,
};
pub const SUSPENSION: Part = Part {
    run: 0x1D8,
    dwords: 5,
    damage: 0xF8,
};

pub fn player() -> Result<u32, BadInput> {
    current_player().ok_or(BadInput(
        "no current player object: the original writes through null",
    ))
}

unsafe fn zero(at: u32, dwords: u32) {
    for i in 0..dwords {
        unsafe { ((at + 4 * i) as *mut u32).write_unaligned(0) };
    }
}

pub unsafe fn repair_part(p: Part) -> Result<(), BadInput> {
    let player = player()?;
    unsafe {
        zero(player + p.run, p.dwords);
        zero(player + p.damage, 1);
    }
    Ok(())
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
    let player = player()?;
    unsafe {
        for d in CAR_DAMAGE {
            zero(player + d, 1);
        }
        repair_part(BODY)?;
        crate::service::reset_tyre_wear::reset_tyre_wear()?;
        repair_part(GEARBOX)?;
        repair_part(BRAKES)?;
        repair_part(LIGHTS)?;
        repair_part(SUSPENSION)?;
        crate::service::reset_tyre_wear::reset_tyre_wear()?;
        ((player + FLAGS) as *mut u32).write_unaligned(u32::MAX);
        zero(player + FIELD_24C, 8);
    }
    Ok(())
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
    for p in [BODY, GEARBOX, BRAKES, LIGHTS, SUSPENSION] {
        runs.extend([(p.run, p.dwords), (p.damage, 1)]);
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
