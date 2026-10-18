//! `physics::sound_on_bit` (0x441324): when bit cl of `g_car` +0x298 (`unk_71A270`) is
//! set, it is cleared and `physics::place_car_sound` runs.
//!
//! In:  cl the bit
//! Out: eax 1 << cl; the rest kept; no flags read (the callers load floats next)
//!
//! Callers: `sub_4411EC` (four).
//! Refuses what `place_car_sound` refuses (before anything: the bit is cleared after the
//! check).
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back) with a test sound object: bits set and clear.

use crate::errors::Refused;
use crate::framework::{
    EAX, RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva,
};
use crate::physics::car_sounds::{object, test_regions};
use crate::physics::place_car_sound::place_car_sound;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::sound_on_bit", rva(0x441324), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const BITS_RVA: usize = rva(0x71A270); // g_car + 0x298

/// The Rust version.
pub unsafe fn sound_on_bit(cl: u32) -> Result<Left, Refused> {
    let bit = 1u32.wrapping_shl(cl & 0x1F);
    let left = Left::nothing().with(EAX, bit);
    let bits: u32 = unsafe { game_read(BITS_RVA) };
    if bits & bit == 0 {
        return Ok(left);
    }
    object()?;
    unsafe { game_write(BITS_RVA, bits ^ bit) };
    unsafe { place_car_sound() }?;
    Ok(left)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { sound_on_bit(r.ecx & 0xFF) }
}

extern "C" fn dispatch(r: &mut Regs) {
    let mut regions = vec![(game_addr(BITS_RVA) as u32, 4)];
    if let Ok(o) = object() {
        regions.extend([(o + 0x10, 4), (o + 0x214, 0x14)]);
    }
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut cells = vec![0u32; 0x98];
    let o = cells.as_mut_ptr() as u32;
    let mut all = test_regions(o);
    all.push((game_addr(BITS_RVA) as u32, 4));
    let saved = capture(&all);
    for (cl, bits) in [(0u32, 1u32), (0, 2), (3, 0xFF), (31, 0x8000_0000), (5, 0)] {
        let set = || unsafe {
            for k in 0..0x98u32 {
                ((o + 4 * k) as *mut u32).write(0xA5A5_0000 | k);
            }
            game_write(rva(0x71B3F8), cl % 8);
            for k in 0..8usize {
                game_write(rva(0x512A10 + 4 * k), o);
            }
            game_write(BITS_RVA, bits);
        };
        check_state(
            &mut t,
            &format!("bit {cl}, bits {bits:#x}"),
            call_original,
            Regs {
                ecx: 0xAB00 | cl,
                ..Regs::SENTINEL
            },
            &set,
            &all,
            &rust,
        );
    }
    restore(&all, &saved);
    drop(cells);
    t.finish()
}
