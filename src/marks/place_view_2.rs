//! `marks::place_view_2` (0x4C2AA5): `marks::place_view` on other variables (the point
//! (0, 0, 0x80000), the matrix at `dword_8D216C`, the place `dword_8D2160`, the height
//! `flt_8DB53C`, out at `dword_8D2214`, the copy at `dword_8D2220`), then `dword_8D20C0`
//! .. `dword_8D20DC` 0 and the four floats at `xmmword_8D20E0` 0x80000.
//!
//! In:  nothing
//! Out: eax 0; the rest as `scene::transform_int_points` leaves them
//!
//! See `marks/place_view.rs`; no variable shared with it. One caller, `sub_4C25F6`.
//! Self-test: as `place_view`'s, the two blocks also watched.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::marks::place_view::{ViewPlace, WRITTEN_LEN, place_view_with, test_with};
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::place_view_2", rva(0x4C2AA5), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const SECOND: ViewPlace = ViewPlace {
    point: rva(0x8D2208), // dword_8D2208
    start_z: 0x8_0000,
    place: rva(0x8D2160),       // dword_8D2160
    matrix: rva(0x8D216C),      // dword_8D216C
    out: rva(0x8D2214),         // dword_8D2214
    height: rva(0x8DB53C),      // flt_8DB53C
    matrix_copy: rva(0x8D2220), // dword_8D2220
};
const CLEARED_RVA: usize = rva(0x8D20C0); // dword_8D20C0 .. dword_8D20DC
const FILLED_RVA: usize = rva(0x8D20E0); // xmmword_8D20E0: four floats

/// The Rust version (see the header).
pub unsafe fn place_view_2(entry: &Regs) -> Result<Regs, Refused> {
    unsafe {
        let mut g = place_view_with(entry, &SECOND)?;
        g.eax = 0;
        game_write(CLEARED_RVA, [0u32; 8]);
        let z = x87::from_int(game_read(SECOND.point + 8));
        game_write(FILLED_RVA, [z; 4]);
        Ok(g)
    }
}

fn extra() -> [Region; 1] {
    [(game_addr(CLEARED_RVA) as u32, 0x30)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { place_view_2(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let mut regions = vec![(game_addr(SECOND.point) as u32, WRITTEN_LEN)];
    regions.extend(extra());
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    test_with(&mut t, &SECOND, call_original, &extra(), &rust);
    t.finish()
}
