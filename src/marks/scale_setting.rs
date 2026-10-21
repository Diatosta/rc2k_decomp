//! `marks::scale_setting` (0x4C08BB): `flt_8BC25E` = the settings' whole number +0x2750 by
//! `flt_8BC39E` (`fild; fmul; fstp`).
//!
//! In:  nothing (`g_marksSettings`)
//! Out: esi the settings; the rest kept
//!
//! Caller: `sub_4C1148`. Refuses null settings (the original reads through 0). Self-test:
//! settings of the test's with several numbers, the original and the Rust version
//! compared on the result and esi; put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `scale_setting_plain`, through
//! `rc2k_core::marks::scales::scaled_setting` (plain floats; the x87 version stays for the
//! Rust callers in unconverted code). Shadow mode and the self-test compare within
//! `service::floats`' tolerances (the self-test at both test precisions, exact at double).

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::marks::lines::SETTINGS_RVA;
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::scene::matrix::float_places;
use crate::screens::harness::Left;
use crate::service::floats::*;
use crate::service::harness::{at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::scale_setting", rva(0x4C08BB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const NUMBER_AT: u32 = 0x2750;
const FACTOR_RVA: usize = rva(0x8BC39E); // flt_8BC39E
const RESULT_RVA: usize = rva(0x8BC25E); // flt_8BC25E

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn scale_setting(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        g.esi = game_read(SETTINGS_RVA);
        if g.esi == 0 {
            return Err(BadInput("no marks settings").into());
        }
        let n = ((g.esi + NUMBER_AT) as *const i32).read_unaligned();
        game_write(RESULT_RVA, x87::int_times(n, game_read(FACTOR_RVA)));
    }
    Ok(g)
}

/// The converted version (plain floats): the hook's.
// Converted: `rc2k_core::marks::scales::scaled_setting`
pub unsafe fn scale_setting_plain(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        g.esi = game_read(SETTINGS_RVA);
        if g.esi == 0 {
            return Err(BadInput("no marks settings").into());
        }
        let n = ((g.esi + NUMBER_AT) as *const i32).read_unaligned();
        let factor = f32::from_bits(game_read(FACTOR_RVA));
        let scaled = rc2k_core::marks::scales::scaled_setting(n, factor);
        game_write(RESULT_RVA, scaled.to_bits());
    }
    Ok(g)
}

/// What it writes.
pub fn regions() -> Vec<crate::service::harness::Region> {
    vec![at(RESULT_RVA)]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { scale_setting_plain(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    let places = float_places(&[at(RESULT_RVA)]);
    let floats = Floats {
        memory: &places,
        regs: &[],
    };
    dispatch_state_floats(&HOOK, r, call_original, &[at(RESULT_RVA)], &floats, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let vars = [at(SETTINGS_RVA), at(FACTOR_RVA), at(RESULT_RVA)];
    let saved = capture(&vars);
    let mut settings = vec![0u32; (NUMBER_AT as usize + 4) / 4];
    let s = settings.as_mut_ptr();
    let places = float_places(&[at(RESULT_RVA)]);
    let tolerant = Floats {
        memory: &places,
        regs: &[],
    };
    at_test_precisions(&tolerant, &mut |word, held| {
        for (n, factor) in [
            (0i32, 1.0f32),
            (1, 0.5),
            (-7, 3.25),
            (0x7FFF_FFFF, 1e-3),
            (i32::MIN, -2.0),
            (12345, 1.0 / 3.0),
        ] {
            check_state_floats(
                &mut t,
                "marks::scale_setting",
                &format!("control word {word:#x}, number {n}, factor {factor}"),
                call_original,
                Regs::SENTINEL,
                &|| unsafe {
                    game_write(SETTINGS_RVA, s as u32);
                    *s.add(NUMBER_AT as usize / 4) = n as u32;
                    game_write(FACTOR_RVA, factor.to_bits());
                    game_write(RESULT_RVA, 0xAAAA_AAAAu32);
                },
                &[at(RESULT_RVA)],
                held,
                &rust,
            );
        }
    });
    restore(&vars, &saved);
    drop(settings);
    t.finish()
}
