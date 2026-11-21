//! The tyre-change screen's turning quad (the steering
//! wheel): the device's turn turning the half-sizes of the rectangle (floats: x, y, w, h)
//! round its place plus `TyreChange::wheel`'s place, with the given colour and the texture
//! rectangle (`quad_tail`), drawn (place 5).

use crate::d3d::vertices_store::Vertex;
use crate::drive::tyre_parts::*;
use crate::errors::Refused;
use crate::input::types::InputDevice;
use crate::math::x87_float::{int, ld, stf};
use crate::race::owner_shared::RaceShared;
use crate::race::tyre_store::TyreChange;

/// The turn's arithmetic: `ty` the tyre change's variables (the constants and results), `gauge`
/// the input device's gauge, `d` the rectangle (floats), `v` the vertices, `two_pi` the scratch
/// float's bits holding 2 pi.
fn turn(ty: &mut TyreChange, gauge: u32, d: [f32; 4], v: &[Vertex], two_pi: u32) {
    let angle = int(gauge as i32) * ld(ty.gauge_unit) * ld(two_pi);
    let (sin, cos) = angle.sin_cos();
    ty.turn_cos = stf(cos);
    ty.turn_sin = stf(sin);
    ty.half_h_sin = stf(ld(d[3]) * ld(ty.turn_sin));
    ty.neg_half_h_cos = stf(-(ld(d[3]) * ld(ty.turn_cos)));
    ty.half_w_cos = stf(ld(d[2]) * ld(ty.turn_cos));
    ty.half_w_sin = stf(ld(d[2]) * ld(ty.turn_sin));
    let (wheel_x, wheel_y) = (int(ty.wheel[0] as i32), int(ty.wheel[1] as i32));
    let x = -ld(ty.half_w_cos) + ld(d[0]) + wheel_x;
    v[2].x.set(stf(x));
    v[0].x.set(stf(x + ld(ty.half_h_sin)));
    let x = ld(ty.half_w_cos) + ld(d[0]) + wheel_x;
    v[3].x.set(stf(x));
    v[1].x.set(stf(x + ld(ty.half_h_sin)));
    let y = -ld(ty.half_w_sin) + ld(d[1]) + wheel_y;
    v[2].y.set(stf(y));
    v[0].y.set(stf(y + ld(ty.neg_half_h_cos)));
    let y = ld(ty.half_w_sin) + ld(d[1]) + wheel_y;
    v[3].y.set(stf(y));
    v[1].y.set(stf(y + ld(ty.neg_half_h_cos)));
}

/// The texture rectangle `uv`, the screen rectangle `rect`, the colour `colour`, the input device
/// `device`.
#[allow(
    clippy::too_many_arguments,
    reason = "the tyre change, the quad's inputs, the owners and the part drawn on"
)]
pub fn tyre_turned_quad(
    shared: &RaceShared,
    uv: [u32; 4],
    rect: [f32; 4],
    colour: u32,
    device: &InputDevice,
    system: &crate::system::owner::System,
    panel: &crate::race::viewer_screen_store::ViewerPanelBlock,
    part: &crate::scene::scratch_store::PartScratch,
) -> Result<(), Refused> {
    let buffer = &part.vertices;
    system.machine().scratch.set(0x40C9_0FDB);
    turn(
        &mut shared.tyre_change_mut(),
        device.gauge.get(),
        rect,
        &buffer.vertices[..4],
        system.machine().scratch.get(),
    );
    quad_words(buffer, colour);
    quad_tail(buffer, uv, panel, system, Spread::Centred);
    draw(part)
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::math::x87::{control_word, set_control_word};
    use crate::test_support::vectors::{CONTROL_WORDS, Recorder, Rng, close_float};

    /// The struct's offsets of the window the turn writes and reads.
    const WINDOW: usize = 0;
    const WINDOW_END: usize = 0x148;

    fn put(buf: &mut [u8], off: usize, v: u32) {
        buf[off..off + 4].copy_from_slice(&v.to_le_bytes());
    }

    fn get(buf: &[u8], off: usize) -> u32 {
        u32::from_le_bytes(buf[off..off + 4].try_into().unwrap())
    }

    fn fresh() -> Box<TyreChange> {
        Box::new(TyreChange::new())
    }

    fn bytes(ty: &TyreChange) -> &[u8] {
        const { assert!(WINDOW_END <= size_of::<TyreChange>()) };
        // SAFETY: the record's first `WINDOW_END` bytes (inside it, asserted above), all of them
        // `u32`s, `f32`s and bytes (no padding under the test's `repr(C)`, any bits a byte); read only
        // while the slice lives, which borrows `ty`.
        unsafe {
            std::slice::from_raw_parts(
                std::ptr::from_ref::<TyreChange>(ty).cast::<u8>(),
                WINDOW_END,
            )
        }
    }

    fn put_ty(ty: &mut TyreChange, off: usize, v: u32) {
        assert!(
            off.is_multiple_of(4) && off + 4 <= size_of::<TyreChange>(),
            "a dword of the record"
        );
        // SAFETY: a whole dword of the record (asserted above), 4-aligned as the record is, inside
        // fields that are `u32`s, `f32`s or bytes (any bits valid); `ty` is borrowed for the write.
        unsafe {
            std::ptr::from_mut::<TyreChange>(ty)
                .cast::<u8>()
                .add(off)
                .cast::<u32>()
                .write(v);
        }
    }

    /// The device's turn is a small whole number and the factor small, so the angle is
    /// moderate (the x87 and libm reduce large ones differently). The tyre block is one
    /// buffer per version, only its window refilled each case.
    #[test]
    fn turn_matches_the_assembly() {
        let _g = crate::test_support::test_lock();
        let saved = control_word();
        let mut rng = Rng::new(206);
        let (mut ty_a, mut ty_b) = (fresh(), fresh());
        let (mut e_a, mut e_b) = (vec![0u8; 0x4F0 + 16], vec![0u8; 0x4F0 + 16]);
        let (mut d_a, mut d_b) = (vec![0u8; 32], vec![0u8; 32]);
        let (mut v_a, mut v_b) = (vec![0u8; 0x90], vec![0u8; 0x90]);
        let (mut t_a, mut t_b) = (vec![0u8; 16], vec![0u8; 16]);
        let mut checked = 0u64;
        let name = "drive.tyre_turned_quad.turn";
        let mut file = Recorder::replay(name);
        for &cw in &CONTROL_WORDS {
            set_control_word(cw);
            for n in 0..1000 {
                for off in (WINDOW..WINDOW_END).step_by(4) {
                    put_ty(&mut ty_a, off, 0xAAAA_AAAA);
                }
                v_a.fill(0xAA);
                put(&mut e_a, 0x4EC, rng.small() as u32);
                let factor = match rng.below(3) {
                    0 => rng.unit() * 1e-3,
                    1 => rng.unit() * 1e-4,
                    _ => (rng.small() as f32) * 1e-6,
                };
                put_ty(&mut ty_a, 0x8, factor.to_bits());
                put(&mut t_a, 0, 0x40C9_0FDB);
                for off in [0, 4, 8, 0xC] {
                    put(&mut d_a, off, rng.moderate());
                }
                put_ty(&mut ty_a, 0x20, rng.small() as u32);
                put_ty(&mut ty_a, 0x24, rng.small() as u32);
                for off in (WINDOW..WINDOW_END).step_by(4) {
                    put_ty(&mut ty_b, off, get(bytes(&ty_a), off));
                }
                v_b.copy_from_slice(&v_a);
                e_b.copy_from_slice(&e_a);
                d_b.copy_from_slice(&d_a);
                t_b.copy_from_slice(&t_a);
                let before: Vec<u8> = [bytes(&ty_a), &v_a[..0x80]].concat();
                file.input(&before);
                file.input(&e_a[0x4EC..0x4F0]);
                file.input(&d_a[..16]);
                file.input(&t_a[..4]);
                let expected = file.results(&before, || unreachable!("replayed"));
                let floats: [f32; 4] = std::array::from_fn(|k| f32::from_bits(get(&d_b, 4 * k)));
                // SAFETY: the buffer is four vertices' (`0x90` bytes), aligned.
                let vertices = unsafe { &*v_b.as_ptr().cast::<[Vertex; 4]>() };
                turn(&mut ty_b, get(&e_b, 0x4EC), floats, vertices, get(&t_b, 0));
                let (want_ty, want_v) = expected.split_at(WINDOW_END);
                let mut places: Vec<(&str, usize, &[u8], &[u8])> = Vec::new();
                for off in (WINDOW..WINDOW_END).step_by(4) {
                    places.push(("ty", off, want_ty, bytes(&ty_b)));
                }
                for off in (0..0x80).step_by(4) {
                    places.push(("v", off, want_v, &v_b));
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
        file.finish();
        eprintln!("turn: {checked} dwords compared");
    }
}
