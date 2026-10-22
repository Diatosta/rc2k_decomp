//! `race::both_countdown_lights` (0x45AFDB): `race::countdown_lights` for the first view
//! (edi 0, flag `dword_6D1830`) and the second (edi 0x10, `dword_6D1834`).
//!
//! In:  nothing
//! Out: edi 0x10, esi the second block, ebp 0x6D1834, the rest as the second call leaves
//!      them (a `call` follows)
//!
//! See `race/frame_draw.rs`. Caller: `sub_45AB36` (0x45AB4F), each frame drawn. Shadow
//! mode: by state. Self-test: the game's view blocks' +0x14/+0x1C set for times around the
//! steps; all put back (skipped without view blocks).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::race::frame::VIEW_BLOCKS;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::both_countdown_lights",
    rva(0x45AFDB),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn view_blocks() -> Vec<u32> {
    VIEW_BLOCKS
        .iter()
        .map(|&a| unsafe { game_read::<u32>(a) })
        .collect()
}

fn regions() -> Vec<Region> {
    let mut v = light_regions();
    v.extend([
        (game_addr(rva(0x6D1830)) as u32, 4),
        (game_addr(rva(0x6D1834)) as u32, 4),
    ]);
    v
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { both_countdown_lights(r) }))
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
    let views = view_blocks();
    if views.contains(&0) {
        return t.skip("no view blocks (no race loaded)");
    }
    let regions = regions();
    let mut kept = regions.clone();
    kept.push(at(LIGHTS_START));
    kept.extend(views.iter().map(|&b| (b + 0x14, 0xC)));
    let saved = capture(&kept);
    for (a, b, held) in [
        (0u32, 600u32, 0u32),
        (300, 500, 0),
        (1500, 0xFFFF_FFFF, 0),
        (550, 100, 1),
    ] {
        check_state(
            &mut t,
            &format!("times {a} {b} held {held}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                for &(r, len) in &regions {
                    std::ptr::write_bytes(r as *mut u8, 0xAA, len);
                }
                game_write(LIGHTS_START, 2000u32);
                for (&v, time) in views.iter().zip([a, b]) {
                    ((v + 0x14) as *mut u32).write_unaligned(time.wrapping_add(2000));
                    ((v + 0x1C) as *mut u32).write_unaligned(held);
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
