//! `physics::drive_wheels_step` (0x43DFB1): the driven wheels' step: the slip shares from
//! the car-frame wheel speeds (+0xB8, +0xD8; held 0 ..= 1 and turned into a ±1 share), the
//! clutch (+0x250) and the gear deciding the drive force (`flt_614750` 0 in neutral),
//! `physics::cap_drive_speed`; the drive speeds stepped by the drive force less the
//! wheels' drag and held toward the car's speed, `physics::revs_from_wheels` (twice, on
//! the two paths); the slips `flt_614234` / `614238`; `physics::cap_reverse_inputs`,
//! `physics::add_drive_forces`, `physics::ease_low_speed_forces`; and the rest of the
//! wheels' forces into the force records.
//!
//! The flow, the compares, the integer steps and the calls are Rust; each straight x87
//! run is a small block of the original's instructions (generated from IDA,
//! `physics/flow.rs`).
//!
//! In:  nothing
//! Out: eax and ebx as the instructions and calls leave them; the rest kept; no flags
//!      read (the caller calls on)
//!
//! Caller: `sub_43B635`, every physics step.
//! Refuses (before anything) a gear outside -1 ..= 7 (`revs_from_wheels` would; the gear
//! is not written here).
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): the work car's figures by lot, gears, the clutch, under three precisions.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, EBX, Regs};
use crate::math::x87;
use crate::math::x87_run::{abs_compare, abs_of, neg_rounded};
use crate::physics::flow::{a, base, c0, ease_zero, g, hold_high, hold_low, p, x87_run};
use crate::physics::gearing::{GearPlaces, revs_from_wheels_at};
use crate::scene::matrix3::extended;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// `fld v; fabs; fsub one; fchs; fstp to` (0x43DFB1, 0x43E00B): 1 less |v|.
fn one_less_abs(v: usize, to: usize) {
    x87_run!(
        "fld dword ptr [{v}]",
        ".byte 0xD9, 0xE1", // fabs
        "fsub dword ptr [{b} + 0x1C5D38]",
        ".byte 0xD9, 0xE0", // fchs
        "fstp dword ptr [{t}]";
        b = base(), v = a(v), t = a(to)
    );
}

/// `fld b; fabs; fld a; fabs; fcompp; fnstsw`: C0, |a| below |b| or unordered.
fn abs_below(x: u32, y: u32) -> bool {
    let (v, mut status) = ([x, y], 0u16);
    unsafe {
        core::arch::asm!(
            "fld dword ptr [{v} + 4]",
            ".byte 0xD9, 0xE1",
            "fld dword ptr [{v}]",
            ".byte 0xD9, 0xE1",
            ".byte 0xDE, 0xD9", // fcompp
            "fnstsw word ptr [{s}]",
            v = in(reg) v.as_ptr(),
            s = in(reg) &raw mut status,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
    status & 0x100 != 0
}

/// `fld from; fstp to`.
fn store_loaded(to: usize, from: usize) {
    p(to, x87::loaded(g(from)));
}

/// 0x43E5CF .. 0x43E643 (and the second wheel's at 0x43E68D): the slip `s` (at `slip`)
/// signed by the slip ratio's sign, the drive speed (`drive`) less it x (`dword_614BA8`
/// - 1), and the two force records' parts at `f1`, `f2`.
fn settle_slip(drive: usize, slip: usize, ratio: usize, f1: usize, f2: usize) {
    x87_run!(
        "fld dword ptr [{d}]",
        "fsub dword ptr [{b} + 0x31A184]",
        "fdiv dword ptr [{b} + 0x214BA8]",
        "fstp dword ptr [{r}]";
        b = base(), d = a(drive), r = a(ratio)
    );
    let s = abs_of(g(slip));
    let signed = if c0(g(ratio), g(0x5C5D90)) {
        neg_rounded(extended(s))
    } else {
        s
    };
    p(slip, signed);
    x87_run!(
        "fld dword ptr [{d}]",
        "fld dword ptr [{b} + 0x214BA8]",
        "fsub dword ptr [{b} + 0x1C5D38]",
        "fmul dword ptr [{s}]",
        ".byte 0xDE, 0xE9", // fsubp st(1), st
        "fstp dword ptr [{d}]",
        "fld dword ptr [{s}]",
        "fdiv dword ptr [{b} + 0x21388C]",
        "fmul dword ptr [{b} + 0x1C5E40]",
        "fst dword ptr [{f1}]",
        "fstp dword ptr [{f2}]";
        b = base(), d = a(drive), s = a(slip), f1 = a(f1), f2 = a(f2)
    );
}

/// The slip's first limit (0x43E58B / 0x43E649): |drive - speed| / `dword_614BA8` into
/// `ratio`, and `slip` the lesser of it and `force` (by `fcompp`).
fn slip_limit(drive: usize, ratio: usize, force: usize, slip: usize) {
    x87_run!(
        "fld dword ptr [{d}]",
        "fsub dword ptr [{b} + 0x31A184]",
        "fdiv dword ptr [{b} + 0x214BA8]",
        ".byte 0xD9, 0xE1", // fabs
        "fstp dword ptr [{r}]";
        b = base(), d = a(drive), r = a(ratio)
    );
    let pick = if c0(g(ratio), g(force)) { ratio } else { force };
    store_loaded(slip, pick);
}

/// The steps; `ebx` as the original leaves it.
fn steps(mut eax: u32, mut ebx: u32) -> Result<[u32; 2], Refused> {
    let gear = g(0x71A1AC);
    one_less_abs(0x71A090, 0x6140B0);
    hold_low(0x6140B0, 0x5C5DD0);
    hold_high(0x6140B0, 0x5C5D38);
    one_less_abs(0x71A0B0, 0x6140B4);
    hold_low(0x6140B4, 0x5C5DD0);
    hold_high(0x6140B4, 0x5C5D38);
    x87_run!(
        "fld dword ptr [{b} + 0x2140B4]",
        ".byte 0xD8, 0xC0", // fadd st, st
        "fsub dword ptr [{b} + 0x1C5D38]",
        "fstp dword ptr [{b} + 0x2140B8]";
        b = base()
    );
    hold_low(0x6140B8, 0x5C5D90);
    hold_high(0x6140B8, 0x5C5D38);
    if g(0x71A228) == 0 {
        if gear == 0 {
            p(0x614750, 0);
        }
        unsafe { crate::physics::cap_drive_speed::cap_drive_speed() };
        p(0x614134, 0);
        ebx = 0;
        if g(0x71A614) != 0 {
            ebx = 0x14;
            p(0x614134, 1);
        }
        x87_run!(
            "fld dword ptr [{b} + {e} + 0x214298]",
            "fmul dword ptr [{b} + 0x214774]",
            "fld dword ptr [{b} + {e} + 0x21429C]",
            "fmul dword ptr [{b} + 0x214794]",
            ".byte 0xDE, 0xC1", // faddp st(1), st
            ".byte 0xD9, 0xE0", // fchs
            "fmul dword ptr [{b} + 0x21388C]",
            "fmul dword ptr [{b} + 0x214170]",
            "fmul dword ptr [{b} + 0x31A3D8]",
            "fstp dword ptr [{b} + 0x214248]",
            "fld dword ptr [{b} + {e} + 0x2142A0]",
            "fmul dword ptr [{b} + 0x2147B4]",
            "fld dword ptr [{b} + {e} + 0x2142A4]",
            "fmul dword ptr [{b} + 0x2147D4]",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xE0",
            "fmul dword ptr [{b} + 0x21388C]",
            "fmul dword ptr [{b} + 0x214170]",
            "fmul dword ptr [{b} + 0x31A3DC]",
            "fstp dword ptr [{b} + 0x21424C]",
            "fld dword ptr [{b} + 0x214284]",
            "fadd dword ptr [{b} + 0x214288]",
            "fadd dword ptr [{b} + 0x21428C]",
            "fadd dword ptr [{b} + 0x214290]",
            "fmul dword ptr [{b} + 0x214774]",
            ".byte 0xD9, 0xE0",
            "fmul dword ptr [{b} + 0x21388C]",
            "fmul dword ptr [{b} + 0x214170]",
            "fstp dword ptr [{b} + 0x21423C]";
            b = base(), e = ebx
        );
        eax = g(0x71A610).wrapping_add(0x100);
        p(0x62400C, eax);
        x87_run!(
            "fild dword ptr [{b} + 0x22400C]",
            "fdiv dword ptr [{b} + 0x1C5E2C]",
            "fld dword ptr [{b} + 0x31A17C]",
            "fmul dword ptr [{b} + 0x214268]",
            ".byte 0xD9, 0xE1", // fabs
            ".byte 0xDE, 0xC9", // fmulp st(1), st
            "fmul dword ptr [{b} + 0x214170]",
            "fstp dword ptr [{b} + 0x214240]",
            "fld dword ptr [{b} + 0x21423C]",
            "fmul dword ptr [{b} + 0x213EC4]",
            "fstp dword ptr [{b} + 0x22400C]";
            b = base()
        );
        hold_high(0x614240, 0x62400C);
        let pick = if abs_below(g(0x614240), g(0x61423C)) {
            0x61423C
        } else {
            0x614240
        };
        store_loaded(0x614244, pick);
        let kind = g(0x71A018);
        if gear != 0 && kind == 1 {
            x87_run!(
                "fld dword ptr [{b} + 0x214750]",
                "fmul dword ptr [{b} + 0x1C5E40]",
                "fmul dword ptr [{b} + 0x21388C]",
                "fmul dword ptr [{b} + 0x214BA8]",
                "fadd dword ptr [{b} + 0x31A190]",
                "fstp dword ptr [{b} + 0x31A190]",
                "fld dword ptr [{b} + 0x214750]",
                "fmul dword ptr [{b} + 0x1C5E40]",
                "fmul dword ptr [{b} + 0x21388C]",
                "fmul dword ptr [{b} + 0x214BA8]",
                "fadd dword ptr [{b} + 0x31A194]",
                "fstp dword ptr [{b} + 0x31A194]";
                b = base()
            );
        }
        if kind == 0 {
            x87_run!(
                "fld dword ptr [{b} + 0x214750]",
                "fmul dword ptr [{b} + 0x21388C]",
                "fmul dword ptr [{b} + 0x214BA8]",
                "fadd dword ptr [{b} + 0x31A190]",
                "fstp dword ptr [{b} + 0x31A190]";
                b = base()
            );
        }
        ebx = revs(ebx)?;
        // 0x43E2C2: the wheels' drag on each drive speed.
        x87_run!(
            ".byte 0xD9, 0xE8", // fld1
            "fsub dword ptr [{b} + 0x214BF0]",
            "fmul dword ptr [{b} + 0x214BB0]",
            "fild dword ptr [{b} + 0x31A614]",
            ".byte 0xDE, 0xC9", // fmulp st(1), st
            "fdiv dword ptr [{b} + 0x1C5E2C]",
            "fmul dword ptr [{b} + 0x21388C]",
            "fmul dword ptr [{b} + 0x31A3D0]",
            "fmul dword ptr [{b} + 0x214BA8]",
            "fstp dword ptr [{b} + 0x21422C]";
            b = base()
        );
        ease_zero(0x71A190, 0x61422C);
        x87_run!(
            "fld dword ptr [{b} + 0x214BF0]",
            "fmul dword ptr [{b} + 0x214BB0]",
            "fild dword ptr [{b} + 0x31A614]",
            ".byte 0xDE, 0xC9",
            "fdiv dword ptr [{b} + 0x1C5E2C]",
            "fmul dword ptr [{b} + 0x21388C]",
            "fmul dword ptr [{b} + 0x31A3D4]",
            "fmul dword ptr [{b} + 0x214BA8]",
            "fstp dword ptr [{b} + 0x21422C]";
            b = base()
        );
        ease_zero(0x71A194, 0x61422C);
        x87_run!(
            "fld dword ptr [{b} + 0x214BAC]",
            "fild dword ptr [{b} + 0x31A618]",
            ".byte 0xDE, 0xC9",
            "fdiv dword ptr [{b} + 0x1C5E2C]",
            "fmul dword ptr [{b} + 0x21388C]",
            "fmul dword ptr [{b} + 0x214BA8]",
            "fmul dword ptr [{b} + 0x214170]",
            "fstp dword ptr [{b} + 0x21422C]";
            b = base()
        );
        ease_zero(0x71A194, 0x61422C);
        if g(0x71A618) != 0 {
            let (v, mut status) = ([g(0x71A190), g(0x71A194), g(0x5C5D38)], 0u16);
            unsafe {
                core::arch::asm!(
                    "fld dword ptr [{v}]",
                    "fadd dword ptr [{v} + 4]",
                    ".byte 0xD9, 0xE1", // fabs
                    "fcomp dword ptr [{v} + 8]",
                    "fnstsw word ptr [{s}]",
                    v = in(reg) v.as_ptr(),
                    s = in(reg) &raw mut status,
                    out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                    out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                    options(nostack),
                );
            }
            if status & 0x100 != 0 {
                p(0x71A190, 0);
                p(0x71A194, 0);
            }
        }
        ebx = revs(ebx)?;
        x87_run!(
            "fld dword ptr [{b} + 0x31A194]",
            "fsub dword ptr [{b} + 0x31A184]",
            ".byte 0xD9, 0xE1",
            "fstp dword ptr [{b} + 0x214238]",
            "fld dword ptr [{b} + 0x31A190]",
            "fsub dword ptr [{b} + 0x31A184]",
            ".byte 0xD9, 0xE1",
            "fstp dword ptr [{b} + 0x214234]";
            b = base()
        );
        unsafe {
            crate::physics::cap_reverse_inputs::cap_reverse_inputs();
            crate::physics::add_drive_forces::add_drive_forces();
            crate::physics::ease_low_speed_forces::ease_low_speed_forces();
        }
        x87_run!(
            "fld dword ptr [{b} + 0x214244]",
            "fmul dword ptr [{b} + 0x2140B0]",
            "fstp dword ptr [{b} + 0x214244]",
            "fld dword ptr [{b} + 0x214248]",
            "fmul dword ptr [{b} + 0x2140B4]",
            "fstp dword ptr [{b} + 0x214248]",
            "fld dword ptr [{b} + 0x21424C]",
            "fmul dword ptr [{b} + 0x2140B4]",
            "fstp dword ptr [{b} + 0x21424C]";
            b = base()
        );
        if g(0x71A25C) != 0 {
            p(0x614248, x87::mul(g(0x614248), g(0x5C5E40)));
            p(0x61424C, x87::mul(g(0x61424C), g(0x5C5E40)));
        }
        slip_limit(0x71A190, 0x614C1C, 0x614248, 0x614258);
        settle_slip(0x71A190, 0x614258, 0x614C1C, 0x614958, 0x614978);
        slip_limit(0x71A194, 0x614C20, 0x61424C, 0x61425C);
        settle_slip(0x71A194, 0x61425C, 0x614C20, 0x614998, 0x6149B8);
        // 0x43E707: the car-frame side speed eased by the side grip.
        p(0x62400C, g(0x71A17C));
        ease_zero(0x62400C, 0x614244);
        x87_run!(
            "fld dword ptr [{b} + 0x22400C]",
            "fsub dword ptr [{b} + 0x31A17C]",
            "fst dword ptr [{b} + 0x214250]",
            "fdiv dword ptr [{b} + 0x21388C]",
            "fmul dword ptr [{b} + 0x1C5DA4]",
            "fst dword ptr [{b} + 0x2148D0]",
            "fst dword ptr [{b} + 0x2148F0]",
            "fst dword ptr [{b} + 0x214910]",
            "fstp dword ptr [{b} + 0x214930]";
            b = base()
        );
        store_loaded(0x614264, 0x61423C);
        store_loaded(0x6141D4, 0x614BFC);
        let status = abs_compare(g(0x614250), g(0x614264));
        if status & 0x4100 == 0 {
            x87_run!(
                "fld dword ptr [{b} + 0x214250]",
                ".byte 0xD9, 0xE1", // fabs
                "fsub dword ptr [{b} + 0x214264]",
                "fmul dword ptr [{b} + 0x213ED0]",
                "fadd dword ptr [{b} + 0x214264]",
                "fstp dword ptr [{b} + 0x214254]",
                "fld dword ptr [{b} + 0x214254]",
                "fdiv dword ptr [{b} + 0x214250]",
                ".byte 0xD9, 0xE1",
                "fmul dword ptr [{b} + 0x214BFC]",
                "fstp dword ptr [{b} + 0x2141D4]";
                b = base()
            );
        }
        x87_run!(
            "fld dword ptr [{b} + 0x214BFC]",
            "fmul dword ptr [{b} + 0x213ED4]",
            "fstp dword ptr [{b} + 0x2141D8]";
            b = base()
        );
        p(0x6148B0, 0);
        p(0x6148B8, 0);
        // 0x43E855: the rolling drag against the motion.
        let mut status = 0u16;
        x87_run!(
            "fld dword ptr [{b} + 0x31A17C]",
            ".byte 0xD8, 0xC8", // fmul st, st
            "fld dword ptr [{b} + 0x31A184]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1", // faddp st(1), st
            ".byte 0xD9, 0xFA", // fsqrt
            "fst dword ptr [{b} + 0x22400C]",
            "fcomp dword ptr [{b} + 0x214764]",
            "fnstsw word ptr [{s}]";
            b = base(), s = &raw mut status
        );
        if status & 0x4100 == 0 {
            x87_run!(
                "fld dword ptr [{b} + 0x214760]",
                "fdiv dword ptr [{b} + 0x22400C]",
                "fmul dword ptr [{b} + 0x31A17C]",
                ".byte 0xD9, 0xE0", // fchs
                "fstp dword ptr [{b} + 0x2148B0]",
                "fld dword ptr [{b} + 0x214760]",
                "fdiv dword ptr [{b} + 0x22400C]",
                "fmul dword ptr [{b} + 0x31A184]",
                ".byte 0xD9, 0xE0",
                "fstp dword ptr [{b} + 0x2148B8]";
                b = base()
            );
        }
        if g(0x62389C) != 1 && (g(0x719FEC).wrapping_sub(0x320) as i32) < 0 {
            p(0x71A190, 0);
            p(0x71A194, 0);
        }
        x87_run!(
            "fld dword ptr [{b} + 0x2149D0]",
            "fmul dword ptr [{b} + 0x1C5E40]",
            "fadd dword ptr [{b} + 0x2148D0]",
            "fstp dword ptr [{b} + 0x2148D0]",
            "fld dword ptr [{b} + 0x2149D0]",
            "fmul dword ptr [{b} + 0x1C5E40]",
            "fadd dword ptr [{b} + 0x2148F0]",
            "fstp dword ptr [{b} + 0x2148F0]",
            "fld dword ptr [{b} + 0x2149F0]",
            "fmul dword ptr [{b} + 0x1C5E40]",
            "fadd dword ptr [{b} + 0x214910]",
            "fstp dword ptr [{b} + 0x214910]",
            "fld dword ptr [{b} + 0x2149F0]",
            "fmul dword ptr [{b} + 0x1C5E40]",
            "fadd dword ptr [{b} + 0x214930]",
            "fstp dword ptr [{b} + 0x214930]";
            b = base()
        );
        p(0x6149D0, 0);
        p(0x6149F0, 0);
    }
    // 0x43E957: the wheels' slip figures.
    x87_run!(
        "fld dword ptr [{b} + 0x31A190]",
        "fsub dword ptr [{b} + 0x31A184]",
        "fst dword ptr [{b} + 0x31A158]",
        ".byte 0xD9, 0xE1", // fabs
        "fsub dword ptr [{b} + 0x213EDC]",
        "fstp dword ptr [{b} + 0x22401C]";
        b = base()
    );
    hold_low(0x62401C, 0x5C5D90);
    x87_run!(
        "fld dword ptr [{b} + 0x31A17C]",
        ".byte 0xD9, 0xE1",
        "fsub dword ptr [{b} + 0x213ED8]",
        "fstp dword ptr [{b} + 0x224020]";
        b = base()
    );
    hold_low(0x624020, 0x5C5D90);
    x87_run!(
        "fld dword ptr [{b} + 0x22401C]",
        ".byte 0xD8, 0xC8",
        "fld dword ptr [{b} + 0x224020]",
        ".byte 0xD8, 0xC8",
        ".byte 0xDE, 0xC1",
        ".byte 0xD9, 0xFA",
        "fstp dword ptr [{b} + 0x31A260]",
        "fld dword ptr [{b} + 0x31A17C]",
        ".byte 0xD9, 0xE1",
        "fstp dword ptr [{b} + 0x224020]",
        "fld dword ptr [{b} + 0x31A190]",
        "fsub dword ptr [{b} + 0x31A184]",
        ".byte 0xD9, 0xE1",
        "fstp dword ptr [{b} + 0x22401C]",
        "fld dword ptr [{b} + 0x22401C]",
        ".byte 0xD8, 0xC8",
        "fld dword ptr [{b} + 0x224020]",
        ".byte 0xD8, 0xC8",
        ".byte 0xDE, 0xC1",
        ".byte 0xD9, 0xFA",
        "fstp dword ptr [{b} + 0x31AF18]",
        "fld dword ptr [{b} + 0x31A194]",
        "fsub dword ptr [{b} + 0x31A184]",
        ".byte 0xD9, 0xE1",
        "fstp dword ptr [{b} + 0x22401C]",
        "fld dword ptr [{b} + 0x22401C]",
        ".byte 0xD8, 0xC8",
        "fld dword ptr [{b} + 0x224020]",
        ".byte 0xD8, 0xC8",
        ".byte 0xDE, 0xC1",
        ".byte 0xD9, 0xFA",
        "fstp dword ptr [{b} + 0x31AF1C]";
        b = base()
    );
    let _ = &mut eax;
    Ok([eax, ebx])
}

/// `revs_from_wheels` (the gear checked before): ebx as it leaves it.
fn revs(ebx: u32) -> Result<u32, Refused> {
    let mut r = Regs {
        ebx,
        ..Regs::default()
    };
    unsafe { revs_from_wheels_at(&GearPlaces::game()) }?.apply(&mut r);
    Ok(r.ebx)
}

/// The Rust version: eax, ebx in, as left out.
pub unsafe fn drive_wheels_step(eax: u32, ebx: u32) -> Result<[u32; 2], Refused> {
    let gear = g(0x71A1AC);
    if gear != 0 && !(-1..=7).contains(&(gear as i32)) {
        return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
    }
    steps(eax, ebx)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let [eax, ebx] = unsafe { drive_wheels_step(r.eax, r.ebx) }?;
    Ok(Left::nothing().with(EAX, eax).with(EBX, ebx))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
