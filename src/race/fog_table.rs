//! `race::fog_table` (0x44FC33): the fog's table for the frame (see `race/frame_draw.rs`):
//! `race::draw_distance_settings`, the noise at the car (`race::sample_noise` of
//! `g_noiseBytes`) spread into a step (`dword_62DAF0`), two shares from `dword_6415DC` x
//! `dword_6415D8` (`dword_6365F0` / `dword_6365F4`), `g_fogTableLength` = `dword_63DC58` x 8
//! bytes of `g_fogTable` by the summed step, and `dword_636610` the value past it.
//!
//! In:  nothing
//! Out: the registers as it leaves them (a `mov` follows in `sub_44FBD9`)
//!
//! It begins with its call: private copy (0x105, both calls relocated), manual patch.
//! Caller: `sub_44FBD9` (0x44FC04). Refuses (before anything) a length of 0 (the loop tests
//! at its end) or past `g_fogTable`'s 0x1000 bytes. Shadow mode: by state. Self-test: the
//! game's tables and noise, the inputs set (levels, the car's place, the shares, lengths
//! 0x10 .. 0x1000); all put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::draw_distance_settings::distance_regions;
use crate::race::frame_draw::*;
use crate::race::sample_noise::noise_regions;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("race::fog_table", rva(0x44FC33), thunk, &ORIGINAL)
    .private_copy(0x105)
    .relocate_calls(&[0x00, 0x0B])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn fog_regions() -> Vec<Region> {
    let mut v = distance_regions();
    v.extend(noise_regions());
    v.extend([
        (game_addr(FOG_TABLE_RVA) as u32, 0x1000),
        at(rva(0x63761C)),
        at(rva(0x62DAF0)),
        at(rva(0x6365F0)),
        at(rva(0x6365F4)),
        at(rva(0x636610)),
    ]);
    v
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { fog_table(r)? }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &fog_regions(), &rust);
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
    let regions = fog_regions();
    let car = game_addr(rva(0x71A67C)) as u32;
    let mut kept = regions.clone();
    for a in [0x60AC84usize, 0x6415DC, 0x6415D8, 0x63DC58] {
        kept.push(at(rva(a)));
    }
    kept.extend([(car, 4), (car + 8, 4)]);
    let saved = capture(&kept);
    for (level, length, shares, place) in [
        (0u32, 0x02u32, (0x80u32, 0x100u32), 0x1234_5678u32),
        (2, 0x20, (0x40, 0x40), 0x8765_4321),
        (4, 0x200, (0x10, 0x3000), 0x0F0F_0F0F),
        (3, 0xC0, (0, 0x77), 0xFFFF_FFFF),
    ] {
        check_state(
            &mut t,
            &format!("level {level} length {length:#x} shares {shares:x?} place {place:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&kept, &saved);
                game_write(rva(0x60AC84), level);
                game_write(rva(0x63DC58), length);
                game_write(rva(0x6415DC), shares.0);
                game_write(rva(0x6415D8), shares.1);
                (car as *mut u32).write_unaligned(place);
                ((car + 8) as *mut u32).write_unaligned(place.rotate_left(11));
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
