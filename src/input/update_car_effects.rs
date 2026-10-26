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
use crate::input::devices::at;
use crate::math::x87::mul_to_int;
use crate::math::x87_float::{ld, st, to_int};
use crate::pointers::GamePtr;

/// dword_51809C: a surface's effect kind, 16 entries.
const KIND_COUNT: u32 = 16;

/// `fld a; fsub b; fmul c; fistp`.
fn sub_mul_to_int(a: u32, b: u32, c: u32) -> i32 {
    to_int((ld(a) - ld(b)) * ld(c))
}

/// `fld y; fld x; fpatan; fst angle; fabs; fmul s; fsubr full; fmul d; fmul tenth;
/// fistp`: (the angle's bits, the knock).
fn knock(y: u32, x: u32, s: u32, full: u32, d: u32, tenth: u32) -> (u32, i32) {
    let angle = ld(y).atan2(ld(x));
    let knock = (ld(full) - angle.abs() * ld(s)) * ld(d) * ld(tenth);
    (st(angle), to_int(knock))
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
    let o = crate::sound::store::SoundObject::at(object)?;
    let surface = o.surface.get();
    if surface >= KIND_COUNT {
        return Err(BadInput("a surface number past the effect kinds' 16").into());
    }
    o.flag_ac.set(at(view, 0x248)?);
    let effects = &crate::input::store::FORCE_FEEDBACK;
    let ten = effects.effect_ten.bits();
    o.effect_in[0].set(mul_to_int(at(view, 0x1A4)?, ten) as u32);
    o.effect_in[1].set(sub_mul_to_int(at(view, 0x1B8)?, at(view, 0x1AC)?, ten) as u32);
    o.effect_in[2].set(mul_to_int(at(view, 0x1A4)?, ten) as u32);
    o.effect_in[3].set(mul_to_int(at(view, 0x1AC)?, ten) as u32);
    if o.knock_in[3].get() & 0x7FFF_FFFF != 0 && o.knock_in[0].get() & 0x7FFF_FFFF != 0 {
        let (angle, force) = knock(
            o.knock_in[2].get(),
            o.knock_in[0].get(),
            effects.effect_angle_scale.bits(),
            effects.effect_full.bits(),
            o.knock_in[3].get(),
            effects.effect_tenth.bits(),
        );
        GamePtr::new(scratch).write(angle, "the angle's scratch")?;
        o.knock.set(force as u32);
    } else {
        o.knock.set(0);
    }
    for d in &o.knock_in {
        d.set(0);
    }
    r.ecx = effects.surface_effect_kinds[surface as usize].get();
    r.edx = o.sum_a.get();
    if (r.edx as i32) < 0 {
        r.edx = r.edx.wrapping_neg();
    }
    if r.edx <= 5 {
        r.ecx = 0;
        let s = o.revs.get();
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
    let d8 = || o.effect_d8.get();
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
            if o.flag_ac.get() != 0 {
                o.landed.set(1);
                set(r, 0, 0, d8())?;
                set(r, 1, 0, 0x100)?;
                set(r, 4, 0, d8())?;
                to_703 = true;
            } else if o.landed.get() == 1 {
                o.landed.set(0);
                set(r, 0, 0x100, d8())?;
                let edx = r.edx;
                set(r, 2, 0x100, edx)?;
            } else {
                let mut a = (o.effect_in[0].get() << 1).wrapping_add(o.effect_in[1].get()) << 1;
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
        r.ecx = o.knock.get();
        let mut counted = false;
        if r.ecx != 0 && o.flag_ac.get() == 0 {
            if o.knock_count.get() == 0 {
                let ecx = if (r.ecx as i32) < 0 {
                    0xFFFF_FF00
                } else {
                    0x100
                };
                let edx = r.edx;
                set(r, 3, ecx, edx)?;
                o.knock_count.set(o.knock_count.get().wrapping_add(1));
                counted = true;
            } else {
                o.knock.set(0);
            }
        }
        if !counted && o.knock_count.get() != 0 {
            o.knock_count.set(o.knock_count.get().wrapping_add(1));
            if o.knock_count.get() >= 4 {
                o.knock_count.set(0);
            }
        }
    }
    // loc_40F703: the change of +0xBC.
    let now = o.effect_in[3].get();
    let before = o.last_effect_bc.get();
    o.last_effect_bc.set(now);
    let change = (now.wrapping_sub(before) as i32).clamp(-0x200, 0x200) >> 1;
    let edx = r.edx;
    set(r, 5, change as u32, edx)
}

/// The car's force feedback for the frame (the original's x87 arithmetic). The
/// converted version (`rc2k_core::input::feedback::update_car_effects`, plain floats)
/// waits for the float pass.
pub fn update_car_effects(view: u32, object: u32, device: u32) -> Result<(), Refused> {
    update_car_effects_with(view, object, device, crate::session::MACHINE.scratch.addr())
}

#[cfg(test)]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Kind, differential};

    /// `fld a; fsub b; fmul c; fistp`, the original's instructions over `v` (3 dwords) and
    /// the result at dword 3.
    unsafe fn ref_sub_mul(p: *mut u8) {
        unsafe {
            core::arch::asm!(
                "fld dword ptr [{v}]",
                "fsub dword ptr [{v} + 4]",
                "fmul dword ptr [{v} + 8]",
                "fistp dword ptr [{o}]",
                v = in(reg) p,
                o = in(reg) p.add(12),
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    /// The knock's instructions over `v` (6 dwords), the angle at dword 6, the knock at 7.
    unsafe fn ref_knock(p: *mut u8) {
        unsafe {
            core::arch::asm!(
                "fld dword ptr [{v}]",
                "fld dword ptr [{v} + 4]",
                "fpatan",
                "fst dword ptr [{o}]",
                "fabs",
                "fmul dword ptr [{v} + 8]",
                "fsubr dword ptr [{v} + 12]",
                "fmul dword ptr [{v} + 16]",
                "fmul dword ptr [{v} + 20]",
                "fistp dword ptr [{o} + 4]",
                v = in(reg) p,
                o = in(reg) p.add(24),
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    #[test]
    fn sub_mul_to_int_matches_the_assembly() {
        differential(
            "sub_mul_to_int",
            6000,
            209,
            [Kind::Exact; 4],
            |rng, m| {
                for d in &mut m[..3] {
                    *d = if rng.below(2) == 0 {
                        rng.moderate()
                    } else {
                        rng.fl()
                    };
                }
                m[3] = 0xAAAA_AAAA;
            },
            |p| unsafe { ref_sub_mul(p) },
            |p| unsafe {
                let m = p.cast::<u32>();
                m.add(3)
                    .write(sub_mul_to_int(m.read(), m.add(1).read(), m.add(2).read()) as u32);
            },
        );
    }

    #[test]
    fn knock_matches_the_assembly() {
        let mut kinds = [Kind::Exact; 8];
        kinds[6] = Kind::Ulp1;
        differential(
            "knock",
            6000,
            210,
            kinds,
            |rng, m| {
                for d in &mut m[..6] {
                    *d = rng.moderate();
                }
                m[6] = 0xAAAA_AAAA;
                m[7] = 0xAAAA_AAAA;
            },
            |p| unsafe { ref_knock(p) },
            |p| unsafe {
                let m = p.cast::<u32>();
                let v: Vec<u32> = (0..6).map(|k| m.add(k).read()).collect();
                let (angle, force) = knock(v[0], v[1], v[2], v[3], v[4], v[5]);
                m.add(6).write(angle);
                m.add(7).write(force as u32);
            },
        );
    }
}
