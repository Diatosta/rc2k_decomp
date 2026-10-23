//! `physics::passed_split_point` (0x44165E): whether the work car has passed the next
//! split's point. Nothing (CF clear) with `dword_64BB18` set. The car's x / z into
//! `dword_639824` / `dword_63982C`; the split's checkpoint (`dword_601030[+0x90]`), its
//! face (`g_checkpointFaces`; past `g_courseFaceCount` the original logs
//! "[car0.currsegment]" .. and raises fatal error 0x80601843) and that face's first point;
//! the distance (`fild` of the two gaps, squared, added, `fsqrt`, / 65536) into
//! `flt_614D04`; within `flt_614C70`: the angle from the point to the car (`fpatan`, /
//! pi, x 0.5, x 65536, `fistp`, low 16 bits, shifted up 16) into `dword_614CFC`, the
//! checkpoint's (`dword_9ED808`) into `dword_614D00`; CF when the angle is not below the
//! checkpoint's less a quarter turn (by the sign).
//!
//! The x87 runs (with the integer steps between) are the original's instructions.
//!
//! In:  nothing
//! Out: CF; eax, ebx, ecx, esi as left; no other flag read (the caller `jnb`s)
//!
//! Caller: `physics::track_checkpoints`.
//! Raises fatal error 0x80601843 where the original does. Refuses (before anything) a
//! split past 0x40, a checkpoint past 0x1418, no faces or points.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back) with test faces and points: near and far, angles about the quarter turn.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, FatalError, Refused};
use crate::framework::{EAX, EBX, ECX, ESI, Regs, game_addr, game_read, game_write, rva};
use crate::screens::harness::Left;
use rc2k_core::physics::progress;
use std::cmp::Ordering;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

/// 0x4416EC .. 0x441727: the distance to the point at `point`.
unsafe fn distance(point: u32) {
    let b = game_addr(0) as u32;
    unsafe {
        core::arch::asm!(
            "mov eax, [{s}]",
            "sub eax, [{b} + 0x239824]",
            "mov [{b} + 0x22400C], eax",
            "fild dword ptr [{b} + 0x22400C]",
            ".byte 0xD8, 0xC8", // fmul st, st
            "mov eax, [{s} + 8]",
            "sub eax, [{b} + 0x23982C]",
            "mov [{b} + 0x22400C], eax",
            "fild dword ptr [{b} + 0x22400C]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1", // faddp st(1), st
            ".byte 0xD9, 0xFA", // fsqrt
            "fmul dword ptr [{b} + 0x1C5DA0]",
            "fstp dword ptr [{b} + 0x214D04]",
            b = in(reg) b,
            s = in(reg) point,
            out("eax") _,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
}

/// 0x441770 .. 0x4417AD: the angle from the point at `point` to the car, into
/// `dword_62400C` (`fistp`, then the low 16 bits).
unsafe fn angle(point: u32) {
    let b = game_addr(0) as u32;
    unsafe {
        core::arch::asm!(
            "mov eax, [{b} + 0x239824]",
            "sub eax, [{s}]",
            "mov [{b} + 0x22400C], eax",
            "fild dword ptr [{b} + 0x22400C]",
            "mov eax, [{b} + 0x23982C]",
            "sub eax, [{s} + 8]",
            "mov [{b} + 0x22400C], eax",
            "fild dword ptr [{b} + 0x22400C]",
            ".byte 0xD9, 0xF3", // fpatan
            ".byte 0xD9, 0xEB", // fldpi
            ".byte 0xDE, 0xF9", // fdivp st(1), st
            "fmul dword ptr [{b} + 0x1C5E40]",
            "fmul dword ptr [{b} + 0x1C5D98]",
            "fistp dword ptr [{b} + 0x22400C]",
            "and dword ptr [{b} + 0x22400C], 0xFFFF",
            b = in(reg) b,
            s = in(reg) point,
            out("eax") _,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        );
    }
}

/// The split's checkpoint, its face and the face's first point (esi), refused or fatal
/// as said.
fn point() -> Result<(u32, u32, u32), Refused> {
    let split = g(0x71A068);
    let cp = progress::check_split_point(split, || g(0x601030 + 4 * split as usize))?;
    let face = g(0x9E3738 + 4 * cp as usize);
    if !progress::face_known(face, g(0x5CCD1C)) {
        return Err(FatalError {
            code: progress::SEGMENT_FATAL,
            message: "\u{1}",
        }
        .into());
    }
    if g(0x5CCCBC) == 0 || g(0x5CCCC0) == 0 {
        return Err(BadInput("no course faces or points").into());
    }
    let f = (face << 6).wrapping_add(g(0x5CCCBC));
    let index = unsafe { ((f + 4) as *const u32).read_unaligned() };
    Ok((cp, index, index.wrapping_mul(12).wrapping_add(g(0x5CCCC0))))
}

/// The Rust version.
// Converted: `rc2k_core::physics::progress::{check_split_point, face_known, passed_point}`
// (the distance and the angle stay x87).
pub unsafe fn passed_split_point(entry: &Regs) -> Result<Left, Refused> {
    let _ = entry;
    if g(0x64BB18) != 0 {
        return Ok(Left::carry(false));
    }
    let (cp, index, pt) = point()?;
    p(0x639824, g(0x71A10C));
    p(0x63982C, g(0x71A114));
    unsafe { distance(pt) };
    let within = !matches!(
        f32::from_bits(g(0x614D04)).partial_cmp(&f32::from_bits(g(0x614C70))),
        Some(Ordering::Greater | Ordering::Equal)
    );
    if !within {
        let z = unsafe { ((pt + 8) as *const u32).read_unaligned() };
        return Ok(Left::carry(false)
            .with(EAX, z.wrapping_sub(g(0x63982C)))
            .with(EBX, index)
            .with(ECX, cp)
            .with(ESI, pt));
    }
    unsafe { angle(pt) };
    let eax = g(0x62400C) << 16;
    p(0x614CFC, eax);
    let heading = g(0x9ED808 + 4 * cp as usize);
    p(0x614D00, heading);
    let (carry, ebx) = progress::passed_point(eax, heading);
    Ok(Left::carry(carry)
        .with(EAX, eax)
        .with(EBX, ebx)
        .with(ECX, cp)
        .with(ESI, pt))
}
