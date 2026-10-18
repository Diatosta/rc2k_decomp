//! `physics::drop_broken_parts` (0x462386): seven parts of the work car checked against
//! their limits: when `g_car` +0xA3C + 4k is not 0x80000000 and not below (unsigned)
//! `dword_71BA2C` + 4k, `physics::fill_slot_on_bit` with bit 9, 0xD, 0xE, 0x10, 0x11, 0xC,
//! 0xF (the first only with `dword_62DAD0` set). A reading: damaged parts falling off.
//!
//! In:  nothing
//! Out: as the last `fill_slot_on_bit` leaves them; no flags read
//!
//! Caller: `sub_43B635` (0x43B98B), every physics step.
//! Refuses what `fill_slot_on_bit` refuses; what was written is put back.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): damage about the limits, the switch.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::fill_race_slot::{g, inputs, p, regions, set_slots};
use crate::physics::fill_slot_on_bit::fill_slot_on_bit_in;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "physics::drop_broken_parts",
    rva(0x462386),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The bits by part (the damage at +0xA3C + 4k).
const BITS: [u32; 7] = [9, 0xD, 0xE, 0x10, 0x11, 0xC, 0xF];

/// The steps on `r`.
pub fn drop_broken_parts_in(r: &mut Regs) -> Result<(), Refused> {
    for (k, bit) in BITS.iter().enumerate() {
        if k == 0 && g(0x62DAD0) == 0 {
            continue;
        }
        let damage = g(0x71AA14 + 4 * k);
        if damage == 0x8000_0000 {
            continue;
        }
        r.eax = g(0x71BA2C + 4 * k);
        if damage < r.eax {
            continue;
        }
        r.ecx = *bit;
        fill_slot_on_bit_in(r)?;
    }
    Ok(())
}

/// The Rust version.
pub unsafe fn drop_broken_parts(entry: &Regs) -> Result<Left, Refused> {
    let all = regions();
    let saved = capture(&all);
    let mut r = *entry;
    match drop_broken_parts_in(&mut r) {
        Ok(()) => Ok(crate::race::stage_prefs::left_of(&r)),
        Err(e) => {
            restore(&all, &saved);
            Err(e)
        }
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { drop_broken_parts(r) }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut all = regions();
    all.extend(inputs());
    let saved = capture(&all);
    for seed in 0..8u32 {
        check_state(
            &mut t,
            &format!("seed {seed}"),
            call_original,
            Regs {
                ebp: 0x1234,
                ..Regs::SENTINEL
            },
            &|| {
                set_slots(seed, Some(seed % 4));
                p(0x71B3F8, seed % 8);
                p(0x71A270, 0xFFFF_FFFF ^ (seed << 10));
                p(0x62DAD0, seed % 2);
                for k in 0..7usize {
                    let limit = 100 + 10 * k as u32;
                    p(0x71BA2C + 4 * k, limit);
                    let d = match (seed as usize + k) % 4 {
                        0 => 0x8000_0000,
                        1 => limit - 1,
                        2 => limit,
                        _ => limit + 50,
                    };
                    p(0x71AA14 + 4 * k, d);
                }
            },
            &all,
            &rust,
        );
    }
    restore(&all, &saved);
    t.finish()
}
