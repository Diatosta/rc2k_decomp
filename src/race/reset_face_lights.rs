//! `race::reset_face_lights` (0x45ED70): the lights of the faces marked 0xFF cleared, then
//! `race::clear_lit_faces`.
//!
//! In:  nothing
//! Out: ebx the face count, esi the last face; the rest preserved
//!
//! See `race/course_data.rs`. Callers `sub_4455FC` and `sub_4462B9` (race init). Refuses no
//! faces or 0x100000 or more. Self-test: faces of the test's in the game's pointer and
//! count, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, hook_asm, rva};
use crate::race::course_data::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, check_state, dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::reset_face_lights", rva(0x45ED70), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    let count: u32 = unsafe { game_read(FACE_COUNT_RVA) };
    let faces: u32 = unsafe { game_read(FACES_RVA) };
    let len = if count < MOST_FACES {
        count as usize * 0x40
    } else {
        0
    };
    vec![(faces, len), at(LIT_COUNT_RVA)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(crate::race::stage_prefs::left_of(&unsafe {
        reset_face_lights(r)
    }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let lit = [at(LIT_COUNT_RVA)];
    let saved = crate::service::harness::capture(&lit);
    for count in [1u32, 2, 7, 40] {
        let faces = TestTable::new(FACES_RVA, FACE_COUNT_RVA, 0x40, count, 41, &|n, f| {
            for (k, b) in f.iter_mut().enumerate() {
                *b = (n * 13 + k * 7) as u8 | 0x80;
            }
            f[0] = if n % 3 == 0 { 0xFF } else { 0xFE };
        });
        let mut regions = vec![faces.region()];
        regions.extend(lit);
        check_state(
            &mut t,
            &format!("{count} faces"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                faces.reset();
                crate::framework::game_write(LIT_COUNT_RVA, 9u32);
            },
            &regions,
            &rust,
        );
    }
    crate::service::harness::restore(&lit, &saved);
    let refused = {
        let _faces = TestTable::new(FACES_RVA, FACE_COUNT_RVA, 0x40, 0, 1, &|_, _| {});
        unsafe { reset_face_lights(&Regs::SENTINEL) }.is_err()
    };
    t.check(refused, || "no faces: not refused".into());
    t.finish()
}
