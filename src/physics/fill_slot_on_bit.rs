//! `physics::fill_slot_on_bit` (0x462237): when bit cl of `g_car` +0x298 is set, it is
//! cleared and part 0x50 + 0x14 x `g_currentCar` + cl goes into a race slot
//! (`physics::fill_race_slot`).
//!
//! In:  cl the bit (ecx passed on)
//! Out: eax 1 << cl, or as `fill_race_slot` leaves them; no flags read
//!
//! Callers: `physics::drop_broken_parts` (seven), `sub_4410C8` (four).
//! Refuses what `fill_race_slot` refuses (before anything: the bit is cleared after).
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): bits set and clear, cars 0 .. 7.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::physics::fill_race_slot::{fill_race_slot_in, g, inputs, p, regions, set_slots};
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore, state_entry};
use rc2k_core::physics::parts::{part_number, take_bit};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::fill_slot_on_bit", rva(0x462237), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const BITS: usize = 0x71A270; // g_car + 0x298

/// The steps on `r` (cl the bit).
// Converted: `rc2k_core::physics::parts::{take_bit, part_number}`.
pub fn fill_slot_on_bit_in(r: &mut Regs) -> Result<(), Refused> {
    let (mask, cleared) = take_bit(g(BITS), r.ecx);
    r.eax = mask;
    let Some(cleared) = cleared else {
        return Ok(());
    };
    let Some(part) = part_number(g(0x71B3F8), r.ecx) else {
        return Err(BadInput("a part past 0xFF").into());
    };
    p(BITS, cleared);
    r.eax = part;
    fill_race_slot_in(r)
}

/// The Rust version.
pub unsafe fn fill_slot_on_bit(entry: &Regs) -> Result<Left, Refused> {
    let all = regions();
    let saved = capture(&all);
    let mut r = *entry;
    match fill_slot_on_bit_in(&mut r) {
        Ok(()) => Ok(crate::race::stage_prefs::left_of(&r)),
        Err(e) => {
            restore(&all, &saved);
            Err(e)
        }
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { fill_slot_on_bit(r) }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut all = regions();
    all.extend(inputs());
    let saved = capture(&all);
    for (n, (cl, bits)) in [
        (9u32, 0x200u32),
        (9, 0),
        (13, 0xFFFF_FFFF),
        (0x11, 0x2_0000),
        (0xC, 0x1000),
    ]
    .into_iter()
    .enumerate()
    {
        for car in [0u32, 3, 7] {
            check_state(
                &mut t,
                &format!("bit {cl}, bits {bits:#x}, car {car}"),
                call_original,
                Regs {
                    ecx: cl,
                    ebp: 0x1234,
                    ..Regs::SENTINEL
                },
                &|| {
                    set_slots(n as u32 + car, Some(2));
                    p(0x71B3F8, car);
                    p(BITS, bits);
                },
                &all,
                &rust,
            );
        }
    }
    restore(&all, &saved);
    t.finish()
}
