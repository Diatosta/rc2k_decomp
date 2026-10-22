//! `scene::angle_between` (0x450223): the direction from one place to another as an
//! angle of sixteen bits a turn.
//!
//! In:  eax, ebx = the first place, ecx, edx = the second
//! Out: eax = the angle; ecx, edx = the two differences; everything else preserved
//!
//! `fpatan` of (ecx - eax) over (edx - ebx), over pi, times `flt_5C5E40` (0.5) and
//! `flt_5C5D98` (65536), rounded (`fistp`), its low sixteen bits; the result is also
//! left in `dword_62400C`. The instructions themselves, as `fpatan`'s result is the
//! processor's.
//! Four callers.
//! Self-test: 300 pairs of places.

use crate::errors::Refused;
use crate::framework::{
    EAX, ECX, EDX, RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva,
};
use crate::math::x87;
use crate::scene::draw_polygon::CHECKED_RVA;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::angle_between", rva(0x450223), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const HALF_RVA: usize = rva(0x5C5E40); // flt_5C5E40, 0.5
const TURN_RVA: usize = rva(0x5C5D98); // flt_5C5D98, 65536.0

// ---- Rust version ----

/// The angle of the two differences.
pub unsafe fn angle_of(first: u32, second: u32) -> u32 {
    unsafe {
        let angle = x87::angle_scaled_to_int(
            first as i32,
            second as i32,
            game_read(HALF_RVA),
            game_read(TURN_RVA),
        ) as u32
            & 0xFFFF;
        game_write(CHECKED_RVA, angle);
        angle
    }
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    let (first, second) = (r.ecx.wrapping_sub(r.eax), r.edx.wrapping_sub(r.ebx));
    Ok(Left::nothing()
        .with(EAX, unsafe { angle_of(first, second) })
        .with(ECX, first)
        .with(EDX, second))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[at(CHECKED_RVA)], &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions: [Region; 1] = [at(CHECKED_RVA)];
    let saved = capture(&regions);
    let mut seed = 0x6A09_E667u32;
    let mut lot = || {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        seed
    };
    for case in 0..300u32 {
        let places = match case {
            0 => [0u32; 4],
            1 => [0, 0, 0, 0x100],
            2 => [0, 0, 0x100, 0],
            3 => [0, 0, 0xFFFF_FF00, 0],
            4 => [0, 0, 0, 0xFFFF_FF00],
            5 => [0x7FFF_FFFF, 0, 0x8000_0000, 1],
            6 => [5, 5, 0xFFFF_FFFB, 0xFFFF_FFFB],
            n if n < 150 => [lot() >> 12, lot() >> 12, lot() >> 12, lot() >> 12],
            _ => [lot(), lot(), lot(), lot()],
        };
        check_state(
            &mut t,
            &format!("places {places:x?}"),
            call_original,
            Regs {
                eax: places[0],
                ebx: places[1],
                ecx: places[2],
                edx: places[3],
                ..Regs::SENTINEL
            },
            &|| unsafe { game_write(CHECKED_RVA, 0xAAAA_AAAAu32) },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
