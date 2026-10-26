//! `marks::clamp_pair` (0x4BEF22): the pair in st(0), st(1) scaled to whole numbers, held
//! to 0 .. 0x7FFF and 0 .. 0xFFFF, and scaled back.
//!
//! In:  st(0), st(1) the pair
//! Out: ebx, eax the whole numbers; st(0), st(1) them scaled back (unrounded)
//!
//! See `marks/turns.rs`. Thunk by `pair_hook!`. Callers: `sub_4BEB25` (twice). Self-test:
//! the original and the Rust version on pairs by lot (negative, large, NaN among them),
//! at the test precisions.

use crate::errors::Refused;
use crate::framework::{EAX, EBX, RawHook, Regs, SelfTest, rva};
use crate::marks::turns::*;
use crate::race::stage_prefs::left_of;
use crate::scene::vector3::{Pair80, pair_hook};
use crate::screens::harness::Left;
use crate::service::floats::{
    EXTENDED, Floats, ROUNDED, Tolerance, at_test_precisions, check_state_floats,
    dispatch_state_floats,
};
use crate::service::harness::Region;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);
static PAIR: Pair80 = Pair80::new();

pub static HOOK: RawHook =
    RawHook::new("marks::clamp_pair", rva(0x4BEF22), thunk, &ORIGINAL).self_test(self_test);

pair_hook!(
    thunk,
    call_original,
    original_with_pair,
    dispatch = dispatch,
    original = ORIGINAL,
    pair = PAIR
);

/// The converted `clamp_pair` (plain floats): the hook's. The x87 version stays for
/// `marks::make_corners` (unconverted).
// Converted: `rc2k_core::marks::turns::clamp_pair`
pub unsafe fn clamp_pair_plain(entry: &Regs, inputs: [[u8; 10]; 2]) -> (Regs, [[u8; 10]; 2]) {
    use rc2k_core::math::extended::{from_f64, to_f64};
    let mut g = *entry;
    let k: [u32; 4] = unsafe { crate::framework::game_read(PAIR_SCALES_RVA) };
    let (b, a, out) = rc2k_core::marks::turns::clamp_pair(
        to_f64(inputs[0]),
        to_f64(inputs[1]),
        k.map(f32::from_bits),
    );
    [g.ebx, g.eax] = [b, a];
    (g, out.map(from_f64))
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    let (g, outputs) = unsafe { clamp_pair_plain(r, PAIR.inputs()) };
    PAIR.set_outputs(outputs);
    Ok(left_of(&g))
}

/// The two values left on the stack (80 bits).
fn memory() -> [(Region, Tolerance); 1] {
    [(PAIR.output_region(), EXTENDED)]
}

/// The two whole numbers (`fistp`).
const REGS: [(u8, Tolerance); 2] = [(EAX, ROUNDED), (EBX, ROUNDED)];

extern "C" fn dispatch(r: &mut Regs) {
    let memory = memory();
    let floats = Floats {
        memory: &memory,
        regs: &REGS,
    };
    dispatch_state_floats(
        &HOOK,
        r,
        original_with_pair,
        &[PAIR.output_region()],
        &floats,
        &rust,
    );
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    use crate::scene::matrix3::extended;
    let special = [
        [extended(0.5f32.to_bits()), extended(200.0f32.to_bits())],
        [extended((-3.0f32).to_bits()), extended(1.0e9f32.to_bits())],
        [extended(f32::NAN.to_bits()), extended(127.99f32.to_bits())],
    ];
    let pairs = (1..=10u32).map(test_pair).chain(special);
    let memory = memory();
    let tolerant = Floats {
        memory: &memory,
        regs: &REGS,
    };
    let pairs: Vec<_> = pairs.collect();
    at_test_precisions(&tolerant, &mut |word, held| {
        for (n, &pair) in pairs.iter().enumerate() {
            check_state_floats(
                &mut t,
                "marks::clamp_pair",
                &format!("control word {word:#x}, pair {n}"),
                original_with_pair,
                Regs::SENTINEL,
                &|| {
                    PAIR.set_inputs(pair);
                    PAIR.set_outputs([[0xAA; 10]; 2]);
                },
                &[PAIR.output_region()],
                held,
                &rust,
            );
        }
    });
    t.finish()
}
