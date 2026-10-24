//! `physics::contact_force` (0x45F17E): the force of one contact into the record that
//! `dword_614B40` names. From the contact's point (`dword_6D3090` .., car frame), the
//! car's speed (`dword_6D30A8` ..), its turn speed (`dword_6D30B4` ..) and the surface's
//! normal (`qword_6D3078` ..): the point's speed (turn x point + speed) into
//! `flt_6D30CC` ..; its depth along the normal, held at least 1 - |normal y| + `flt_5C5CFC`,
//! into `flt_6D31DC`; the push the turn would give (point x normal turned by the inverse
//! matrix, times `flt_613FA8` .., turned back, x point) along the normal into `flt_6D31D8`
//! and the share `flt_6D1DF4` / (it + 1) into `flt_6D31E0`; the normal push
//! (`qword_6D3130` ..), the speed along the normal and across it (`qword_6D30E4` ..,
//! `dword_6D30F0` ..; their lengths `flt_6D3108`, `dword_6D310C`), with `dword_614148`
//! a part of the normal x `flt_6D1DF8` x it / `dword_61414C` added; the slide against the
//! motion (`dword_6D3140` ..) held to `flt_6D3100` long; their sum (`qword_6D3150` ..)
//! turned by the inverse matrix and / `flt_61388C` into the record's force, the point
//! turned into its point (+0xC ..). The turns are `physics::turn_by_inverse` /
//! `turn_by_car_matrix`, the products `scene::cross_product`, `physics::dot_product`,
//! `multiply_parts`, `scale_vector`, `vector_length`.
//!
//! The flow and the calls are Rust; each straight x87 run is a block of the original's
//! instructions (generated from IDA, `physics/flow.rs`).
//!
//! In:  nothing (`dword_614B40` the record)
//! Out: eax `dword_6D3098` (the point's z turned), ebx and esi 0x6D3090, ecx 0x6D31F4,
//!      edx 0x6D31E4, edi the record; no flags read (callers call on)
//!
//! Callers: `sub_45D8FB`, `sub_45DA14`, `sub_45DC3E`, `sub_45E3BF`,
//! `physics::crash_contact`, `physics::ground_contact`.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): the contact's figures and both matrices by lot, the extra part on and off, a
//! test record, under three precisions.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{EAX, EBX, ECX, EDI, EDX, ESI};
use crate::math::x87_run::{compare_80, neg_rounded};
use crate::physics::dot_product::dot_product;
use crate::physics::flow::{a, copy3, fill3, g, hold_low, negate, p, x87_run};
use crate::physics::multiply_parts::multiply_parts;
use crate::physics::scale_vector::scale_vector;
use crate::physics::turn_by_car_matrix::turn_by_car_matrix;
use crate::physics::turn_by_inverse::turn_by_inverse;
use crate::physics::vector_length::vector_length;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::view_matrix::cross_product;
use crate::screens::harness::Left;

/// `dword_614B40`: the record the force goes to.
pub const RECORD: usize = 0x614B40;

/// `scene::cross_product` with the original's ebx, ecx, edx.
fn cross(ebx: usize, ecx: usize, edx: usize) {
    unsafe { cross_product(a(ebx) as usize, a(ecx) as usize, a(edx) as usize) };
}

/// The Rust version: the registers as the original leaves them.
pub unsafe fn contact_force() -> Left {
    cross(0x6D30B4, 0x6D3090, 0x6D30C0);
    x87_run!(
        "fld dword ptr [{rc} + 0x2D30C0]",
        "fadd dword ptr [{rc} + 0x2D30A8]",
        "fstp dword ptr [{rc} + 0x2D30CC]",
        "fld dword ptr [{rc} + 0x2D30C4]",
        "fadd dword ptr [{rc} + 0x2D30AC]",
        "fstp dword ptr [{rc} + 0x2D30D0]",
        "fld dword ptr [{rc} + 0x2D30C8]",
        "fadd dword ptr [{rc} + 0x2D30B0]",
        "fstp dword ptr [{rc} + 0x2D30D4]";
        rc = crate::race::store::asm_base()
    );
    p(
        0x6D31DC,
        neg_rounded(unsafe { dot_product(a(0x6D30CC), a(0x6D3078)) }),
    );
    x87_run!(
        ".byte 0xD9, 0xE8", // fld1
        "fld dword ptr [{rc} + 0x2D307C]",
        ".byte 0xD9, 0xE1", // fabs
        ".byte 0xDE, 0xE9", // fsubp st(1), st
        "fadd dword ptr [{rc} + 0x1C5CFC]",
        "fstp dword ptr [{rc} + 0x22400C]";
        rc = crate::race::store::asm_base()
    );
    hold_low(0x6D31DC, 0x62400C);
    // 0x45F222: the push the turn would give.
    cross(0x6D3090, 0x6D3078, 0x6D31E4);
    unsafe { turn_by_inverse(a(0x6D31E4)) };
    unsafe { multiply_parts(a(0x6D31E4), a(0x613FA8), a(0x6D31F4)) };
    unsafe { turn_by_car_matrix(a(0x6D31F4)) };
    cross(0x6D3090, 0x6D31F4, 0x6D31E4);
    p(
        0x6D31D8,
        neg_rounded(unsafe { dot_product(a(0x6D31E4), a(0x6D3078)) }),
    );
    x87_run!(
        "fld dword ptr [{rc} + 0x2D1DF4]",
        "fld dword ptr [{rc} + 0x2D31D8]",
        "fadd dword ptr [{rc} + 0x1C5D38]",
        ".byte 0xDE, 0xF9", // fdivp st(1), st
        "fstp dword ptr [{rc} + 0x2D31E0]",
        "fld dword ptr [{rc} + 0x2D31DC]",
        "fmul dword ptr [{rc} + 0x2D31E0]",
        "fstp dword ptr [{rc} + 0x2D3204]";
        rc = crate::race::store::asm_base()
    );
    // 0x45F2C2: the normal push, the speed along the normal and across it.
    copy3(0x6D3130, 0x6D3078);
    let factor = extended(g(0x6D3204));
    unsafe { scale_vector(a(0x6D3130), factor) };
    p(0x62400C, rounded(factor));
    let along = unsafe { dot_product(a(0x6D30CC), a(0x6D3078)) };
    copy3(0x6D30E4, 0x6D3078);
    unsafe { scale_vector(a(0x6D30E4), along) };
    p(0x62400C, rounded(along));
    copy3(0x6D30F0, 0x6D30CC);
    x87_run!(
        "fld dword ptr [{rc} + 0x2D30F0]",
        "fsub dword ptr [{rc} + 0x2D30E4]",
        "fstp dword ptr [{rc} + 0x2D30F0]",
        "fld dword ptr [{rc} + 0x2D30F4]",
        "fsub dword ptr [{rc} + 0x2D30E8]",
        "fstp dword ptr [{rc} + 0x2D30F4]",
        "fld dword ptr [{rc} + 0x2D30F8]",
        "fsub dword ptr [{rc} + 0x2D30EC]",
        "fstp dword ptr [{rc} + 0x2D30F8]";
        rc = crate::race::store::asm_base()
    );
    p(0x6D3108, rounded(unsafe { vector_length(a(0x6D30E4)) }));
    p(0x6D310C, rounded(unsafe { vector_length(a(0x6D30F0)) }));
    fill3(0x6D3160, 0);
    if g(0x614148) != 0 {
        fill3(0x6D3130, 0);
        copy3(0x6D3160, 0x6D3078);
        let mut part = [0u8; 10];
        x87_run!(
            "fld dword ptr [{rc} + 0x2D1DF8]",
            "fild dword ptr [{rc} + 0x214148]",
            ".byte 0xDE, 0xC9", // fmulp st(1), st
            "fidiv dword ptr [{rc} + 0x21414C]",
            "fstp tbyte ptr [{o}]";
            o = part.as_mut_ptr(), rc = crate::race::store::asm_base()
        );
        unsafe { scale_vector(a(0x6D3160), part) };
        p(0x62400C, rounded(part));
    }
    x87_run!(
        "fld dword ptr [{rc} + 0x2D3130]",
        "fadd dword ptr [{rc} + 0x2D3160]",
        "fstp dword ptr [{rc} + 0x2D3130]",
        "fld dword ptr [{rc} + 0x2D3134]",
        "fadd dword ptr [{rc} + 0x2D3164]",
        "fstp dword ptr [{rc} + 0x2D3134]",
        "fld dword ptr [{rc} + 0x2D3138]",
        "fadd dword ptr [{rc} + 0x2D3168]",
        "fstp dword ptr [{rc} + 0x2D3138]";
        rc = crate::race::store::asm_base()
    );
    copy3(0x6D3150, 0x6D3130);
    p(0x6D30FC, crate::math::x87::loaded(g(0x6D3100)));
    // 0x45F48D: the slide against the motion, held to `flt_6D30FC` long.
    copy3(0x6D3140, 0x6D30F0);
    negate(0x6D3140);
    negate(0x6D3144);
    negate(0x6D3148);
    if compare_80(unsafe { vector_length(a(0x6D3140)) }, g(0x5C5D90)) & 0x4000 != 0 {
        fill3(0x6D3140, 0);
    }
    let mut status = 0u16;
    x87_run!(
        "fld dword ptr [{rc} + 0x2D3140]",
        ".byte 0xD8, 0xC8", // fmul st, st
        "fld dword ptr [{rc} + 0x2D3144]",
        ".byte 0xD8, 0xC8",
        "fld dword ptr [{rc} + 0x2D3148]",
        ".byte 0xD8, 0xC8",
        ".byte 0xDE, 0xC1", // faddp st(1), st
        ".byte 0xDE, 0xC1",
        ".byte 0xD9, 0xFA", // fsqrt
        "fst dword ptr [{rc} + 0x22400C]",
        "fcomp dword ptr [{rc} + 0x2D30FC]",
        "fnstsw word ptr [{s}]";
        s = &raw mut status, rc = crate::race::store::asm_base()
    );
    if status & 0x4100 == 0 {
        x87_run!(
            "fld dword ptr [{rc} + 0x2D30FC]",
            "fdiv dword ptr [{rc} + 0x22400C]",
            "fld dword ptr [{rc} + 0x2D3140]",
            ".byte 0xD8, 0xC9", // fmul st, st(1)
            "fstp dword ptr [{rc} + 0x2D3140]",
            "fld dword ptr [{rc} + 0x2D3144]",
            ".byte 0xD8, 0xC9",
            "fstp dword ptr [{rc} + 0x2D3144]",
            "fld dword ptr [{rc} + 0x2D3148]",
            ".byte 0xD8, 0xC9",
            "fstp dword ptr [{rc} + 0x2D3148]",
            "fstp dword ptr [{rc} + 0x22400C]";
            rc = crate::race::store::asm_base()
        );
    }
    // 0x45F57E: the sum into the record.
    x87_run!(
        "fld dword ptr [{rc} + 0x2D3140]",
        "fadd dword ptr [{rc} + 0x2D3130]",
        "fstp dword ptr [{rc} + 0x2D3150]",
        "fld dword ptr [{rc} + 0x2D3144]",
        "fadd dword ptr [{rc} + 0x2D3134]",
        "fstp dword ptr [{rc} + 0x2D3154]",
        "fld dword ptr [{rc} + 0x2D3148]",
        "fadd dword ptr [{rc} + 0x2D3138]",
        "fstp dword ptr [{rc} + 0x2D3158]";
        rc = crate::race::store::asm_base()
    );
    let edi = g(RECORD);
    unsafe { turn_by_inverse(a(0x6D3150)) };
    x87_run!(
        "fld dword ptr [{rc} + 0x2D3150]",
        "fdiv dword ptr [{rc} + 0x21388C]",
        "fstp dword ptr [{d}]",
        "fld dword ptr [{rc} + 0x2D3154]",
        "fdiv dword ptr [{rc} + 0x21388C]",
        "fstp dword ptr [{d} + 0x4]",
        "fld dword ptr [{rc} + 0x2D3158]",
        "fdiv dword ptr [{rc} + 0x21388C]",
        "fstp dword ptr [{d} + 0x8]";
        d = edi, rc = crate::race::store::asm_base()
    );
    unsafe { turn_by_inverse(a(0x6D3090)) };
    for k in 0..3 {
        unsafe { ((edi + 0xC + 4 * k) as *mut u32).write_unaligned(g(0x6D3090 + 4 * k as usize)) };
    }
    Left::nothing()
        .with(EAX, g(0x6D3098))
        .with(EBX, a(0x6D3090))
        .with(ECX, a(0x6D31F4))
        .with(EDX, a(0x6D31E4))
        .with(ESI, a(0x6D3090))
        .with(EDI, edi)
}
