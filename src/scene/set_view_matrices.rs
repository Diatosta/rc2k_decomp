//! `scene::set_view_matrices` (0x438FBF): the view's matrices from its angles and place.
//!
//! In:  nothing
//! Out: eax, ebx = the place's x and y negated, ecx = edx = 0, esi = the view's
//!      turning, edi = that undone, ebp = the view's matrix
//!
//! `g_viewTurnMatrix` = the identity `unk_5E9F4C`; the fine turnings about y, x and z
//! for `dword_6062B8`, `dword_6062BC`, `dword_6062C0`, each negated; y times x to
//! `g_litMatrix` (used as work), that times z to `g_viewTurnMatrix`; `scene::set_place`
//! with `dword_606298` .. negated, and `g_placedMatrix` copied to `unk_5EA1CC`; the
//! placed matrix times the turning to `g_viewMatrix`; `scene::invert_turn` of the
//! turning to `flt_5EA20C`.
//! Two callers: `scene::prepare_menu_scene` and the front end's 3D set-up.
//! Self-test: 40 settings by lot on the game's matrices, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::scene::matrix::test_floats;
use crate::scene::view_matrix::{self, VIEW_ANGLES_RVA, VIEW_PLACE_RVA, view_regions};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::set_view_matrices", rva(0x438FBF), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- hook ----

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { view_matrix::set_view_matrices() })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &view_regions(), &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = view_regions();
    let inputs: [Region; 2] = [
        (game_addr(VIEW_ANGLES_RVA) as u32, 12),
        (game_addr(VIEW_PLACE_RVA) as u32, 12),
    ];
    let saved = (capture(&regions), capture(&inputs));
    let mut seed = 0x3141_5927u32;
    for case in 0..40 {
        let mut lot = || {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            seed
        };
        let angles = match case {
            0 => [0u32; 3],
            1 => [0x4000_0000, 0x8000_0000, 0xC000_0000],
            _ => [lot(), lot(), lot()],
        };
        let place = match case {
            0 => [0u32; 3],
            2 => [0x8000_0000, 0x7FFF_FFFF, 1],
            _ => [lot() >> 6, (lot() >> 9).wrapping_neg(), lot() >> 4],
        };
        // The matrices that keep some of their floats start from floats of the test's.
        let floats: [[u32; 16]; 9] = std::array::from_fn(|_| test_floats(&mut seed));
        check_state(
            &mut t,
            &format!("case {case}: angles {angles:x?}, place {place:x?}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&regions, &vec![0xAA; saved.0.len()]);
                for (region, floats) in regions.iter().zip(floats) {
                    (region.0 as *mut [u32; 16]).write_unaligned(floats);
                }
                game_write(VIEW_ANGLES_RVA, angles);
                game_write(VIEW_PLACE_RVA, place);
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
