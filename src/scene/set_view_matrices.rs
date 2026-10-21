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
//!
//! Converted 2026-10-21: the hook's Rust version is `set_view_matrices_plain`, through
//! `rc2k_core::scene::matrices::set_view_matrices` (plain floats; the x87 version stays for
//! the Rust callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::scene::matrix::{sum_places, test_floats};
use crate::scene::view_matrix::{self, VIEW_ANGLES_RVA, VIEW_PLACE_RVA, view_regions};
use crate::screens::harness::Left;
use crate::service::floats::*;
use crate::service::harness::{Region, capture, restore};
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
    Ok(unsafe { view_matrix::set_view_matrices_plain() })
}

extern "C" fn dispatch(r: &mut Regs) {
    let places = sum_places(&view_regions());
    let floats = Floats {
        memory: &places,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &view_regions(), &floats, &rust);
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
    let tolerant_places = sum_places(&regions);
    let tolerant = Floats {
        memory: &tolerant_places,
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, held| {
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
            check_state_floats(
                &mut t,
                "scene::set_view_matrices",
                &format!(
                    "control word {word:#x}, case {case}: angles {angles:x?}, place {place:x?}"
                ),
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
                held,
                &rust,
            );
        }
    });
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
