//! `physics::bump_cars` (0x45F7B1): the work car and another (`dword_6D3BB8`, its number
//! `dword_6D3CD4`) bumped. The corner met (`flt_6D3BC8` ..) into `dword_6D3090` ..; a
//! note of the meeting at `dword_6D3CA0` .. (the current car, the other, the corner, the
//! point; two parts 0, as `flt_5C5D90`); contact record 6's force 0, `unk_71A20C` = 1,
//! the other's +0x234 = 1; the speed between them (+0x198 ..) into `flt_6D3CD8` .. and
//! its length into `flt_6D3CD0`; the speeds shared out (each part a x `flt_6D3BB0` + b x
//! `flt_6D3BAC` and the other way); the places' difference / 65536 x `flt_6D3B88` into
//! `flt_6D3B9C` .., times `flt_5C5D40` (`physics::scale_vector`), added to the work car's
//! speed and taken from the other's; both speeds turned into the cars' frames (+0x9CC,
//! `scene::rotate_point`, into +0x1A4 ..); `physics::place_sound_e1_other`.
//!
//! The flow and the calls are Rust; the straight x87 runs are blocks of the original's
//! instructions (generated from IDA, `physics/flow.rs`).
//!
//! In:  edi the other car's block (as `dword_6D3BB8`)
//! Out: esi `dword_6D3BB8`, edi `g_car`; the rest kept; no flags read (the caller
//!      returns)
//!
//! Caller: `physics::corner_hits_car` (its tail).
//! Refuses a null block (before anything) and what `place_sound_e1_other` refuses (what
//! was written put back).
//! Self-test: `physics/car_collisions.rs` (`test_bump`): the original and the Rust version
//! on the game's variables (saved and put back) with test car blocks, under three
//! precisions.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{EDI, ESI};
use crate::math::x87::loaded;
use crate::physics::flow::{a, fill3, g, p, x87_run};
use crate::physics::scale_vector::scale_vector;
use crate::physics::vector_length::vector_length;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;
use crate::screens::harness::Left;

/// `g_car`: the work car's block.
pub const CAR: usize = 0x719FD8;
/// `dword_6D3BB8`: the other car's block.
pub const OTHER: usize = 0x6D3BB8;

/// The block's speed (+0x198 ..) turned by its matrix (+0x9CC) into +0x1A4 ...
fn turn_speed(block: u32) {
    let rd = |o: u32| extended(unsafe { ((block + o) as *const u32).read_unaligned() });
    let turned = unsafe { rotate_point(block + 0x9CC, [rd(0x198), rd(0x19C), rd(0x1A0)]) };
    for (k, e) in turned.iter().enumerate() {
        unsafe { ((block + 0x1A4 + 4 * k as u32) as *mut u32).write_unaligned(rounded(*e)) };
    }
}

fn steps(edi: u32) -> Result<Left, Refused> {
    for k in 0..3 {
        p(0x6D3090 + 4 * k, loaded(g(0x6D3BC8 + 4 * k)));
    }
    let zero = g(0x5C5D90);
    let note = [
        g(0x71B3F8),
        g(0x6D3CD4),
        g(0x6D3BD4),
        zero,
        g(0x6D3BDC),
        g(0x6D3090),
        zero,
        g(0x6D3098),
    ];
    for (k, v) in note.iter().enumerate() {
        p(0x6D3CA0 + 4 * k, *v);
    }
    fill3(0x614B10, 0);
    p(0x71A20C, 1);
    unsafe { ((edi + 0x234) as *mut u32).write_unaligned(1) };
    x87_run!(
        "fld dword ptr [{rc} + 0x31A170]",
        "fsub dword ptr [{d} + 0x198]",
        "fstp dword ptr [{rc} + 0x2D3CD8]",
        "fld dword ptr [{rc} + 0x31A174]",
        "fsub dword ptr [{d} + 0x19C]",
        "fstp dword ptr [{rc} + 0x2D3CDC]",
        "fld dword ptr [{rc} + 0x31A178]",
        "fsub dword ptr [{d} + 0x1A0]",
        "fstp dword ptr [{rc} + 0x2D3CE0]";
        d = edi, rc = crate::race::store::asm_base()
    );
    p(0x6D3CD0, rounded(unsafe { vector_length(a(0x6D3CD8)) }));
    let esi = g(OTHER);
    x87_run!(
        "fld dword ptr [{rc} + 0x31A170]",
        "fmul dword ptr [{rc} + 0x2D3BB0]",
        "fld dword ptr [{s} + 0x198]",
        "fmul dword ptr [{rc} + 0x2D3BAC]",
        ".byte 0xDE, 0xC1", // faddp st(1), st
        "fld dword ptr [{rc} + 0x31A170]",
        "fmul dword ptr [{rc} + 0x2D3BAC]",
        "fld dword ptr [{s} + 0x198]",
        "fmul dword ptr [{rc} + 0x2D3BB0]",
        ".byte 0xDE, 0xC1",
        "fstp dword ptr [{rc} + 0x31A170]",
        "fstp dword ptr [{s} + 0x198]",
        "fld dword ptr [{rc} + 0x31A174]",
        "fmul dword ptr [{rc} + 0x2D3BB0]",
        "fld dword ptr [{s} + 0x19C]",
        "fmul dword ptr [{rc} + 0x2D3BAC]",
        ".byte 0xDE, 0xC1",
        "fld dword ptr [{rc} + 0x31A174]",
        "fmul dword ptr [{rc} + 0x2D3BAC]",
        "fld dword ptr [{s} + 0x19C]",
        "fmul dword ptr [{rc} + 0x2D3BB0]",
        ".byte 0xDE, 0xC1",
        "fstp dword ptr [{rc} + 0x31A174]",
        "fstp dword ptr [{s} + 0x19C]",
        "fld dword ptr [{rc} + 0x31A178]",
        "fmul dword ptr [{rc} + 0x2D3BB0]",
        "fld dword ptr [{s} + 0x1A0]",
        "fmul dword ptr [{rc} + 0x2D3BAC]",
        ".byte 0xDE, 0xC1",
        "fld dword ptr [{rc} + 0x31A178]",
        "fmul dword ptr [{rc} + 0x2D3BAC]",
        "fld dword ptr [{s} + 0x1A0]",
        "fmul dword ptr [{rc} + 0x2D3BB0]",
        ".byte 0xDE, 0xC1",
        "fstp dword ptr [{rc} + 0x31A178]",
        "fstp dword ptr [{s} + 0x1A0]";
        s = esi, rc = crate::race::store::asm_base()
    );
    // 0x45F98C: pushed apart by the places' difference.
    let other = g(OTHER);
    x87_run!(
        "fild dword ptr [{rc} + 0x31A10C]",
        "fisub dword ptr [{d} + 0x134]",
        "fdiv dword ptr [{rc} + 0x1C5D98]",
        "fmul dword ptr [{rc} + 0x2D3B88]",
        "fstp dword ptr [{rc} + 0x2D3B9C]",
        "fild dword ptr [{rc} + 0x31A110]",
        "fisub dword ptr [{d} + 0x138]",
        "fdiv dword ptr [{rc} + 0x1C5D98]",
        "fmul dword ptr [{rc} + 0x2D3B88]",
        "fstp dword ptr [{rc} + 0x2D3BA0]",
        "fild dword ptr [{rc} + 0x31A114]",
        "fisub dword ptr [{d} + 0x13C]",
        "fdiv dword ptr [{rc} + 0x1C5D98]",
        "fmul dword ptr [{rc} + 0x2D3B88]",
        "fstp dword ptr [{rc} + 0x2D3BA4]";
        d = other, rc = crate::race::store::asm_base()
    );
    let factor = extended(g(0x5C5D40));
    unsafe { scale_vector(a(0x6D3B9C), factor) };
    p(0x62400C, rounded(factor));
    x87_run!(
        "fld dword ptr [{rc} + 0x31A170]",
        "fadd dword ptr [{rc} + 0x2D3B9C]",
        "fstp dword ptr [{rc} + 0x31A170]",
        "fld dword ptr [{rc} + 0x31A174]",
        "fadd dword ptr [{rc} + 0x2D3BA0]",
        "fstp dword ptr [{rc} + 0x31A174]",
        "fld dword ptr [{rc} + 0x31A178]",
        "fadd dword ptr [{rc} + 0x2D3BA4]",
        "fstp dword ptr [{rc} + 0x31A178]",
        "fld dword ptr [{d} + 0x198]",
        "fsub dword ptr [{rc} + 0x2D3B9C]",
        "fstp dword ptr [{d} + 0x198]",
        "fld dword ptr [{d} + 0x19C]",
        "fsub dword ptr [{rc} + 0x2D3BA0]",
        "fstp dword ptr [{d} + 0x19C]",
        "fld dword ptr [{d} + 0x1A0]",
        "fsub dword ptr [{rc} + 0x2D3BA4]",
        "fstp dword ptr [{d} + 0x1A0]";
        d = other, rc = crate::race::store::asm_base()
    );
    turn_speed(esi);
    turn_speed(a(CAR));
    unsafe { crate::physics::place_sound_e1_other::place_sound_e1_other() }?;
    Ok(Left::nothing().with(ESI, esi).with(EDI, a(CAR)))
}

/// The Rust version: `edi` the other car's block.
pub unsafe fn bump_cars(edi: u32) -> Result<Left, Refused> {
    if edi == 0 || g(OTHER) == 0 {
        return Err(BadInput("a null car block").into());
    }
    steps(edi)
}
