//! `scene::add_car_light` (0x44F29F): what one car's lamps add to the light on an
//! object.
//!
//! In:  esi = the car's block
//! Out: nothing changed if the car's lamps are off; else the original leaves eax, ebx,
//!      ecx and edx changed; both its callers keep every register, and the Rust
//!      version changes none
//!
//! Nothing unless the block's dword +0x84 is set. The place being lit
//! (`dword_636588` ..) less the car's (+0x134 ..), through the car's matrix (+0x9CC;
//! `scene::rotate_point`), as whole numbers in `dword_6365B8` ..: the place as the
//! car sees it, x across, y up, z ahead. Then three amounts, each held to 0 .. 255:
//! - 0x100 less a quarter of ((z - 0x200000) squared + (2x) squared) times
//!   `flt_5C5DA0` twice: a cone ahead of the car;
//! - one the original works out and then overwrites (((z - 0x100000) squared + x
//!   squared) times `flt_5C5DA0`, whose rounding is left in `dword_62400C`);
//! - (0x100000 less y without its sign) over 0x1000: how near the car's height.
//!
//! The third times twice the first over 256, times the block's dword +0xABC over 256, held to
//! 255, picks a colour from the table `dword_635D18`, which is added to `g_lightColour` byte by
//! byte without overflow. The x87's arithmetic in `X` (plain Rust since 2026-10-24; the
//! original's instructions are `asm_reference` in the tests). Callers: `sub_44F1FE`, twice.
//! Takes the car's block as a typed reference.

use crate::physics::car_store::Car;
use crate::race::numbers_store::NUMBERS;
use crate::scene::draw_store::DRAW;
use crate::scene::light_store::LIGHT_STATE;
use crate::scene::light_work_store::LIGHT_WORK;
use crate::scene::vertex_light::plus;
use crate::scene::x87_mem::{Base, cells, fild, fistp, fld};
use crate::session::MACHINE;

// ---- Rust version ----

/// Three whole numbers through a matrix and back to whole numbers: `fild` three times,
/// `scene::rotate_point`, `fistp` three times.
fn turned(from: [u32; 3], matrix: &[u32]) -> [u32; 3] {
    let mut w = [from[0], from[1], from[2], 0, 0, 0];
    turned_at(cells(&mut w), matrix);
    [w[3], w[4], w[5]]
}

fn turned_at(w: impl Base, m: impl Base) {
    let t0 = fild(w, 0x8);
    let t1 = fild(w, 0x4);
    let t2 = fild(w, 0x0);
    let t3 = fld(m, 0x0);
    let t4 = t3 * t2;
    let t5 = fld(m, 0x10);
    let t6 = t5 * t1;
    let t7 = fld(m, 0x20);
    let t8 = t7 * t0;
    let t9 = t6 + t8;
    let t10 = t4 + t9;
    let t11 = fld(m, 0x30);
    let t12 = t10 + t11;
    let t13 = fld(m, 0x4);
    let t14 = t13 * t2;
    let t15 = fld(m, 0x14);
    let t16 = t15 * t1;
    let t17 = fld(m, 0x24);
    let t18 = t17 * t0;
    let t19 = t16 + t18;
    let t20 = t14 + t19;
    let t21 = fld(m, 0x34);
    let t22 = t20 + t21;
    let t23 = fld(m, 0x8);
    let t24 = t2 * t23;
    let t25 = fld(m, 0x18);
    let t26 = t1 * t25;
    let t27 = fld(m, 0x28);
    let t28 = t0 * t27;
    let t29 = t28 + t26;
    let t30 = t24 + t29;
    let t31 = fld(m, 0x38);
    let t32 = t30 + t31;
    fistp(w, 0xC, t12);
    fistp(w, 0x10, t22);
    fistp(w, 0x14, t32);
}

/// `fild a; fmul st, st; fild b; fmul st, st; faddp; fmul k` (once or twice) `; fistp`.
fn squares(a: u32, b: u32, factor: u32, twice: bool) -> u32 {
    let (v, mut out) = ([a, b, factor], [0u32]);
    if twice {
        squares_twice(&v[..], cells(&mut out));
    } else {
        squares_once(&v[..], cells(&mut out));
    }
    out[0]
}

/// Twice the factor.
fn squares_twice(v: impl Base, o: impl Base) {
    let t0 = fild(v, 0x0);
    let t1 = t0 * t0;
    let t2 = fild(v, 0x4);
    let t3 = t2 * t2;
    let t4 = t1 + t3;
    let t5 = fld(v, 0x8);
    let t6 = t4 * t5;
    let t7 = fld(v, 0x8);
    let t8 = t6 * t7;
    fistp(o, 0x0, t8);
}

/// Once the factor.
fn squares_once(v: impl Base, o: impl Base) {
    let t0 = fild(v, 0x0);
    let t1 = t0 * t0;
    let t2 = fild(v, 0x4);
    let t3 = t2 * t2;
    let t4 = t1 + t3;
    let t5 = fld(v, 0x8);
    let t6 = t4 * t5;
    fistp(o, 0x0, t6);
}

/// `cmp eax, 0; jns; eax = 0` and `cmp eax, 0FFh; js; eax = 0FFh`.
fn held(value: i32) -> u32 {
    let value = value.max(0);
    if value.wrapping_sub(0xFF) < 0 {
        value as u32
    } else {
        0xFF
    }
}

pub fn add_car_light(car: &Car) {
    if car.lights_on.get() == 0 {
        return;
    }
    let place: [u32; 3] = crate::session::read_bits(&LIGHT_STATE.lit_place);
    let mut from = [0u32; 3];
    for k in 0..3 {
        from[k] = place[k].wrapping_sub(car.position[k].bits());
    }
    let matrix = crate::session::read_bits(&car.car_inverse_matrix);
    let seen = turned(from, &matrix);
    crate::session::write_bits(&LIGHT_STATE.turned_light, seen);
    let [x, y, z] = seen;
    let factor: u32 = NUMBERS.one_over_65536.bits();
    let ahead = squares(z.wrapping_sub(0x20_0000), x.wrapping_add(x), factor, true);
    let cone = held(0x100i32.wrapping_sub((ahead >> 2) as i32));
    // Worked out and not used; its rounding is what `dword_62400C` is left with.
    DRAW.scratch
        .set(squares(z.wrapping_sub(0x10_0000), x, factor, false));
    let height = if (y as i32) < 0 { y.wrapping_neg() } else { y };
    let level = held((0x10_0000u32.wrapping_sub(height) as i32) >> 12);
    let amount = (level.wrapping_mul(cone * 2) >> 8).wrapping_mul(car.marks_count.bits()) >> 8;
    let colour: u32 = crate::scene::owner::scene().light_tables.car_light_table
        [held(amount as i32) as usize]
        .get();
    let mmx = MACHINE.has_mmx.get() != 0;
    LIGHT_WORK
        .light_colour
        .set(plus(colour, LIGHT_WORK.light_colour.bits(), mmx));
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Kind, differential};

    /// The original's instructions for `turned`.
    unsafe fn turned_at_ref(w: *mut u8, m: *mut u8) {
        unsafe {
            core::arch::asm!(
                "fild dword ptr [{w} + 8]",
                "fild dword ptr [{w} + 4]",
                "fild dword ptr [{w}]",
                "fld dword ptr [{m}]",
                "fmul st(0), st(1)",
                "fld dword ptr [{m} + 0x10]",
                "fmul st(0), st(3)",
                "fld dword ptr [{m} + 0x20]",
                "fmul st(0), st(5)",
                "faddp st(1), st(0)",
                "faddp st(1), st(0)",
                "fadd dword ptr [{m} + 0x30]",
                "fxch st(3)",
                "fld dword ptr [{m} + 4]",
                "fmul st(0), st(2)",
                "fld dword ptr [{m} + 0x14]",
                "fmul st(0), st(4)",
                "fld dword ptr [{m} + 0x24]",
                "fmul st(0), st(3)",
                "faddp st(1), st(0)",
                "faddp st(1), st(0)",
                "fadd dword ptr [{m} + 0x34]",
                "fxch st(3)",
                "fld dword ptr [{m} + 8]",
                "fmulp st(3), st(0)",
                "fld dword ptr [{m} + 0x18]",
                "fmulp st(1), st(0)",
                "fld dword ptr [{m} + 0x28]",
                "fmulp st(2), st(0)",
                "faddp st(1), st(0)",
                "faddp st(1), st(0)",
                "fadd dword ptr [{m} + 0x38]",
                "fxch st(2)",
                "fistp dword ptr [{w} + 12]",
                "fistp dword ptr [{w} + 16]",
                "fistp dword ptr [{w} + 20]",
                w = in(reg) w,
                m = in(reg) m,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    /// The original's instructions for `squares` (twice).
    unsafe fn squares_twice_ref(v: *mut u8, o: *mut u8) {
        unsafe {
            core::arch::asm!(
                "fild dword ptr [{v}]",
                ".byte 0xD8, 0xC8", // fmul st, st
                "fild dword ptr [{v} + 4]",
                ".byte 0xD8, 0xC8",
                ".byte 0xDE, 0xC1", // faddp st(1), st
                "fmul dword ptr [{v} + 8]",
                "fmul dword ptr [{v} + 8]",
                "fistp dword ptr [{o}]",
                v = in(reg) v,
                o = in(reg) o,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    /// The original's instructions for `squares` (once).
    unsafe fn squares_once_ref(v: *mut u8, o: *mut u8) {
        unsafe {
            core::arch::asm!(
                "fild dword ptr [{v}]",
                ".byte 0xD8, 0xC8",
                "fild dword ptr [{v} + 4]",
                ".byte 0xD8, 0xC8",
                ".byte 0xDE, 0xC1",
                "fmul dword ptr [{v} + 8]",
                "fistp dword ptr [{o}]",
                v = in(reg) v,
                o = in(reg) o,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    #[test]
    fn turned_matches_the_assembly() {
        // dwords 0 .. 3 the place, 3 .. 6 the result, 8 .. 23 the matrix
        differential::<23>(
            "add_car_light turned",
            8000,
            16,
            [Kind::Exact; 23],
            |rng, m| {
                m.fill(0xAAAA_AAAA);
                for d in m.iter_mut().take(3) {
                    *d = rng.int();
                }
                let moderate = rng.below(2) == 0;
                for d in m.iter_mut().skip(8) {
                    *d = if moderate { rng.moderate() } else { rng.fl() };
                }
            },
            |base| unsafe { turned_at_ref(base, base.add(32)) },
            |base| unsafe { turned_at(base, base.add(32)) },
        );
    }

    #[test]
    fn squares_matches_the_assembly() {
        // dwords 0 .. 3 the two whole numbers and the factor, 3 the result of the first
        // way, 4 of the second
        differential::<5>(
            "add_car_light squares",
            8000,
            17,
            [Kind::Exact; 5],
            |rng, m| {
                m[0] = rng.int();
                m[1] = rng.int();
                m[2] = if rng.below(2) == 0 {
                    rng.moderate()
                } else {
                    rng.fl()
                };
                m[3] = 0xAAAA_AAAA;
                m[4] = 0xAAAA_AAAA;
            },
            |base| unsafe {
                squares_twice_ref(base, base.add(12));
                squares_once_ref(base, base.add(16));
            },
            |base| unsafe {
                squares_twice(base, base.add(12));
                squares_once(base, base.add(16));
            },
        );
    }
}
