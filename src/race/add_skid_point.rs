//! `race::add_skid_point` (0x45860C): a wheel's next skid point across its place at the
//! angle from its last, linked into the strip, the last turned to match (see
//! `race/skid_laying.rs`).
//!
//! In:  esi = the wheel's slot (its last point; -1 none), `dword_643AD8` .. its place
//! Out: edx the new point, ebp the last, esi / eax / ebx / ecx as it leaves them (inside its
//!      caller's `pusha`)
//!
//! Callers: `race::lay_skidmarks` (four). Shadow mode: by state. Self-test: test strips in
//! the game's ring (put back): no last point, a last that starts its strip, one inside a
//! strip, the ring wrapping, a reused point with links.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::frame_draw::all_regs;
use crate::race::skid_laying::add_skid_point;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::add_skid_point", rva(0x45860C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    // Converted: `rc2k_core::race::skids::add_skid_point`.
    Ok(all_regs(&unsafe { add_skid_point(r)? }))
}

fn regions(slot: u32) -> Vec<Region> {
    let g = |a: usize, len: usize| (game_addr(rva(a)) as u32, len);
    vec![
        g(0x645B18, 0x400 * 0x18),
        g(0x643AE8, 0x400 * 8),
        g(0x645AE8, 0x30),
        g(0x5C5F4C, 0xC),
        at(rva(0x62400C)),
        (slot, 4),
    ]
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(r.esi), &rust);
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
    let mut slot = vec![0u32; 1];
    let s = slot.as_mut_ptr() as u32;
    let regions = regions(s);
    let mut kept = regions.clone();
    kept.extend([
        (game_addr(rva(0x643AD8)) as u32, 0xC),
        (game_addr(rva(0x645B10)) as u32, 8),
    ]);
    let saved = capture(&kept);
    let pts = game_addr(rva(0x645B18)) as u32;
    let links = game_addr(rva(0x643AE8)) as u32;
    for (last, next, prev_of_last) in [
        (u32::MAX, 5u32, 0xFFFFu16),
        (7, 8, 0xFFFF),
        (9, 10, 3),
        (0x3FE, 0x3FF, 0x3FD),
        (2, 4, 1),
    ] {
        check_state(
            &mut t,
            &format!("last {last:#x} next {next:#x} its previous {prev_of_last:#x}"),
            call_original,
            Regs {
                esi: s,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                restore(&kept, &saved);
                (s as *mut u32).write(last);
                game_write(rva(0x645B10), next);
                game_write(rva(0x645B14), 4u32);
                game_write(rva(0x643AD8), [0x12_3400u32, 0x5600, 0x78_9A00]);
                for n in 0..0x400u32 {
                    let p = pts + 0x18 * n;
                    for i in 0..6u32 {
                        ((p + 4 * i) as *mut u32)
                            .write_unaligned(n.wrapping_mul(0x3_1337) ^ (i << 12));
                    }
                    let l = links + 8 * n;
                    (l as *mut u16).write_unaligned(if n % 5 == 0 {
                        0xFFFF
                    } else {
                        (n as u16 + 1) & 0x3FF
                    });
                    ((l + 2) as *mut u16).write_unaligned(if n % 4 == 0 {
                        0xFFFF
                    } else {
                        (n as u16).wrapping_sub(1) & 0x3FF
                    });
                    ((l + 4) as *mut u16).write_unaligned((n * 0x97) as u16);
                }
                if last != u32::MAX {
                    ((links + 8 * last + 2) as *mut u16).write_unaligned(prev_of_last);
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    drop(slot);
    t.finish()
}
