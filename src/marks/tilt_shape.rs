//! `marks::tilt_shape` (0x4C1194): the sine and cosine of `g_marksShapeTilt` into
//! `flt_811BC7` / `flt_811BCB`; the point's y and z (`flt_811BE3`, `flt_811BE7`) turned by
//! them (`x87::turn_yz`); then the point `flt_811BDF` .. made of length 1 into
//! `unk_811BD3` .., its length in `dword_811BCF` (`x87::unit_and_length`).
//!
//! In:  nothing
//! Out: esi the point (0x811BDF), edi the unit point (0x811BD3); the rest kept
//!
//! Caller: `sub_4C1148`. Self-test: the angle and the point by lot, compared on the eight
//! results and the registers; put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `tilt_shape_plain`, through
//! `rc2k_core::marks::view::tilt` (plain floats; the x87 version stays for the Rust callers
//! in unconverted code). Shadow mode and the self-test compare within `service::floats`'
//! tolerances (the self-test at both test precisions, exact at double).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::marks::leaf_test::test_leaf_floats;
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::scene::matrix::float_places;
use crate::screens::harness::Left;
use crate::service::floats::*;
use crate::service::harness::{Region, at};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::tilt_shape", rva(0x4C1194), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const TILT_RVA: usize = rva(0x811BC3); // g_marksShapeTilt
const SINE_RVA: usize = rva(0x811BC7); // flt_811BC7
const COSINE_RVA: usize = rva(0x811BCB); // flt_811BCB
const POINT_RVA: usize = rva(0x811BDF); // flt_811BDF .. flt_811BE7
const UNIT_RVA: usize = rva(0x811BD3); // unk_811BD3
const LENGTH_RVA: usize = rva(0x811BCF); // dword_811BCF

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn tilt_shape(entry: &Regs) -> Regs {
    unsafe {
        let (s, c) = x87::sine_and_cosine(game_read(TILT_RVA));
        game_write(COSINE_RVA, c);
        game_write(SINE_RVA, s);
        let (y, z) = x87::turn_yz(game_read(POINT_RVA + 4), game_read(POINT_RVA + 8), s, c);
        game_write(POINT_RVA + 4, y);
        game_write(POINT_RVA + 8, z);
        let (unit, length) = x87::unit_and_length(game_read(POINT_RVA));
        game_write(LENGTH_RVA, length);
        game_write(UNIT_RVA, unit);
        Regs {
            esi: game_addr(POINT_RVA) as u32,
            edi: game_addr(UNIT_RVA) as u32,
            ..*entry
        }
    }
}

/// The converted version (plain floats): the hook's.
// Converted: `rc2k_core::marks::view::tilt`
pub unsafe fn tilt_shape_plain(entry: &Regs) -> Regs {
    unsafe {
        let point: [u32; 3] = game_read(POINT_RVA);
        let t = rc2k_core::marks::view::tilt(
            f32::from_bits(game_read(TILT_RVA)),
            point.map(f32::from_bits),
        );
        game_write(COSINE_RVA, t.turn.cosine.to_bits());
        game_write(SINE_RVA, t.turn.sine.to_bits());
        game_write(POINT_RVA + 4, t.y.to_bits());
        game_write(POINT_RVA + 8, t.z.to_bits());
        game_write(LENGTH_RVA, t.unit.length.to_bits());
        game_write(UNIT_RVA, t.unit.unit.map(f32::to_bits));
        Regs {
            esi: game_addr(POINT_RVA) as u32,
            edi: game_addr(UNIT_RVA) as u32,
            ..*entry
        }
    }
}

fn written() -> Vec<usize> {
    let mut w = vec![
        SINE_RVA,
        COSINE_RVA,
        POINT_RVA + 4,
        POINT_RVA + 8,
        LENGTH_RVA,
    ];
    w.extend((0..3).map(|k| UNIT_RVA + 4 * k));
    w
}

/// What it writes.
pub fn regions() -> Vec<Region> {
    written().into_iter().map(at).collect()
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { tilt_shape_plain(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions: Vec<Region> = written().into_iter().map(at).collect();
    let places = float_places(&regions);
    let floats = Floats {
        memory: &places,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &regions, &floats, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let inputs = [TILT_RVA, POINT_RVA, POINT_RVA + 4, POINT_RVA + 8];
    test_leaf_floats(
        &mut t,
        "marks::tilt_shape",
        &inputs,
        &written(),
        Regs::SENTINEL,
        &[],
        10,
        call_original,
        &rust,
    );
    t.finish()
}
