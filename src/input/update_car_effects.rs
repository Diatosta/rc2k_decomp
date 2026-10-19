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
//! See `input/effects.rs`. Three callers, all in `input::update_effects`. Its calls through
//! `part` (`set_effect`). The x87 work with the original's instructions (its rounding).
//! Refuses (before anything) a surface number past the table's 16. Shadow mode runs the
//! original alone (requests are sent). Self-test: a copy whose `g_soundScratch` is the
//! test's and whose calls go to the noting stand-ins, against the Rust version, on blocks
//! and objects of the test's: every kind, both branches of each test.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, hook_asm, rva};
use crate::input::devices::{at, put};
use crate::input::effects::{Part, SET_EFFECT, compare, dispatch_effects};
use crate::math::x87::{mul_to_int, x87};
use crate::sound::turn_parts::SCRATCH;
use crate::system::api_stand_ins::copy_with;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::update_car_effects", rva(0x40F347), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const TEN: usize = 0x5180DC; // flt_5180DC: 10.0
const ANGLE_SCALE: usize = 0x5180E0; // flt_5180E0: 162.97
const FULL: usize = 0x5180E4; // flt_5180E4: 256.0
const TENTH: usize = 0x5180EC; // flt_5180EC: 0.1
/// dword_51809C: a surface's effect kind, 16 entries.
const KINDS: usize = 0x51809C;
const KIND_COUNT: u32 = 16;
/// g_controllers: 0x50 bytes a controller.
const CONTROLLERS: usize = 0x4F78E4;

fn k(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
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

/// The Rust version, the angle written to `scratch` (an address).
pub fn update_car_effects_with(r: &mut Regs, scratch: u32, part: Part) -> Result<(), Refused> {
    let (view, object, device) = (r.ebx, r.edi, r.esi);
    let surface = at(object, 0x294);
    if surface >= KIND_COUNT {
        return Err(BadInput("a surface number past the effect kinds' 16").into());
    }
    r.eax = at(view, 0x248);
    put(object, 0xAC, r.eax);
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
    r.eax = 0;
    for o in [0xC4, 0xC8, 0xCC, 0xD0] {
        put(object, o, 0);
    }
    r.ebx = at(device, 0)
        .wrapping_mul(0x50)
        .wrapping_add(game_addr(rva(CONTROLLERS)) as u32);
    r.eax = surface;
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
    let mut set = |r: &mut Regs, eax: u32, ecx: u32, edx: u32| {
        r.eax = eax;
        r.ebx = 3;
        r.ecx = ecx;
        r.edx = edx;
        part(SET_EFFECT, r)
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

/// The Rust version.
pub fn update_car_effects(r: &mut Regs, part: Part) -> Result<(), Refused> {
    update_car_effects_with(r, game_addr(rva(SCRATCH)) as u32, part)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_effects(&HOOK, r, call_original, update_car_effects);
}

// ---- self-test ----

/// The offsets of the calls (to `set_effect`).
const CALLS: [usize; 19] = [
    0x119, 0x151, 0x170, 0x187, 0x1A6, 0x1C0, 0x1DC, 0x1F3, 0x212, 0x229, 0x258, 0x271, 0x288,
    0x2BA, 0x2CE, 0x306, 0x330, 0x383, 0x3F6,
];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut scratch = 0u32;
    let scratch_at = &raw mut scratch as u32;
    // mov eax, [ebx+248h]
    let entry = [0x8B, 0x83, 0x48, 0x02, 0x00, 0x00];
    let copy = match copy_with(
        0x40F347,
        0x3FC,
        &entry,
        &CALLS,
        &[SET_EFFECT],
        &[],
        &[(SCRATCH, 4, scratch_at)],
        1,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let mut view = vec![0u32; 0x250 / 4];
    let mut object = vec![0u32; 0x300 / 4];
    let mut device = [0u32; 4];
    let (vb, ob, db) = (
        view.as_mut_ptr() as u32,
        object.as_mut_ptr() as u32,
        device.as_mut_ptr() as u32,
    );
    let all = [(vb, 0x250usize), (ob, 0x300), (scratch_at, 4)];
    let f = |v: f32| v.to_bits();
    let mut seed = 0x1357_9BDFu32;
    let mut next = || {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        seed
    };
    for case in 0..160u32 {
        let surface = case % 14; // 13 .. 15 read the table's zeros
        let speed =
            [0u32, 3, 5, 6, 0xFFFF_FFFB, 0x150, 0xFFFF_FE00, 0x8000_0000][(case / 14) as usize % 8];
        let r1 = next();
        let r2 = next();
        let r3 = next();
        let fill = || unsafe {
            let w = |base: u32, o: u32, v: u32| ((base + o) as *mut u32).write_volatile(v);
            for o in (0..0x250).step_by(4) {
                w(vb, o, 0xA5A5_0000 | o);
            }
            for o in (0..0x300).step_by(4) {
                w(ob, o, 0x5A5A_0000 | o);
            }
            w(vb, 0x248, (r1 >> 3) & 1);
            w(vb, 0x1A4, f(((r1 >> 8) % 400) as f32 / 37.0 - 5.0));
            w(vb, 0x1AC, f(((r2 >> 8) % 400) as f32 / 29.0 - 7.0));
            w(vb, 0x1B8, f(((r3 >> 8) % 400) as f32 / 31.0 - 6.0));
            w(
                ob,
                0xC4,
                if r1 & 0x10 != 0 {
                    0x8000_0000
                } else {
                    f(((r2 >> 4) % 200) as f32 / 9.0 - 11.0)
                },
            );
            w(ob, 0xCC, f(((r3 >> 4) % 200) as f32 / 7.0 - 14.0));
            w(
                ob,
                0xD0,
                if r1 & 0x20 != 0 {
                    0
                } else {
                    f(((r1 >> 12) % 100) as f32 / 3.0)
                },
            );
            w(ob, 0x14, r2 >> 18);
            w(ob, 0x284, speed);
            w(ob, 0x294, surface);
            w(ob, 0x278, (r2 >> 2) & 1);
            w(ob, 0xDC, (r3 >> 1) % 5);
            w(ob, 0xD8, r3 >> 20);
            w(ob, 0x27C, (r1 >> 21).wrapping_sub(0x200));
            w(db, 0, r2 & 0xF);
            (scratch_at as *mut u32).write_volatile(0xAAAA_AAAA);
        };
        let entry = Regs {
            ebx: vb,
            edi: ob,
            esi: db,
            ..Regs::SENTINEL
        };
        compare(
            &mut t,
            &format!("case {case} (surface {surface}, speed {speed:#x})"),
            &copy,
            entry,
            &fill,
            &all,
            &|r, part| update_car_effects_with(r, scratch_at, part),
            false,
        );
    }
    // A surface past the table is refused.
    object[0x294 / 4] = 16;
    let mut r = Regs {
        ebx: vb,
        edi: object.as_mut_ptr() as u32,
        esi: db,
        ..Regs::SENTINEL
    };
    t.check(
        update_car_effects_with(&mut r, scratch_at, &mut crate::sound::turn_parts::test_part)
            .is_err(),
        || "surface 16 was not refused".into(),
    );
    crate::sound::turn_parts::take_called();
    let _ = (&mut view, &mut device);
    t.finish()
}
