//! `marks::aim_vector` (0x4C1294): for each of x, y, z `dword_769306` .. =
//! -`flt_7692B2` .. less `flt_76927A` .. plus `flt_7692CE` .. plus `flt_7692EA` ..; then
//! that vector made of length 1 into `unk_7692FA` .., its length (`fst`) in
//! `dword_7692F6` (`x87::unit_and_length`).
//!
//! In:  nothing
//! Out: esi the vector (0x769306), edi the unit vector (0x7692FA); the rest kept
//!
//! Caller: `sub_4C1270`. Self-test: the twelve inputs by lot, compared on the seven
//! results and the registers; put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::marks::leaf_test::test_leaf;
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::aim_vector", rva(0x4C1294), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// Per axis: (negated, less, plus, plus).
const TERMS: [[usize; 4]; 3] = [
    [rva(0x7692B2), rva(0x76927A), rva(0x7692CE), rva(0x7692EA)],
    [rva(0x7692B6), rva(0x76927E), rva(0x7692D2), rva(0x7692EE)],
    [rva(0x7692BA), rva(0x769282), rva(0x7692D6), rva(0x7692F2)],
];
const VECTOR_RVA: usize = rva(0x769306); // dword_769306 .. dword_76930E
const UNIT_RVA: usize = rva(0x7692FA); // unk_7692FA
const LENGTH_RVA: usize = rva(0x7692F6); // dword_7692F6

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn aim_vector(entry: &Regs) -> Regs {
    unsafe {
        let v: [u32; 3] = std::array::from_fn(|k| {
            let t = TERMS[k].map(|a| game_read::<u32>(a));
            x87::negated_less_plus_plus(t[0], t[1], t[2], t[3])
        });
        game_write(VECTOR_RVA, v);
        let (unit, length) = x87::unit_and_length(v);
        game_write(LENGTH_RVA, length);
        game_write(UNIT_RVA, unit);
        Regs {
            esi: game_addr(VECTOR_RVA) as u32,
            edi: game_addr(UNIT_RVA) as u32,
            ..*entry
        }
    }
}

fn written() -> Vec<usize> {
    let mut w: Vec<usize> = (0..3).map(|k| VECTOR_RVA + 4 * k).collect();
    w.extend((0..3).map(|k| UNIT_RVA + 4 * k));
    w.push(LENGTH_RVA);
    w
}

/// What it writes.
pub fn regions() -> Vec<Region> {
    written().into_iter().map(at).collect()
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { aim_vector(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions: Vec<Region> = written().into_iter().map(at).collect();
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let inputs: Vec<usize> = TERMS.iter().flatten().copied().collect();
    test_leaf(
        &mut t,
        &inputs,
        &written(),
        Regs::SENTINEL,
        10,
        call_original,
        &rust,
    );
    t.finish()
}
