//! `marks::draw_snow` (0x4C2E36): the snowflakes, `dword_8D21A4` of them (also into
//! `dword_8D21A0`; `dword_8DA428` the first, 0, then plus the count; `dword_8D219C` the
//! drawn). For each flake k: its place from `g_marksRandomsA` .. [k], y plus the fall
//! `dword_8D2194`, x and z moved by `g_marksCurve` at (the fall's bits 11 .. 19) plus
//! `g_marksRemaindersA` / `B`[k], plus `dword_8D2214` .., wrapped to 0x100000 and centred;
//! turned by the matrix at `dword_8D2220` .. plus `dword_8D2208` .. into `flt_8D22D4` ..
//! (the depth also whole, `dword_8D21F4`); unless that is at or below 0x10000: when
//! `dword_76523C` is set the colour `g_marksFadeColours`[the square of the light's dot
//! with the way from `flt_8DB430` .. times `flt_76518C`, 0 when the dot is below 0]; the
//! projected square (side `flt_8DB548` over the depth), depth times `flt_8D21F8`,
//! texture u `g_marksSpreads`[k] (+ `flt_8DB54C`), v `dword_8DB550` / `8DB554`;
//! `marks::clip_quad`, and unless all is outside `scene::draw_fan`, the drawn count plus 1
//! and `g_polygonVertexCount` 4.
//!
//! In:  nothing (`dword_8D21A4`)
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `marks::run_snow` (0x4C2A8F). The x87 blocks are plain Rust on the x87 model (`math::x87_float`), in the original's order,
//! over the game's variables. Refuses, before anything (a dry pass, its scratch put back),
//! more than 0x800 flakes, null marks settings with a flake, a curve index past the 4095
//! entries and a fade index past the 17.

use crate::errors::{BadInput, Refused};
use crate::framework::rva;
use crate::marks::clip_quad::{clip_quad, draw_fan};
use crate::marks::store::{Slot, game_addr, game_read, game_write};
use crate::service::harness::{Region, capture, restore};

const COUNT_RVA: Slot = Slot::at(0x8D21A4); // dword_8D21A4
const COUNT_COPY_RVA: Slot = Slot::at(0x8D21A0); // dword_8D21A0
const FIRST_RVA: Slot = Slot::at(0x8DA428); // dword_8DA428
const DRAWN_RVA: Slot = Slot::at(0x8D219C); // dword_8D219C
const MOST: i32 = 0x800;
const FALL_RVA: Slot = Slot::at(0x8D2194); // dword_8D2194
const RANDOMS: [Slot; 3] = [Slot::at(0x8C6000), Slot::at(0x8C8000), Slot::at(0x8CA000)];
const REMAINDERS: [Slot; 2] = [Slot::at(0x8CC000), Slot::at(0x8CE000)];
const CURVE_RVA: Slot = Slot::at(0x8D2310); // g_marksCurve: 4095 entries
const CURVE: u32 = 4095;
const SHIFT_RVA: Slot = Slot::at(0x8D2214); // dword_8D2214 .. dword_8D221C
const DEPTH_RVA: Slot = Slot::at(0x8D21F4); // dword_8D21F4
const FADING_RVA: Slot = Slot::at(0x76523C); // dword_76523C
const FADE_INDEX_RVA: Slot = Slot::at(0x8DB460); // dword_8DB460
const DOT_RVA: Slot = Slot::at(0x8DB4C8); // flt_8DB4C8
const FADES_RVA: Slot = Slot::at(0x8DB464); // g_marksFadeColours: 17
const FADES: u32 = 17;
const V_RVA: [Slot; 2] = [Slot::at(0x8DB550), Slot::at(0x8DB554)];
const VERTICES_RVA: usize = rva(0x9605A0); // g_polygonVertices
const VERTEX_COUNT_RVA: usize = rva(0x6240F8); // g_polygonVertexCount
/// What the x87 blocks write: flt_8D22D4 .. flt_8D22DC, dword_8D21F4, flt_8DB448 ..
/// dword_8DB460, flt_8DB4C8.
const SCRATCH: [(Slot, usize); 4] = [
    (Slot::at(0x8D22D4), 12),
    (Slot::at(0x8D21F4), 4),
    (Slot::at(0x8DB448), 0x1C),
    (Slot::at(0x8DB4C8), 4),
];

unsafe fn table(r: Slot, k: u32) -> u32 {
    unsafe { game_read(r + 4 * k as usize) }
}

/// The original's 0x4C2EF0 .. 0x4C2F79: the place (ebx, eax, ecx as `fild`ed) turned.
unsafe fn turn(v: &[u32; 3]) {
    unsafe { turn_x87(crate::marks::store::base() as usize, v.as_ptr() as usize) }
}

/// The original's 0x4C2F9C .. 0x4C3038: the light's dot and its fade index.
unsafe fn fade() {
    unsafe { fade_x87(crate::marks::store::base() as usize) }
}

/// The original's 0x4C305B .. 0x4C30EE: the projected square, its depth and u.
unsafe fn square(i: u32) {
    unsafe {
        square_x87(
            crate::race::store::asm_base() as usize,
            crate::race::store::buffers_asm_base() as usize,
            crate::marks::store::base() as usize,
            i as usize,
        )
    }
}

/// Flake k's whole-number place and the two curve indices: (eax, ebx, ecx, x index, z
/// index, the shift's edx, ebp, edi).
unsafe fn place(k: u32) -> ([u32; 3], [u32; 2], [u32; 3]) {
    unsafe {
        let mut eax = table(RANDOMS[0], k);
        let mut ebx = table(RANDOMS[1], k).wrapping_add(game_read::<u32>(FALL_RVA));
        let mut ecx = table(RANDOMS[2], k);
        let f = (ebx & 0xF_FFFF) >> 11;
        let ix = f.wrapping_add(table(REMAINDERS[0], k));
        let iz = f.wrapping_add(table(REMAINDERS[1], k));
        let shift: [u32; 3] = game_read(SHIFT_RVA);
        let curve = |i: u32| if i < CURVE { table(CURVE_RVA, i) } else { 0 };
        eax = eax.wrapping_add(curve(ix)).wrapping_add(shift[0]);
        ecx = ecx.wrapping_add(curve(iz)).wrapping_add(shift[2]);
        ebx = ebx.wrapping_add(shift[1]);
        let wrap = |v: u32| (v & 0xF_FFFF).wrapping_sub(0x7_FFFF);
        ([wrap(eax), wrap(ebx), wrap(ecx)], [ix, iz], shift)
    }
}

/// Why it refuses, if it does: a dry pass, its scratch put back.
unsafe fn refusal() -> Option<&'static str> {
    unsafe {
        let n: i32 = game_read(COUNT_RVA);
        if n > MOST {
            return Some("more than 0x800 snowflakes");
        }
        if n > 0 && game_read::<u32>(crate::marks::setup::SETTINGS_RVA) == 0 {
            return Some("no marks settings");
        }
        let scratch: Vec<Region> = SCRATCH
            .iter()
            .map(|&(r, l)| (game_addr(r) as u32, l))
            .collect();
        let saved = capture(&scratch);
        let mut why = None;
        for k in 0..n.max(0) as u32 {
            let ([eax, ebx, ecx], idx, _) = place(k);
            if idx.iter().any(|&i| i >= CURVE) {
                why = Some("a snow curve index past the table");
                break;
            }
            turn(&[ebx, eax, ecx]);
            if game_read::<i32>(DEPTH_RVA) > 0x1_0000 && game_read::<u32>(FADING_RVA) != 0 {
                fade();
                let i: u32 = game_read(FADE_INDEX_RVA);
                if game_read::<i32>(DOT_RVA) >= 0 && i >= FADES {
                    why = Some("a snow fade index past the table");
                    break;
                }
            }
        }
        restore(&scratch, &saved);
        why
    }
}

/// The Rust version (see the header).
pub unsafe fn draw_snow() -> Result<(), Refused> {
    if let Some(why) = unsafe { refusal() } {
        return Err(BadInput(why).into());
    }
    unsafe {
        let n: i32 = game_read(COUNT_RVA);
        game_write(COUNT_COPY_RVA, n);
        game_write(FIRST_RVA, 0u32);
        game_write(DRAWN_RVA, 0u32);
        game_write(FIRST_RVA, n);
        for k in 0..n.max(0) as u32 {
            let ([eax, ebx, ecx], _, _) = place(k);
            turn(&[ebx, eax, ecx]);
            if game_read::<i32>(DEPTH_RVA) > 0x1_0000 {
                if game_read::<u32>(FADING_RVA) != 0 {
                    fade();
                    let mut index: u32 = game_read(FADE_INDEX_RVA);
                    if game_read::<i32>(DOT_RVA) < 0 {
                        index = 0;
                    }
                    let colour = table(FADES_RVA, index);
                    for n in 0..4 {
                        game_write(VERTICES_RVA + 0x20 * n + 0x10, colour);
                    }
                }
                square(4 * k);
                let [top, bottom]: [u32; 2] = [game_read(V_RVA[0]), game_read(V_RVA[1])];
                for n in 0..4 {
                    game_write(
                        VERTICES_RVA + 0x20 * n + 0x1C,
                        if n < 2 { top } else { bottom },
                    );
                }
                if !clip_quad(game_addr(VERTICES_RVA) as u32)? {
                    draw_fan();
                    game_write(DRAWN_RVA, game_read::<u32>(DRAWN_RVA).wrapping_add(1));
                    game_write(VERTEX_COUNT_RVA, 4u32);
                }
            }
        }
    }
    Ok(())
}

use crate::marks::plain::{ld, ldi, st, sti};

/// `turn`'s x87 block as plain Rust on the x87 model (34 instructions), same order and memory places.
unsafe fn turn_x87(mk: usize, v: usize) {
    unsafe {
        let t0 = ldi(v, 0);
        let t1 = ldi(v, 0x4);
        let t2 = ldi(v, 0x8);
        let t3 = t1 * ld(mk, 0x16D2E0);
        let t4 = t0 * ld(mk, 0x16D2EC);
        let t5 = t2 * ld(mk, 0x16D2F8);
        let t6 = t4 + t5;
        let t7 = t3 + t6;
        let t8 = t7 + ldi(mk, 0x16D2C8);
        st(mk, 0x16D394, t8);
        let t9 = t1 * ld(mk, 0x16D2E4);
        let t10 = t0 * ld(mk, 0x16D2F0);
        let t11 = t2 * ld(mk, 0x16D2FC);
        let t12 = t10 + t11;
        let t13 = t9 + t12;
        let t14 = t13 + ldi(mk, 0x16D2CC);
        st(mk, 0x16D398, t14);
        let t15 = t1 * ld(mk, 0x16D2E8);
        let t16 = t0 * ld(mk, 0x16D2F4);
        let t17 = t2 * ld(mk, 0x16D300);
        let t18 = t16 + t17;
        let t19 = t15 + t18;
        let t20 = t19 + ldi(mk, 0x16D2D0);
        sti(mk, 0x16D2B4, t20);
        st(mk, 0x16D39C, t20);
    }
}

/// `fade`'s x87 block as plain Rust on the x87 model (34 instructions), same order and memory places.
unsafe fn fade_x87(mk: usize) {
    unsafe {
        let t0 = ld(mk, 0x16D394);
        let t1 = t0 - ld(mk, 0x1764F0);
        st(mk, 0x176508, t1);
        let t2 = t1 * t1;
        let t3 = ld(mk, 0x16D398);
        let t4 = t3 - ld(mk, 0x1764F4);
        st(mk, 0x17650C, t4);
        let t5 = t4 * t4;
        let t6 = ld(mk, 0x16D39C);
        let t7 = t6 - ld(mk, 0x1764F8);
        st(mk, 0x176510, t7);
        let t8 = t7 * t7;
        let t9 = t5 + t8;
        let t10 = t2 + t9;
        let t11 = t10.sqrt();
        let t12 = ld(mk, 0x22C) / t11;
        let t13 = t12 * ld(mk, 0x176508);
        st(mk, 0x176514, t13);
        let t14 = t13 * ld(mk, 0x1764FC);
        let t15 = t12 * ld(mk, 0x17650C);
        st(mk, 0x176518, t15);
        let t16 = t15 * ld(mk, 0x176500);
        let t17 = t12 * ld(mk, 0x176510);
        st(mk, 0x17651C, t17);
        let t18 = t17 * ld(mk, 0x176504);
        let t19 = t14 + t18;
        let t20 = t16 + t19;
        st(mk, 0x176588, t20);
        let t21 = t20 * t20;
        let t22 = t21 * ld(mk, 0x24C);
        sti(mk, 0x176520, t22);
    }
}

/// `square`'s x87 block as plain Rust on the x87 model (38 instructions), same order and memory places.
unsafe fn square_x87(rc: usize, bf: usize, mk: usize, i: usize) {
    unsafe {
        let t0 = ld(mk, 0x16D39C);
        let t1 = ld(rc, 0x224088) / t0;
        st(mk, 0x16D2B4, t1);
        let t2 = ld(mk, 0x16D398);
        let t3 = t2 * t1;
        let t4 = t1 * ld(mk, 0x16D394);
        let t5 = -t3;
        let t6 = t5 + ld(rc, 0x2240A0);
        st(bf, 0x5605A4, t6);
        let t7 = t4 + ld(rc, 0x22409C);
        st(bf, 0x5605A0, t7);
        let t8 = ld(mk, 0x176608);
        let t9 = t8 * ld(mk, 0x16D2B4);
        let t10 = ld(bf, 0x5605A0);
        let t11 = t10 + t9;
        st(bf, 0x5605C0, t11);
        st(bf, 0x5605E0, t11);
        let t12 = t11 - t9;
        st(bf, 0x560600, t12);
        let t13 = ld(bf, 0x5605A4);
        st(bf, 0x5605C4, t13);
        let t14 = t9 + t13;
        st(bf, 0x5605E4, t14);
        st(bf, 0x560604, t14);
        let t15 = ld(mk, 0x16D39C);
        let t16 = t15 * ld(mk, 0x16D2B8);
        st(bf, 0x5605A8, t16);
        st(bf, 0x5605C8, t16);
        st(bf, 0x5605E8, t16);
        st(bf, 0x560608, t16);
        let t17 = ld(mk, i.wrapping_add(0x16B0C0));
        st(bf, 0x5605B8, t17);
        st(bf, 0x560618, t17);
        let t18 = t17 + ld(mk, 0x17660C);
        st(bf, 0x5605D8, t18);
        st(bf, 0x5605F8, t18);
    }
}

#[cfg(test)]
mod asm_reference {
    //! The original assembly blocks, kept verbatim as the reference the plain versions are tested against.

    pub(super) unsafe fn turn_x87(mk: usize, v: usize) {
        unsafe {
            core::arch::asm!(
                    "fild dword ptr [{v}]",
            "fild dword ptr [{v} + 4]",
            "fild dword ptr [{v} + 8]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x16D2E0]",
            ".byte 0xD9, 0xC3",
            "fmul dword ptr [{mk} + 0x16D2EC]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{mk} + 0x16D2F8]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{mk} + 0x16D2C8]",
            "fstp dword ptr [{mk} + 0x16D394]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x16D2E4]",
            ".byte 0xD9, 0xC3",
            "fmul dword ptr [{mk} + 0x16D2F0]",
            ".byte 0xD9, 0xC2",
            "fmul dword ptr [{mk} + 0x16D2FC]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{mk} + 0x16D2CC]",
            "fstp dword ptr [{mk} + 0x16D398]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x16D2E8]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{mk} + 0x16D2F4]",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x16D300]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fiadd dword ptr [{mk} + 0x16D2D0]",
            "fist dword ptr [{mk} + 0x16D2B4]",
            "fstp dword ptr [{mk} + 0x16D39C]",
            mk = in(reg) mk,
            v = in(reg) v,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) unsafe fn fade_x87(mk: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x16D394]",
            "fsub dword ptr [{mk} + 0x1764F0]",
            "fst dword ptr [{mk} + 0x176508]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x16D398]",
            "fsub dword ptr [{mk} + 0x1764F4]",
            "fst dword ptr [{mk} + 0x17650C]",
            ".byte 0xD8, 0xC8",
            "fld dword ptr [{mk} + 0x16D39C]",
            "fsub dword ptr [{mk} + 0x1764F8]",
            "fst dword ptr [{mk} + 0x176510]",
            ".byte 0xD8, 0xC8",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            ".byte 0xD9, 0xFA",
            "fdivr dword ptr [{mk} + 0x22C]",
            ".byte 0xD9, 0xC0",
            "fmul dword ptr [{mk} + 0x176508]",
            "fst dword ptr [{mk} + 0x176514]",
            "fmul dword ptr [{mk} + 0x1764FC]",
            ".byte 0xD9, 0xC1",
            "fmul dword ptr [{mk} + 0x17650C]",
            "fst dword ptr [{mk} + 0x176518]",
            "fmul dword ptr [{mk} + 0x176500]",
            ".byte 0xD9, 0xCA",
            "fmul dword ptr [{mk} + 0x176510]",
            "fst dword ptr [{mk} + 0x17651C]",
            "fmul dword ptr [{mk} + 0x176504]",
            ".byte 0xDE, 0xC1",
            ".byte 0xDE, 0xC1",
            "fst dword ptr [{mk} + 0x176588]",
            ".byte 0xD8, 0xC8",
            "fmul dword ptr [{mk} + 0x24C]",
            "fistp dword ptr [{mk} + 0x176520]",
            mk = in(reg) mk,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }

    pub(super) unsafe fn square_x87(rc: usize, bf: usize, mk: usize, i: usize) {
        unsafe {
            core::arch::asm!(
                    "fld dword ptr [{mk} + 0x16D39C]",
            "fdivr dword ptr [{rc} + 0x224088]",
            "fst dword ptr [{mk} + 0x16D2B4]",
            "fld dword ptr [{mk} + 0x16D398]",
            ".byte 0xD8, 0xC9",
            ".byte 0xD9, 0xC9",
            "fmul dword ptr [{mk} + 0x16D394]",
            ".byte 0xD9, 0xC9",
            ".byte 0xD9, 0xE0",
            "fadd dword ptr [{rc} + 0x2240A0]",
            "fstp dword ptr [{bf} + 0x5605A4]",
            "fadd dword ptr [{rc} + 0x22409C]",
            "fstp dword ptr [{bf} + 0x5605A0]",
            "fld dword ptr [{mk} + 0x176608]",
            "fmul dword ptr [{mk} + 0x16D2B4]",
            "fld dword ptr [{bf} + 0x5605A0]",
            ".byte 0xD8, 0xC1",
            "fst dword ptr [{bf} + 0x5605C0]",
            "fst dword ptr [{bf} + 0x5605E0]",
            ".byte 0xD8, 0xE1",
            "fstp dword ptr [{bf} + 0x560600]",
            "fld dword ptr [{bf} + 0x5605A4]",
            "fst dword ptr [{bf} + 0x5605C4]",
            ".byte 0xDE, 0xC1",
            "fst dword ptr [{bf} + 0x5605E4]",
            "fstp dword ptr [{bf} + 0x560604]",
            "fld dword ptr [{mk} + 0x16D39C]",
            "fmul dword ptr [{mk} + 0x16D2B8]",
            "fst dword ptr [{bf} + 0x5605A8]",
            "fst dword ptr [{bf} + 0x5605C8]",
            "fst dword ptr [{bf} + 0x5605E8]",
            "fstp dword ptr [{bf} + 0x560608]",
            "fld dword ptr [{mk} + {i} + 0x16B0C0]",
            "fst dword ptr [{bf} + 0x5605B8]",
            "fst dword ptr [{bf} + 0x560618]",
            "fadd dword ptr [{mk} + 0x17660C]",
            "fst dword ptr [{bf} + 0x5605D8]",
            "fstp dword ptr [{bf} + 0x5605F8]",
            rc = in(reg) rc,
            bf = in(reg) bf,
            mk = in(reg) mk,
            i = in(reg) i,
            out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
            out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
            options(nostack),
            )
        }
    }
}

#[cfg(test)]
mod asm_differential {
    use super::asm_reference as reference;
    use crate::marks::asm_test::Kind::*;
    use crate::marks::asm_test::{Op, Spec, differential};

    #[test]
    fn turn_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x16D2B4,
                    hi: 0x16D3A0,
                    reads: &[
                        (0x16D2C8, I32),
                        (0x16D2CC, I32),
                        (0x16D2D0, I32),
                        (0x16D2E0, F32),
                        (0x16D2E4, F32),
                        (0x16D2E8, F32),
                        (0x16D2EC, F32),
                        (0x16D2F0, F32),
                        (0x16D2F4, F32),
                        (0x16D2F8, F32),
                        (0x16D2FC, F32),
                        (0x16D300, F32),
                    ],
                    writes: &[
                        (0x16D2B4, I32),
                        (0x16D394, F32),
                        (0x16D398, F32),
                        (0x16D39C, F32),
                    ],
                },
                Op::Base {
                    lo: 0x0,
                    hi: 0xC,
                    reads: &[(0x0, I32), (0x4, I32), (0x8, I32)],
                    writes: &[],
                },
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::turn_x87(b[0], b[1]) },
            |b| unsafe { super::turn_x87(b[0], b[1]) },
        );
    }

    #[test]
    fn fade_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[Op::Base {
                lo: 0x22C,
                hi: 0x17658C,
                reads: &[
                    (0x22C, F32),
                    (0x24C, F32),
                    (0x16D394, F32),
                    (0x16D398, F32),
                    (0x16D39C, F32),
                    (0x1764F0, F32),
                    (0x1764F4, F32),
                    (0x1764F8, F32),
                    (0x1764FC, F32),
                    (0x176500, F32),
                    (0x176504, F32),
                    (0x176508, F32),
                    (0x17650C, F32),
                    (0x176510, F32),
                ],
                writes: &[
                    (0x176508, F32),
                    (0x17650C, F32),
                    (0x176510, F32),
                    (0x176514, F32),
                    (0x176518, F32),
                    (0x17651C, F32),
                    (0x176520, I32),
                    (0x176588, F32),
                ],
            }],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::fade_x87(b[0]) },
            |b| unsafe { super::fade_x87(b[0]) },
        );
    }

    #[test]
    fn square_x87_matches() {
        const SPEC: Spec = Spec {
            operands: &[
                Op::Base {
                    lo: 0x224088,
                    hi: 0x2240A4,
                    reads: &[(0x224088, F32), (0x22409C, F32), (0x2240A0, F32)],
                    writes: &[],
                },
                Op::Base {
                    lo: 0x5605A0,
                    hi: 0x56061C,
                    reads: &[(0x5605A0, F32), (0x5605A4, F32)],
                    writes: &[
                        (0x5605A0, F32),
                        (0x5605A4, F32),
                        (0x5605A8, F32),
                        (0x5605B8, F32),
                        (0x5605C0, F32),
                        (0x5605C4, F32),
                        (0x5605C8, F32),
                        (0x5605D8, F32),
                        (0x5605E0, F32),
                        (0x5605E4, F32),
                        (0x5605E8, F32),
                        (0x5605F8, F32),
                        (0x560600, F32),
                        (0x560604, F32),
                        (0x560608, F32),
                        (0x560618, F32),
                    ],
                },
                Op::Base {
                    lo: 0x16B0C0,
                    hi: 0x176610,
                    reads: &[
                        (0x16B0C0, F32),
                        (0x16B0C4, F32),
                        (0x16B0C8, F32),
                        (0x16B250, F32),
                        (0x16D2B4, F32),
                        (0x16D2B8, F32),
                        (0x16D394, F32),
                        (0x16D398, F32),
                        (0x16D39C, F32),
                        (0x16F0BC, F32),
                        (0x176608, F32),
                        (0x17660C, F32),
                    ],
                    writes: &[(0x16D2B4, F32)],
                },
                Op::Index(&[0, 4, 8, 400, 16380]),
            ],
            sincos: false,
        };
        differential(
            &SPEC,
            |b| unsafe { reference::square_x87(b[0], b[1], b[2], b[3]) },
            |b| unsafe { super::square_x87(b[0], b[1], b[2], b[3]) },
        );
    }
}
