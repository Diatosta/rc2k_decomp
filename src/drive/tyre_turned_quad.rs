//! `drive::tyre_turned_quad` (0x41C969): the tyre-change screen's turning quad (the
//! steering wheel): the device's turn (esi +0x4EC, x `flt_5BEAC8` x 2 pi, `fsincos` into
//! `flt_5BEBF4` cos, `flt_5BEBF0` sin) turning the half-sizes of the rectangle at ebx
//! (floats: x, y, w, h) round its place plus `dword_5BEAE0` / `5BEAE4`, the colour ecx, the
//! texture rectangle at eax (`quad_tail`), drawn (place 5). All in `pusha` / `popa`.
//!
//! In:  eax the texture rectangle, ebx the rectangle, ecx the colour, esi the input device
//! Out: every register kept; no flags read
//!
//! Caller: `drive::tyre_screen`.

use crate::drive::tyre_parts::*;
use crate::errors::Refused;
use crate::physics::flow::a;
use crate::scene::x87_mem::{fild, fld, fstp};
use crate::session::MACHINE;

/// The turn's arithmetic over the memory at the operands' addresses: `ty` the tyre-change
/// block (the constants and results at their RVA offsets), `e` the device, `d` the
/// rectangle, `v` the vertices, `t` the scratch float holding 2 pi.
fn turn(ty: usize, e: usize, d: usize, v: usize, t: usize) {
    let (ty, e, d, v, t) = (
        ty as *mut u8,
        e as *mut u8,
        d as *mut u8,
        v as *mut u8,
        t as *mut u8,
    );
    unsafe {
        let angle = fild(e, 0x4EC) * fld(ty, 0x1BEAC8) * fld(t, 0);
        let (sin, cos) = angle.sin_cos();
        fstp(ty, 0x1BEBF4, cos);
        fstp(ty, 0x1BEBF0, sin);
        fstp(ty, 0x1BEBF8, fld(d, 0xC) * fld(ty, 0x1BEBF0));
        fstp(ty, 0x1BEBFC, -(fld(d, 0xC) * fld(ty, 0x1BEBF4)));
        fstp(ty, 0x1BEC00, fld(d, 0x8) * fld(ty, 0x1BEBF4));
        fstp(ty, 0x1BEC04, fld(d, 0x8) * fld(ty, 0x1BEBF0));
        let x = -fld(ty, 0x1BEC00) + fld(d, 0) + fild(ty, 0x1BEAE0);
        fstp(v, 0x40, x);
        fstp(v, 0, x + fld(ty, 0x1BEBF8));
        let x = fld(ty, 0x1BEC00) + fld(d, 0) + fild(ty, 0x1BEAE0);
        fstp(v, 0x60, x);
        fstp(v, 0x20, x + fld(ty, 0x1BEBF8));
        let y = -fld(ty, 0x1BEC04) + fld(d, 0x4) + fild(ty, 0x1BEAE4);
        fstp(v, 0x44, y);
        fstp(v, 0x4, y + fld(ty, 0x1BEBFC));
        let y = fld(ty, 0x1BEC04) + fld(d, 0x4) + fild(ty, 0x1BEAE4);
        fstp(v, 0x64, y);
        fstp(v, 0x24, y + fld(ty, 0x1BEBFC));
    }
}

/// The Rust version: the texture rectangle at `uv` (eax), the screen rectangle at `rect`
/// (ebx), the colour `colour` (ecx), the input device `device` (esi; its +0x4EC turns
/// the quad); every register kept.
pub unsafe fn tyre_turned_quad(
    uv: u32,
    rect: u32,
    colour: u32,
    device: u32,
) -> Result<(), Refused> {
    MACHINE.scratch.set(0x40C9_0FDB);
    turn(
        crate::rest::low::tyres_asm_base() as usize,
        device as usize,
        rect as usize,
        a(VERTS) as usize,
        crate::session::MACHINE.scratch.addr(),
    );
    quad_words(colour);
    quad_tail(uv);
    draw()
}

#[cfg(test)]
mod asm_reference {
    use super::*;
    use crate::math::x87::{control_word, set_control_word};
    use crate::scene::asm_test::{Rng, close_float, lock};

    // The original's instructions.
    unsafe fn ref_turn(ty: usize, e: usize, d: usize, v: usize, t: usize) {
        unsafe {
            core::arch::asm!(
                "fild dword ptr [{e} + 0x4EC]",
                "fmul dword ptr [{ty} + 0x1BEAC8]",
                "fmul dword ptr [{t}]",
                ".byte 0xD9, 0xFB", // fsincos
                "fstp dword ptr [{ty} + 0x1BEBF4]",
                "fstp dword ptr [{ty} + 0x1BEBF0]",
                "fld dword ptr [{d} + 0xC]",
                "fmul dword ptr [{ty} + 0x1BEBF0]",
                "fstp dword ptr [{ty} + 0x1BEBF8]",
                "fld dword ptr [{d} + 0xC]",
                "fmul dword ptr [{ty} + 0x1BEBF4]",
                ".byte 0xD9, 0xE0", // fchs
                "fstp dword ptr [{ty} + 0x1BEBFC]",
                "fld dword ptr [{d} + 0x8]",
                "fmul dword ptr [{ty} + 0x1BEBF4]",
                "fstp dword ptr [{ty} + 0x1BEC00]",
                "fld dword ptr [{d} + 0x8]",
                "fmul dword ptr [{ty} + 0x1BEBF0]",
                "fstp dword ptr [{ty} + 0x1BEC04]",
                "fld dword ptr [{ty} + 0x1BEC00]",
                ".byte 0xD9, 0xE0",
                "fadd dword ptr [{d}]",
                "fiadd dword ptr [{ty} + 0x1BEAE0]",
                "fst dword ptr [{v} + 0x40]",
                "fadd dword ptr [{ty} + 0x1BEBF8]",
                "fstp dword ptr [{v}]",
                "fld dword ptr [{ty} + 0x1BEC00]",
                "fadd dword ptr [{d}]",
                "fiadd dword ptr [{ty} + 0x1BEAE0]",
                "fst dword ptr [{v} + 0x60]",
                "fadd dword ptr [{ty} + 0x1BEBF8]",
                "fstp dword ptr [{v} + 0x20]",
                "fld dword ptr [{ty} + 0x1BEC04]",
                ".byte 0xD9, 0xE0",
                "fadd dword ptr [{d} + 0x4]",
                "fiadd dword ptr [{ty} + 0x1BEAE4]",
                "fst dword ptr [{v} + 0x44]",
                "fadd dword ptr [{ty} + 0x1BEBFC]",
                "fstp dword ptr [{v} + 0x4]",
                "fld dword ptr [{ty} + 0x1BEC04]",
                "fadd dword ptr [{d} + 0x4]",
                "fiadd dword ptr [{ty} + 0x1BEAE4]",
                "fst dword ptr [{v} + 0x64]",
                "fadd dword ptr [{ty} + 0x1BEBFC]",
                "fstp dword ptr [{v} + 0x24]",
                ty = in(reg) ty, e = in(reg) e, d = in(reg) d, v = in(reg) v, t = in(reg) t,
                out("st(0)") _, out("st(1)") _, out("st(2)") _, out("st(3)") _,
                out("st(4)") _, out("st(5)") _, out("st(6)") _, out("st(7)") _,
                options(nostack),
            );
        }
    }

    const WINDOW: usize = 0x1BEAC0;
    const WINDOW_END: usize = 0x1BEC08;

    fn put(buf: &mut [u8], off: usize, v: u32) {
        buf[off..off + 4].copy_from_slice(&v.to_le_bytes());
    }

    fn get(buf: &[u8], off: usize) -> u32 {
        u32::from_le_bytes(buf[off..off + 4].try_into().unwrap())
    }

    /// The device's turn is a small whole number and the factor small, so the angle is
    /// moderate (the x87 and libm reduce large ones differently). The tyre block is one
    /// buffer per version, only its window refilled each case.
    #[test]
    fn turn_matches_the_assembly() {
        let _g = lock();
        let saved = control_word();
        let mut rng = Rng::new(206);
        let mut ty_a = vec![0u8; WINDOW_END + 16];
        let mut ty_b = ty_a.clone();
        let (mut e_a, mut e_b) = (vec![0u8; 0x4F0 + 16], vec![0u8; 0x4F0 + 16]);
        let (mut d_a, mut d_b) = (vec![0u8; 32], vec![0u8; 32]);
        let (mut v_a, mut v_b) = (vec![0u8; 0x90], vec![0u8; 0x90]);
        let (mut t_a, mut t_b) = (vec![0u8; 16], vec![0u8; 16]);
        let mut checked = 0u64;
        for &cw in &crate::scene::asm_test::CONTROL_WORDS {
            set_control_word(cw);
            for n in 0..4000 {
                ty_a[WINDOW..WINDOW_END].fill(0xAA);
                v_a.fill(0xAA);
                put(&mut e_a, 0x4EC, rng.small() as u32);
                let factor = match rng.below(3) {
                    0 => rng.unit() * 1e-3,
                    1 => rng.unit() * 1e-4,
                    _ => (rng.small() as f32) * 1e-6,
                };
                put(&mut ty_a, 0x1BEAC8, factor.to_bits());
                put(&mut t_a, 0, 0x40C9_0FDB);
                for off in [0, 4, 8, 0xC] {
                    put(&mut d_a, off, rng.moderate());
                }
                put(&mut ty_a, 0x1BEAE0, rng.small() as u32);
                put(&mut ty_a, 0x1BEAE4, rng.small() as u32);
                ty_b[WINDOW..WINDOW_END].copy_from_slice(&ty_a[WINDOW..WINDOW_END]);
                v_b.copy_from_slice(&v_a);
                e_b.copy_from_slice(&e_a);
                d_b.copy_from_slice(&d_a);
                t_b.copy_from_slice(&t_a);
                unsafe {
                    ref_turn(
                        ty_a.as_mut_ptr() as usize,
                        e_a.as_mut_ptr() as usize,
                        d_a.as_mut_ptr() as usize,
                        v_a.as_mut_ptr() as usize,
                        t_a.as_mut_ptr() as usize,
                    );
                }
                turn(
                    ty_b.as_mut_ptr() as usize,
                    e_b.as_mut_ptr() as usize,
                    d_b.as_mut_ptr() as usize,
                    v_b.as_mut_ptr() as usize,
                    t_b.as_mut_ptr() as usize,
                );
                let mut places: Vec<(&str, usize, &[u8], &[u8])> = Vec::new();
                for off in (WINDOW..WINDOW_END).step_by(4) {
                    places.push(("ty", off, &ty_a, &ty_b));
                }
                for off in (0..0x80).step_by(4) {
                    places.push(("v", off, &v_a, &v_b));
                }
                for (what, off, x, y) in places {
                    let (p, q) = (get(x, off), get(y, off));
                    checked += 1;
                    assert!(
                        close_float(p, q, 1),
                        "control word {cw:#06X}, case {n}, {what} +{off:#X}: reference {p:#010X}, plain {q:#010X}"
                    );
                }
            }
        }
        set_control_word(saved);
        eprintln!("turn: {checked} dwords compared");
    }
}
