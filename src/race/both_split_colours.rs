//! `race::both_split_colours` (0x45B147): `race::split_colours` for the first view's block
//! into `dword_6D160C`, then the second's into `dword_6D170C`.
//!
//! In:  nothing
//! Out: esi the second block, edi 0x6D170C, the rest as the second call leaves them (a
//!      `retn` follows)
//!
//! See `race/frame_draw.rs`. Caller: `race::set_up_panel` (0x45AB54). Refuses (before
//! anything) as `split_colours` would for either view. Shadow mode: by state (both
//! buffers). Self-test: the game's view blocks with counts and values set, fills; all put
//! back (skipped without view blocks).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::race::frame::VIEW_BLOCKS;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::both_split_colours", rva(0x45B147), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    COLOUR_BUFFERS
        .iter()
        .map(|&a| (game_addr(rva(a as usize)) as u32, 0x100))
        .collect()
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { both_split_colours(r)? }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let views: Vec<u32> = VIEW_BLOCKS
        .iter()
        .map(|&a| unsafe { game_read::<u32>(a) })
        .collect();
    if views.contains(&0) {
        return t.skip("no view blocks (no race loaded)");
    }
    let regions = regions();
    let mut kept = regions.clone();
    kept.push(at(MARK_COUNT_RVA));
    kept.extend(views.iter().map(|&b| (b + 0x90, 4)));
    kept.extend(views.iter().map(|&b| (b + 0x140, 0x100)));
    let saved = capture(&kept);
    for (fill, a, b) in [(0x20u32, 3u32, 0u32), (0x40, 0x40, 5), (0, 0xFFFF_FFFF, 1)] {
        check_state(
            &mut t,
            &format!("fill {fill:#x} counts {a:#x} {b:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                for &(r, len) in &regions {
                    std::ptr::write_bytes(r as *mut u8, 0xAA, len);
                }
                game_write(MARK_COUNT_RVA, fill);
                for (n, (&v, count)) in views.iter().zip([a, b]).enumerate() {
                    ((v + 0x90) as *mut u32).write_unaligned(count);
                    for k in 0..0x40u32 {
                        let value = (k * 37 + n as u32 * 11) % 0x70;
                        ((v + 0x140 + 4 * k) as *mut u32).write_unaligned(value);
                    }
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
