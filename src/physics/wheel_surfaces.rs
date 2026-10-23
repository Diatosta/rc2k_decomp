//! `physics::wheel_surfaces` (0x43D59A): the surface under each of the five ground points
//! (`unk_71ADC8[c]`, their faces) and the grip figures from it.
//!
//! For each point c: the face and its texture's record (`g_textureInfos`, by the face's
//! +1); the cell (the point's shares `dword_6D2B14[c]` / `dword_6D2A14[c]` x 65536,
//! `fistp`, bit 15, flipped with `dword_6D2914[c]` 1) turned by the face's +0 and +0x20
//! (`physics::flip_cell_index`) into `dword_614710[c]` / `dword_614724[c]`; the two
//! textures' surface kinds and two more bytes by those cells into +0x920 / +0x95C /
//! +0x984 [c] and +0x934 / +0x970 / +0x998 [c]; the blend +0x9AC [c] 0, or (unless the
//! second texture's kind (`dword_63D9D0`) is 4 or 5, its surface 0, or the face's +0x21
//! 0) the face's three blend bytes by the shares / 256; with bit 0x1000 of the face's +2
//! both surfaces 5 (wet, by the look) and the bytes 0x3C, 0. Then each figure blended
//! between the two surface records (`off_613978`, 0x30 each): grip, two more, the record's
//! scaled tables by `g_car` +0x2C and +0x280, into `flt_6142C0`, +0x948, `flt_614284`,
//! `flt_614298`, `flt_6142AC`, `flt_6146C0` .. [c].
//!
//! After the five: `flt_614268` = (the first four `flt_614284`) x `flt_613FEC`; the count
//! of points on hard surfaces (1, 2, 3, 0xC) into +0x258 (`dword_614150` ..); +0x60 the
//! least of the four grips x `flt_614C04`; `flt_614230` by +0x280 and +0x34; four
//! figures copied on.
//!
//! The x87 runs are the original's instructions (from IDA); the integer steps in Rust.
//!
//! In:  nothing
//! Out: the registers as the last steps leave them (eax the count, ebx, ecx 5, edx, esi,
//!      edi); no flags read
//!
//! Callers: `sub_43B635` (0x43B70F), `sub_43A1CA`.
//! Refuses no faces, a table index (+0x2C) past 5 (before anything) and a surface past 15
//! (what was written by then is put back).
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back) with test faces, under three precisions.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, EBX, ECX, EDI, EDX, ESI, Regs, game_addr, game_read, game_write, rva};
use crate::physics::flip_cell_index::flip_cell_index;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, restore, state_entry};
use std::cmp::Ordering;

fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

unsafe fn byte(a: u32) -> u32 {
    unsafe { (a as *const u8).read() as u32 }
}

macro_rules! x87_block {
    ($($line:literal),+ ; $($name:ident = $value:expr),*) => {
        core::arch::asm!(
            $($line),+,
            $($name = in(reg) $value),*,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
        )
    };
}

/// `fld v[c] x 65536; fistp` through `dword_62400C` (`v` the table's offset from the base).
unsafe fn share_bit(v: u32, c: u32) -> u32 {
    let b = game_addr(0) as u32;
    unsafe {
        x87_block!(
            "fld dword ptr [{t} + {c}*4]",
            "fmul dword ptr [{b} + 0x1C5D98]",
            "fistp dword ptr [{b} + 0x22400C]";
            b = b, t = b + v, c = c
        );
    }
    g(0x62400C)
}

/// Everything the function reads that the checks need, refused as said.
fn check() -> Result<(), Refused> {
    let faces = g(0x5CCCBC);
    if faces == 0 {
        return Err(BadInput("no course faces").into());
    }
    if g(0x71A004) > 5 {
        return Err(BadInput("a surface table index past 5").into());
    }
    Ok(())
}

/// The steps (a refusal may come after some are written: `wheel_surfaces` puts them
/// back).
unsafe fn steps() -> Result<Left, Refused> {
    check()?;
    let b = game_addr(0) as u32;
    let infos = game_addr(rva(0x979F30)) as u32;
    let surfaces = game_addr(rva(0x613978)) as u32;
    let mut r = Regs::default();
    for c in 0..5u32 {
        let cc = c as usize;
        r.ecx = c;
        r.ebx = g(0x71ADC8 + 4 * cc);
        let f = (r.ebx << 6).wrapping_add(g(0x5CCCBC));
        r.esi = f;
        let t1 = infos + (unsafe { byte(f + 1) } << 6);
        r.edi = t1;
        let mut edx = unsafe { share_bit(0x2D2B14, c) };
        let mut ebx = unsafe { share_bit(0x2D2A14, c) };
        edx = (edx >> 15) & 1;
        ebx = (ebx >> 15) & 1;
        if g(0x6D2914 + 4 * cc) == 1 {
            edx ^= 1;
            ebx ^= 1;
        }
        let (_, _, d1) = flip_cell_index(unsafe { byte(f) }, ebx, edx);
        p(0x614710 + 4 * cc, d1);
        let (a2, b2, d2) = flip_cell_index(unsafe { byte(f + 0x20) }, ebx, edx);
        r.eax = a2;
        r.ebx = b2;
        p(0x614724 + 4 * cc, d2);
        unsafe {
            let e = g(0x614710 + 4 * cc);
            p(0x71A8F8 + 4 * cc, byte(e + t1 + 0x10));
            p(0x71A934 + 4 * cc, byte(e + t1 + 0x18));
            p(0x71A95C + 4 * cc, byte(e + t1 + 0x14));
            let t2 = infos + (byte(f + 0x21) << 6);
            r.edi = t2;
            r.edx = g(0x614724 + 4 * cc);
            p(0x71A90C + 4 * cc, byte(r.edx + t2 + 0x10));
            p(0x71A948 + 4 * cc, byte(r.edx + t2 + 0x18));
            r.eax = byte(r.edx + t2 + 0x14);
            p(0x71A970 + 4 * cc, r.eax);
            p(0x71A984 + 4 * cc, 0);
            let kind_index = byte(t2 + 0x2C);
            r.eax = g(0x63D9D0 + 4 * kind_index as usize);
            if r.eax != 5 && r.eax != 4 && g(0x71A90C + 4 * cc) != 0 && byte(f + 0x21) != 0 {
                let (x, y, z) = if g(0x6D2914 + 4 * cc) == 1 {
                    (byte(f + 0x25), byte(f + 0x24), byte(f + 0x23))
                } else {
                    (byte(f + 0x22), byte(f + 0x23), byte(f + 0x24))
                };
                r.eax = x;
                p(0x624040, x);
                r.ebx = y.wrapping_sub(x);
                p(0x624044, r.ebx);
                r.edx = z.wrapping_sub(x);
                p(0x624048, r.edx);
                x87_block!(
                    "fild dword ptr [{b} + 0x224044]",
                    "fmul dword ptr [{b} + {c}*4 + 0x2D2A14]",
                    "fild dword ptr [{b} + 0x224048]",
                    "fmul dword ptr [{b} + {c}*4 + 0x2D2B14]",
                    "fild dword ptr [{b} + 0x224040]",
                    ".byte 0xDE, 0xC1",
                    ".byte 0xDE, 0xC1",
                    "fdiv dword ptr [{b} + 0x1C5E2C]",
                    "fstp dword ptr [{b} + {c}*4 + 0x31A984]";
                    b = b, c = c
                );
            }
            if ((f + 2) as *const u16).read_unaligned() & 0x1000 != 0 {
                r.ebx = 5;
                p(0x71A8F8 + 4 * cc, 5);
                p(0x71A90C + 4 * cc, 5);
                p(0x71A934 + 4 * cc, 0x3C);
                p(0x71A948 + 4 * cc, 0x3C);
                p(0x71A95C + 4 * cc, 0);
                p(0x71A970 + 4 * cc, 0);
            }
            let (s1, s2) = (g(0x71A8F8 + 4 * cc), g(0x71A90C + 4 * cc));
            if s1 > 15 || s2 > 15 {
                return Err(BadInput("a surface past 15").into());
            }
            r.ebx = s2.wrapping_mul(0x30);
            let esi = surfaces + s1 * 0x30;
            let edi = surfaces + s2 * 0x30;
            x87_block!(
                "fld dword ptr [{s} + 0x28]",
                "fld dword ptr [{d} + 0x28]",
                ".byte 0xD8, 0xE1",
                "fmul dword ptr [{b} + {c}*4 + 0x31A984]",
                ".byte 0xDE, 0xC1",
                "fstp dword ptr [{b} + {c}*4 + 0x2142C0]",
                "fld dword ptr [{s} + 0x2C]",
                "fld dword ptr [{d} + 0x2C]",
                ".byte 0xD8, 0xE1",
                "fmul dword ptr [{b} + {c}*4 + 0x31A984]",
                ".byte 0xDE, 0xC1",
                "fstp dword ptr [{b} + {c}*4 + 0x31A920]";
                b = b, c = c, s = esi, d = edi
            );
            let (ps, pd) = (*(esi as *const u32), *(edi as *const u32));
            let k = g(0x71A004);
            x87_block!(
                "fild dword ptr [{s}]",
                "fmul dword ptr [{b} + 0x1C5CFC]",
                "fstp dword ptr [{b} + 0x21411C]",
                "fild dword ptr [{d}]",
                "fmul dword ptr [{b} + 0x1C5CFC]",
                "fstp dword ptr [{b} + 0x214120]",
                "fild dword ptr [{s} + {k}*4 + 4]",
                "fild dword ptr [{s} + {k}*4 + 0x1C]",
                ".byte 0xD8, 0xE1",
                "fmul dword ptr [{b} + 0x31A258]",
                ".byte 0xDE, 0xC1",
                "fmul dword ptr [{b} + 0x1C5CFC]",
                "fstp dword ptr [{b} + 0x21411C]",
                "fild dword ptr [{d} + {k}*4 + 4]",
                "fild dword ptr [{d} + {k}*4 + 0x1C]",
                ".byte 0xD8, 0xE1",
                "fmul dword ptr [{b} + 0x31A258]",
                ".byte 0xDE, 0xC1",
                "fmul dword ptr [{b} + 0x1C5CFC]",
                "fstp dword ptr [{b} + 0x214120]";
                b = b, s = ps, d = pd, k = k
            );
            x87_block!(
                "fld dword ptr [{s} + 0x4]",
                "fmul dword ptr [{b} + 0x21411C]",
                "fld dword ptr [{d} + 0x4]",
                "fmul dword ptr [{b} + 0x214120]",
                ".byte 0xD8, 0xE1",
                "fmul dword ptr [{b} + {c}*4 + 0x31A984]",
                ".byte 0xDE, 0xC1",
                "fmul dword ptr [{b} + 0x214BBC]",
                "fstp dword ptr [{b} + {c}*4 + 0x214284]",
                "fld dword ptr [{s} + 0x8]",
                "fmul dword ptr [{b} + 0x21411C]",
                "fld dword ptr [{d} + 0x8]",
                "fmul dword ptr [{b} + 0x214120]",
                ".byte 0xD8, 0xE1",
                "fmul dword ptr [{b} + {c}*4 + 0x31A984]",
                ".byte 0xDE, 0xC1",
                "fmul dword ptr [{b} + 0x214BB4]",
                "fstp dword ptr [{b} + {c}*4 + 0x214298]",
                "fld dword ptr [{s} + 0xC]",
                "fmul dword ptr [{b} + 0x21411C]",
                "fld dword ptr [{d} + 0xC]",
                "fmul dword ptr [{b} + 0x214120]",
                ".byte 0xD8, 0xE1",
                "fmul dword ptr [{b} + {c}*4 + 0x31A984]",
                ".byte 0xDE, 0xC1",
                "fmul dword ptr [{b} + 0x214BB8]",
                "fstp dword ptr [{b} + {c}*4 + 0x2142AC]",
                "fld dword ptr [{s} + 0x20]",
                "fld dword ptr [{d} + 0x20]",
                ".byte 0xD8, 0xE1",
                "fmul dword ptr [{b} + {c}*4 + 0x31A984]",
                ".byte 0xDE, 0xC1",
                "fstp dword ptr [{b} + {c}*4 + 0x2146C0]",
                "fld dword ptr [{s} + 0x24]",
                "fld dword ptr [{d} + 0x24]",
                ".byte 0xD8, 0xE1",
                "fmul dword ptr [{b} + {c}*4 + 0x31A984]",
                ".byte 0xDE, 0xC1",
                "fstp dword ptr [{b} + {c}*4 + 0x2146D4]",
                "fld dword ptr [{s} + 0x14]",
                "fld dword ptr [{d} + 0x14]",
                ".byte 0xD8, 0xE1",
                "fmul dword ptr [{b} + {c}*4 + 0x31A984]",
                ".byte 0xDE, 0xC1",
                "fstp dword ptr [{b} + {c}*4 + 0x2146E8]",
                "fld dword ptr [{s} + 0x10]",
                "fld dword ptr [{d} + 0x10]",
                ".byte 0xD8, 0xE1",
                "fmul dword ptr [{b} + {c}*4 + 0x31A984]",
                ".byte 0xDE, 0xC1",
                "fstp dword ptr [{b} + {c}*4 + 0x2146FC]";
                b = b, c = c, s = esi, d = edi
            );
            r.esi = esi;
            r.edi = edi;
            r.ebx = k;
        }
    }
    r.ecx = 5;
    unsafe {
        x87_block!(
            "fld dword ptr [{b} + 0x214284]",
            "fadd dword ptr [{b} + 0x214288]",
            "fadd dword ptr [{b} + 0x21428C]",
            "fadd dword ptr [{b} + 0x214290]",
            "fmul dword ptr [{b} + 0x213FEC]",
            "fstp dword ptr [{b} + 0x214268]";
            b = b
        );
    }
    for c in 0..5usize {
        let hard = |s: u32| matches!(s, 1 | 2 | 3 | 0xC);
        let on = hard(g(0x71A8F8 + 4 * c)) || hard(g(0x71A90C + 4 * c));
        p(0x614150 + 4 * c, on as u32);
    }
    r.eax = g(0x614150)
        .wrapping_add(g(0x614154))
        .wrapping_add(g(0x614158))
        .wrapping_add(g(0x61415C));
    p(0x71A230, r.eax);
    let below = |a: u32, b: u32| {
        !matches!(
            f32::from_bits(a).partial_cmp(&f32::from_bits(b)),
            Some(Ordering::Greater | Ordering::Equal)
        )
    };
    let mut least = if below(g(0x6142C0), g(0x6142C4)) {
        g(0x6142C0)
    } else {
        g(0x6142C4)
    };
    p(0x71A038, crate::math::x87::loaded(least));
    for k in [0x6142C8usize, 0x6142CC, 0x6142D0] {
        least = if below(g(0x71A038), g(k)) {
            g(0x71A038)
        } else {
            g(k)
        };
        p(0x71A038, crate::math::x87::loaded(least));
    }
    unsafe {
        x87_block!(
            "fld dword ptr [{b} + 0x31A038]",
            "fmul dword ptr [{b} + 0x214C04]",
            "fstp dword ptr [{b} + 0x31A038]",
            "fld dword ptr [{b} + 0x213F5C]",
            "fsub dword ptr [{b} + 0x213F64]",
            "fmul dword ptr [{b} + 0x31A258]",
            "fadd dword ptr [{b} + 0x213F64]",
            "fstp dword ptr [{b} + 0x224040]",
            "fld dword ptr [{b} + 0x213F60]",
            "fsub dword ptr [{b} + 0x213F68]",
            "fmul dword ptr [{b} + 0x31A258]",
            "fadd dword ptr [{b} + 0x213F68]",
            "fstp dword ptr [{b} + 0x224044]",
            "fld dword ptr [{b} + 0x224044]",
            "fsub dword ptr [{b} + 0x224040]",
            "fild dword ptr [{b} + 0x31A00C]",
            "fdiv dword ptr [{b} + 0x1C5E2C]",
            ".byte 0xDE, 0xC9",
            "fadd dword ptr [{b} + 0x224040]",
            "fstp dword ptr [{b} + 0x214230]",
            "fld dword ptr [{b} + 0x2146D0]",
            "fstp dword ptr [{b} + 0x214760]",
            "fld dword ptr [{b} + 0x2146E4]",
            "fstp dword ptr [{b} + 0x214764]",
            "fld dword ptr [{b} + 0x2146F8]",
            "fstp dword ptr [{b} + 0x223DA8]",
            "fld dword ptr [{b} + 0x21470C]",
            "fstp dword ptr [{b} + 0x223DAC]";
            b = b
        );
    }
    Ok(Left::nothing()
        .with(EAX, r.eax)
        .with(EBX, r.ebx)
        .with(ECX, 5)
        .with(EDX, r.edx)
        .with(ESI, r.esi)
        .with(EDI, r.edi))
}

/// The Rust version: what was written is put back on a refusal.
pub unsafe fn wheel_surfaces() -> Result<Left, Refused> {
    let all = regions();
    let saved = capture(&all);
    let result = unsafe { steps() };
    if result.is_err() {
        restore(&all, &saved);
    }
    result
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { wheel_surfaces() }
}

fn regions() -> Vec<Region> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    vec![
        (a(0x62400C), 4),
        (a(0x624040), 12),
        (a(0x614710), 0x28),
        (a(0x71A8F8), 0xA0),
        (a(0x71A038), 4),
        (a(0x71A230), 4),
        (a(0x61411C), 8),
        (a(0x614150), 0x14),
        (a(0x614230), 4),
        (a(0x614268), 4),
        (a(0x614284), 0x50),
        (a(0x6146C0), 0x50),
        (a(0x614760), 8),
        (a(0x623DA8), 8),
    ]
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
