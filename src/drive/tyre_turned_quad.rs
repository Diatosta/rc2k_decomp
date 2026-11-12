//! `drive::tyre_turned_quad` (0x41C969): the tyre-change screen's turning quad (the steering
//! wheel): the device's turn turning the half-sizes of the rectangle (floats: x, y, w, h)
//! round its place plus `TyreChange::wheel`'s place, with the given colour and the texture
//! rectangle (`quad_tail`), drawn (place 5).
//!
//! Takes: the texture rectangle, the rectangle, the colour, the input device
//!
//! Caller: `drive::tyre_screen`.

use crate::d3d::vertices_store::{VERTICES, Vertex};
use crate::drive::tyre_parts::*;
use crate::errors::Refused;
use crate::input::types::InputDevice;
use crate::math::x87_float::{int, ld, stf};
use crate::race::owner_shared::RaceShared;
use crate::race::tyre_store::TyreChange;
use crate::session::{Dword, Float};

/// The turn's arithmetic: `ty` the tyre change's variables (the constants and results), `e`
/// the input device, `d` the rectangle (floats), `v` the vertices, `t` the scratch float
/// holding 2 pi.
fn turn(ty: &TyreChange, e: &InputDevice, d: &[Float; 4], v: &[Vertex], t: &Dword) {
    let angle = int(e.gauge.get() as i32) * ld(ty.gauge_unit.get()) * ld(t.get());
    let (sin, cos) = angle.sin_cos();
    ty.turn_cos.set(stf(cos));
    ty.turn_sin.set(stf(sin));
    ty.half_h_sin
        .set(stf(ld(d[3].get()) * ld(ty.turn_sin.get())));
    ty.neg_half_h_cos
        .set(stf(-(ld(d[3].get()) * ld(ty.turn_cos.get()))));
    ty.half_w_cos
        .set(stf(ld(d[2].get()) * ld(ty.turn_cos.get())));
    ty.half_w_sin
        .set(stf(ld(d[2].get()) * ld(ty.turn_sin.get())));
    let (wheel_x, wheel_y) = (int(ty.wheel[0].get() as i32), int(ty.wheel[1].get() as i32));
    let x = -ld(ty.half_w_cos.get()) + ld(d[0].get()) + wheel_x;
    v[2].x.set(stf(x));
    v[0].x.set(stf(x + ld(ty.half_h_sin.get())));
    let x = ld(ty.half_w_cos.get()) + ld(d[0].get()) + wheel_x;
    v[3].x.set(stf(x));
    v[1].x.set(stf(x + ld(ty.half_h_sin.get())));
    let y = -ld(ty.half_w_sin.get()) + ld(d[1].get()) + wheel_y;
    v[2].y.set(stf(y));
    v[0].y.set(stf(y + ld(ty.neg_half_h_cos.get())));
    let y = ld(ty.half_w_sin.get()) + ld(d[1].get()) + wheel_y;
    v[3].y.set(stf(y));
    v[1].y.set(stf(y + ld(ty.neg_half_h_cos.get())));
}

/// The texture rectangle `uv`, the screen rectangle `rect`, the colour `colour`, the input device
/// `device`.
pub fn tyre_turned_quad(
    shared: &RaceShared,
    uv: [u32; 4],
    rect: &[Float; 4],
    colour: u32,
    device: &InputDevice,
    system: &crate::system::owner::System,
    panel: &crate::race::viewer_screen_store::ViewerPanelBlock,
) -> Result<(), Refused> {
    system.session.machine.scratch.set(0x40C9_0FDB);
    turn(
        &shared.tyre_change,
        device,
        rect,
        &VERTICES.vertices[..4],
        &system.session.machine.scratch,
    );
    quad_words(colour);
    quad_tail(uv, panel, system);
    draw()
}

#[cfg(all(test, target_arch = "x86"))]
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

    /// The struct's offsets of the window the turn writes and reads (IDA 0x5BEAC0 ..
    /// 0x5BEC08 less the struct's start).
    const WINDOW: usize = 0;
    const WINDOW_END: usize = 0x148;
    /// What the reference's `[ty + 0x1BEAC8]` operands add to: the struct's address less the
    /// block's first RVA, so that RVA offsets name the struct's bytes.
    const RVA_OFFSET: usize = 0x1BEAC0;

    fn put(buf: &mut [u8], off: usize, v: u32) {
        buf[off..off + 4].copy_from_slice(&v.to_le_bytes());
    }

    fn get(buf: &[u8], off: usize) -> u32 {
        u32::from_le_bytes(buf[off..off + 4].try_into().unwrap())
    }

    fn zeroed() -> Box<TyreChange> {
        // SAFETY: atomic cells and byte arrays only: all-zero is valid.
        Box::new(unsafe { std::mem::zeroed() })
    }

    fn bytes(ty: &TyreChange) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                std::ptr::from_ref::<TyreChange>(ty).cast::<u8>(),
                WINDOW_END,
            )
        }
    }

    fn put_ty(ty: &TyreChange, off: usize, v: u32) {
        unsafe { (std::ptr::from_ref::<TyreChange>(ty).cast::<u8>().add(off) as *mut u32).write(v) }
    }

    /// The device's turn is a small whole number and the factor small, so the angle is
    /// moderate (the x87 and libm reduce large ones differently). The tyre block is one
    /// buffer per version, only its window refilled each case.
    #[test]
    fn turn_matches_the_assembly() {
        let _g = lock();
        let saved = control_word();
        let mut rng = Rng::new(206);
        let (ty_a, ty_b) = (zeroed(), zeroed());
        let (mut e_a, mut e_b) = (vec![0u8; 0x4F0 + 16], vec![0u8; 0x4F0 + 16]);
        let (mut d_a, mut d_b) = (vec![0u8; 32], vec![0u8; 32]);
        let (mut v_a, mut v_b) = (vec![0u8; 0x90], vec![0u8; 0x90]);
        let (mut t_a, mut t_b) = (vec![0u8; 16], vec![0u8; 16]);
        // Lent while the runs read them as game memory.
        let as_bytes = |ty: &TyreChange| unsafe {
            std::slice::from_raw_parts(
                std::ptr::from_ref::<TyreChange>(ty).cast::<u8>(),
                size_of::<TyreChange>(),
            )
        };
        let _lent = [
            crate::pointers::Lent::new(as_bytes(&ty_a)),
            crate::pointers::Lent::new(as_bytes(&ty_b)),
            crate::pointers::Lent::new(&e_a),
            crate::pointers::Lent::new(&e_b),
            crate::pointers::Lent::new(&d_a),
            crate::pointers::Lent::new(&d_b),
            crate::pointers::Lent::new(&v_a),
            crate::pointers::Lent::new(&v_b),
            crate::pointers::Lent::new(&t_a),
            crate::pointers::Lent::new(&t_b),
        ];
        let mut checked = 0u64;
        for &cw in &crate::scene::asm_test::CONTROL_WORDS {
            set_control_word(cw);
            for n in 0..4000 {
                for off in (WINDOW..WINDOW_END).step_by(4) {
                    put_ty(&ty_a, off, 0xAAAA_AAAA);
                }
                v_a.fill(0xAA);
                put(&mut e_a, 0x4EC, rng.small() as u32);
                let factor = match rng.below(3) {
                    0 => rng.unit() * 1e-3,
                    1 => rng.unit() * 1e-4,
                    _ => (rng.small() as f32) * 1e-6,
                };
                put_ty(&ty_a, 0x8, factor.to_bits());
                put(&mut t_a, 0, 0x40C9_0FDB);
                for off in [0, 4, 8, 0xC] {
                    put(&mut d_a, off, rng.moderate());
                }
                put_ty(&ty_a, 0x20, rng.small() as u32);
                put_ty(&ty_a, 0x24, rng.small() as u32);
                for off in (WINDOW..WINDOW_END).step_by(4) {
                    put_ty(&ty_b, off, get(bytes(&ty_a), off));
                }
                v_b.copy_from_slice(&v_a);
                e_b.copy_from_slice(&e_a);
                d_b.copy_from_slice(&d_a);
                t_b.copy_from_slice(&t_a);
                unsafe {
                    ref_turn(
                        (crate::pointers::game_address(&raw const *ty_a)
                            .wrapping_sub(RVA_OFFSET as u32)) as usize,
                        (crate::pointers::game_address(e_a.as_mut_ptr())) as usize,
                        (crate::pointers::game_address(d_a.as_mut_ptr())) as usize,
                        (crate::pointers::game_address(v_a.as_mut_ptr())) as usize,
                        (crate::pointers::game_address(t_a.as_mut_ptr())) as usize,
                    );
                }
                // SAFETY: the buffers are the operands' (the device's record, four floats,
                // four vertices, a dword), lent and aligned; the device's record is
                // `0x4F0 + 16` bytes, its `gauge` at 0x4EC.
                unsafe {
                    turn(
                        &ty_b,
                        &*e_b.as_ptr().cast::<InputDevice>(),
                        &*d_b.as_ptr().cast::<[Float; 4]>(),
                        &*v_b.as_ptr().cast::<[Vertex; 4]>(),
                        &*t_b.as_ptr().cast::<Dword>(),
                    );
                }
                let mut places: Vec<(&str, usize, &[u8], &[u8])> = Vec::new();
                for off in (WINDOW..WINDOW_END).step_by(4) {
                    places.push(("ty", off, bytes(&ty_a), bytes(&ty_b)));
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
