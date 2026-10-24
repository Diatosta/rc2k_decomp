//! `input::update_car_effects` (0x40F347): one car's force feedback for one device, from
//! its physics (the view block in ebx) into its sound object (edi) and six requests
//! (`input::set_effect` on the device in esi). A reading of the code:
//! - the object's +0xAC = the block's +0x248 (1 while the car is off the ground, it seems);
//!   +0xB0 = +0x1A4 * 10, +0xB4 = (+0x1B8 - +0x1AC) * 10, +0xB8 = +0x1A4 * 10, +0xBC =
//!   +0x1AC * 10 (`fistp`, `flt_5180DC`);
//! - a knock (+0xC0): with +0xD0 and +0xC4 not ±0, (256 - |atan2(+0xCC, +0xC4)| *
//!   162.97) * +0xD0 * 0.1 (`flt_5180E0`, `flt_5180E4`, `flt_5180EC`; the angle also
//!   into `g_soundScratch`), else 0; then +0xC4 .. +0xD0 cleared;
//! - effect 1 (the surface's rumble): ecx the effect kind of the surface (+0x294, through
//!   `dword_51809C`), edx |+0x284| (the speed?); at 5 or less ecx 0 and edx +0x14 squared
//!   >> 19; edx held to 0x100;
//! - effects 4 and 0 by the kind (5 and 9, 7, 8, 0, others), then by +0xAC and +0x278 (the
//!   car's landing: effects 0, 1, 4; effects 0 and 2) or a steering force from +0xB0 and
//!   +0xB4 on effect 0;
//! - effect 3 once for a knock (±0x100, +0xDC counting to 4 before the next), effect 5
//!   the change of +0xBC since the last time (+0x27C), held to ±0x200, halved.
//!
//! Every request is a "set" (ebx 3). `g_controllers[+0 * 0x50]` is loaded into ebx and not
//! used.
//!
//! In:  ebx = the view block, edi = the car's sound object, esi = the device's record
//! Out: eax 5, ebx 3, ecx effect 5's number, edx as it was at the last call (+0xD8 on
//!      every way through but the first's), esi and edi kept; flags not read
//!
//! See `input/effects.rs`. Three callers, all in `input::update_effects`. The x87 work with the
//! original's instructions (its rounding).
//! Refuses (before anything) a surface number past the table's 16.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::input::devices::{at, put};
use crate::math::x87::{mul_to_int, x87};
use crate::sound::store::EffectsSlot;

const TEN: EffectsSlot = EffectsSlot::at(0x5180DC); // flt_5180DC: 10.0
const ANGLE_SCALE: EffectsSlot = EffectsSlot::at(0x5180E0); // flt_5180E0: 162.97
const FULL: EffectsSlot = EffectsSlot::at(0x5180E4); // flt_5180E4: 256.0
const TENTH: EffectsSlot = EffectsSlot::at(0x5180EC); // flt_5180EC: 0.1
/// dword_51809C: a surface's effect kind, 16 entries.
const KINDS: EffectsSlot = EffectsSlot::at(0x51809C);
const KIND_COUNT: u32 = 16;

fn k(x: impl crate::store::Ida) -> u32 {
    unsafe { crate::store::ida_read(x) }
}

/// `fld a; fsub b; fmul c; fistp`.
fn sub_mul_to_int(a: u32, b: u32, c: u32) -> i32 {
    let (v, mut out) = ([a, b, c], 0i32);
    x87!("fld dword ptr [{v}]", "fsub dword ptr [{v} + 4]", "fmul dword ptr [{v} + 8]",
        "fistp dword ptr [{o}]";
        v = v.as_ptr(), o = &raw mut out);
    out
}

/// `fld y; fld x; fpatan; fst angle; fabs; fmul s; fsubr full; fmul d; fmul tenth;
/// fistp`: (the angle's bits, the knock).
fn knock(y: u32, x: u32, s: u32, full: u32, d: u32, tenth: u32) -> (u32, i32) {
    let v = [y, x, s, full, d, tenth];
    let mut out = [0u32; 2];
    x87!("fld dword ptr [{v}]", "fld dword ptr [{v} + 4]", "fpatan",
        "fst dword ptr [{o}]", "fabs", "fmul dword ptr [{v} + 8]",
        "fsubr dword ptr [{v} + 12]", "fmul dword ptr [{v} + 16]",
        "fmul dword ptr [{v} + 20]", "fistp dword ptr [{o} + 4]";
        v = v.as_ptr(), o = out.as_mut_ptr());
    (out[0], out[1] as i32)
}

/// The original's ecx and edx, which carry the numbers from one request to the next.
#[derive(Default)]
struct Last {
    ecx: u32,
    edx: u32,
}

/// Exactly as the original (the x87 at its precision), the angle written to `scratch` (an
/// address). `view` the view block, `object` the car's sound object, `device` the device's
/// record; the requests through `input::set_effect`.
fn update_car_effects_with(
    view: u32,
    object: u32,
    device: u32,
    scratch: u32,
) -> Result<(), Refused> {
    let r = &mut Last::default();
    let surface = at(object, 0x294);
    if surface >= KIND_COUNT {
        return Err(BadInput("a surface number past the effect kinds' 16").into());
    }
    put(object, 0xAC, at(view, 0x248));
    let ten = k(TEN);
    put(object, 0xB0, mul_to_int(at(view, 0x1A4), ten) as u32);
    put(
        object,
        0xB4,
        sub_mul_to_int(at(view, 0x1B8), at(view, 0x1AC), ten) as u32,
    );
    put(object, 0xB8, mul_to_int(at(view, 0x1A4), ten) as u32);
    put(object, 0xBC, mul_to_int(at(view, 0x1AC), ten) as u32);
    if at(object, 0xD0) & 0x7FFF_FFFF != 0 && at(object, 0xC4) & 0x7FFF_FFFF != 0 {
        let (angle, force) = knock(
            at(object, 0xCC),
            at(object, 0xC4),
            k(ANGLE_SCALE),
            k(FULL),
            at(object, 0xD0),
            k(TENTH),
        );
        unsafe { (scratch as *mut u32).write_volatile(angle) };
        put(object, 0xC0, force as u32);
    } else {
        put(object, 0xC0, 0);
    }
    for o in [0xC4, 0xC8, 0xCC, 0xD0] {
        put(object, o, 0);
    }
    r.ecx = k(KINDS + 4 * surface as usize);
    r.edx = at(object, 0x284);
    if (r.edx as i32) < 0 {
        r.edx = r.edx.wrapping_neg();
    }
    if r.edx <= 5 {
        r.ecx = 0;
        let s = at(object, 0x14);
        r.edx = s.wrapping_mul(s) >> 19;
    }
    if r.edx > 0x100 {
        r.edx = 0x100;
    }
    let set = |r: &mut Last, eax: u32, ecx: u32, edx: u32| {
        r.ecx = ecx;
        r.edx = edx;
        crate::input::set_effect::set_effect(device, eax, 3, ecx, edx)
    };
    let d8 = || at(object, 0xD8);
    let (ecx, edx) = (r.ecx, r.edx);
    set(r, 1, ecx, edx)?;
    let mut to_703 = false;
    match r.ecx {
        5 | 9 | 0 => {
            set(r, 4, 0x100, d8())?;
            set(r, 0, 0, d8())?;
        }
        7 => {
            set(r, 4, 0x55, d8())?;
            set(r, 0, 0x55, d8())?;
        }
        8 => {
            set(r, 4, 0, d8())?;
            set(r, 0, 0, d8())?;
        }
        _ => {
            set(r, 4, 0, d8())?;
            if at(object, 0xAC) != 0 {
                put(object, 0x278, 1);
                set(r, 0, 0, d8())?;
                set(r, 1, 0, 0x100)?;
                set(r, 4, 0, d8())?;
                to_703 = true;
            } else if at(object, 0x278) == 1 {
                put(object, 0x278, 0);
                set(r, 0, 0x100, d8())?;
                let edx = r.edx;
                set(r, 2, 0x100, edx)?;
            } else {
                let mut a = (at(object, 0xB0) << 1).wrapping_add(at(object, 0xB4)) << 1;
                if (a as i32) < 0 {
                    a = a.wrapping_neg();
                }
                if (a as i32) > 0x14 {
                    let a = (a - 0x14).min(0x100);
                    set(r, 0, 0x100 - a, d8())?;
                } else {
                    set(r, 0, 0x100, d8())?;
                }
            }
        }
    }
    if !to_703 {
        // loc_40F67C: the knock.
        r.ecx = at(object, 0xC0);
        let mut counted = false;
        if r.ecx != 0 && at(object, 0xAC) == 0 {
            if at(object, 0xDC) == 0 {
                let ecx = if (r.ecx as i32) < 0 {
                    0xFFFF_FF00
                } else {
                    0x100
                };
                let edx = r.edx;
                set(r, 3, ecx, edx)?;
                put(object, 0xDC, at(object, 0xDC).wrapping_add(1));
                counted = true;
            } else {
                put(object, 0xC0, 0);
            }
        }
        if !counted && at(object, 0xDC) != 0 {
            put(object, 0xDC, at(object, 0xDC).wrapping_add(1));
            if at(object, 0xDC) >= 4 {
                put(object, 0xDC, 0);
            }
        }
    }
    // loc_40F703: the change of +0xBC.
    let now = at(object, 0xBC);
    let before = at(object, 0x27C);
    put(object, 0x27C, now);
    let change = (now.wrapping_sub(before) as i32).clamp(-0x200, 0x200) >> 1;
    let edx = r.edx;
    set(r, 5, change as u32, edx)
}

/// The car's force feedback for the frame (the original's x87 arithmetic). The
/// converted version (`rc2k_core::input::feedback::update_car_effects`, plain floats)
/// waits for the float pass.
pub fn update_car_effects(view: u32, object: u32, device: u32) -> Result<(), Refused> {
    update_car_effects_with(
        view,
        object,
        device,
        crate::session::MACHINE.scratch.addr() as u32,
    )
}
