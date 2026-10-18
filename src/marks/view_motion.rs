//! `marks::view_motion` (0x4C4D40): the view's motion since the last frame. With the
//! view matrix `flt_5EA3CC`: its third row (+0x20 ..) kept on the stack; its place (the
//! origin turned by it, `scene::rotate_point`, unrounded) less the last (`flt_91FFBE` ..)
//! into `flt_8EB951` .. and the stack; that step made length 1
//! (`scene::normalise_vector3`); `flt_8F397D` the third row's dot product with it; the
//! place (turned again) kept as the last; then `flt_8EB951` .. each over
//! `g_marksFrameTicks` (`fidiv`).
//!
//! In:  nothing
//! Out: esi the view matrix; edi and ebp its stack scratch (not compared); the rest kept
//!
//! Caller: `sub_4C3EEE`. Shadow mode and the self-test set edi and ebp back to the
//! caller's on both sides (the original's are addresses on its stack). Self-test: view
//! matrices, last places and ticks by lot; compared on the seven results.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::math::x87;
use crate::race::stage_prefs::left_of;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;
use crate::scene::vector3::{dot_product3, normalise_vector3};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("marks::view_motion", rva(0x4C4D40), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const VIEW_RVA: usize = rva(0x5EA3CC); // flt_5EA3CC: the view matrix
const LAST_RVA: usize = rva(0x91FFBE); // flt_91FFBE .. flt_91FFC6
const STEP_RVA: usize = rva(0x8EB951); // flt_8EB951 .. flt_8EB959
const FACING_RVA: usize = rva(0x8F397D); // flt_8F397D
const TICKS_RVA: usize = rva(0x764FD0); // g_marksFrameTicks

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn view_motion(entry: &Regs) -> Regs {
    let mut g = *entry;
    unsafe {
        let m = game_addr(VIEW_RVA) as u32;
        let get = |a: u32| (a as *const u32).read_unaligned();
        let mut local = [0u32; 6];
        for (k, v) in local.iter_mut().take(3).enumerate() {
            *v = x87::loaded(get(m + 0x20 + 4 * k as u32));
        }
        let zero = [extended(0); 3];
        let place = rotate_point(m, zero);
        for k in 0..3 {
            let step = x87::ext_less(place[k], game_read(LAST_RVA + 4 * k));
            game_write(STEP_RVA + 4 * k, step);
            local[3 + k] = step;
        }
        let at = local.as_mut_ptr() as u32;
        normalise_vector3(at + 12);
        game_write(FACING_RVA, rounded(dot_product3(at, at + 12)));
        let place = rotate_point(m, zero);
        for (k, p) in place.iter().enumerate() {
            game_write(LAST_RVA + 4 * k, rounded(*p));
        }
        let ticks: i32 = game_read(TICKS_RVA);
        for k in 0..3 {
            let a = STEP_RVA + 4 * k;
            game_write(a, x87::over_int(game_read(a), ticks));
        }
        g.esi = m;
    }
    g
}

pub fn regions() -> Vec<Region> {
    vec![
        (game_addr(STEP_RVA) as u32, 12),
        at(FACING_RVA),
        (game_addr(LAST_RVA) as u32, 12),
    ]
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { view_motion(r) }))
}

/// The original, its edi and ebp (stack addresses) made the caller's again.
unsafe extern "C" fn original_without_scratch(r: *mut Regs) {
    unsafe {
        let (edi, ebp) = ((*r).edi, (*r).ebp);
        call_original(r);
        (*r).edi = edi;
        (*r).ebp = ebp;
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, original_without_scratch, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let mut vars = regions.clone();
    vars.push((game_addr(VIEW_RVA) as u32, 0x40));
    vars.push(at(TICKS_RVA));
    let saved = capture(&vars);
    for case in 0..8u32 {
        check_state(
            &mut t,
            &format!("case {case}"),
            original_without_scratch,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&vars, &saved);
                for k in 0..0x10 {
                    let v = ((k as f32 - 6.0) * 0.31 + case as f32 * 1.7)
                        * if k >= 12 { 40.0 } else { 1.0 };
                    game_write(VIEW_RVA + 4 * k, v.to_bits());
                }
                for k in 0..3 {
                    game_write(LAST_RVA + 4 * k, (k as f32 * 3.0 - case as f32).to_bits());
                }
                game_write(TICKS_RVA, case as i32 % 4);
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
