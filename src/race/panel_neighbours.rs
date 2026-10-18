//! `race::panel_neighbours` (0x45AD36): in mode 2 (`dword_62389C`), the first view's place
//! and the cars before and behind it into the panel (see `race/frame_draw.rs`): the place
//! (+0x654), +0x8C x 10, the car behind's +0x24 (-1 at the last place `dword_6051B4`) and
//! its label (`race::car_label` at 0x6D1838 of `dword_605034[place]`), +0x634, the car
//! ahead's +0x24 (-1 at place 1) and its label (0x6D1844, `dword_605030[place]`),
//! -(+0x630). The cars by place are `dword_604EC8[]` (place 0 first).
//!
//! In:  nothing
//! Out: in mode 2: eax -(+0x630), ebx the place, ecx the car ahead's value, esi the block,
//!      edi 0x6D1844, ebp the car ahead's block; else all kept (a `call` follows)
//!
//! Caller: `sub_45AB36` (0x45AB3B), each frame drawn. Refuses (before anything) a
//! neighbour's car number past 7 or a null block. Shadow mode: by state (the panel and the
//! two labels). Self-test: the game's blocks, the place tables set (places 1, 3, the last),
//! mode 2 and not; all put back (skipped with a null block); a car 8 refused.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::race::frame::VIEW_BLOCKS;
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::panel_neighbours", rva(0x45AD36), thunk, &ORIGINAL).self_test(self_test);

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

const MODE: usize = rva(0x62389C); // dword_62389C
const LAST: usize = rva(0x6051B4); // dword_6051B4
const BY_PLACE: usize = rva(0x604EC8); // dword_604EC8[]
const NUMBERS: usize = rva(0x605030); // dword_605030[]

fn regions() -> Vec<Region> {
    let mut v = panel_regions();
    v.extend([
        (game_addr(rva(0x6D1838)) as u32, 7),
        (game_addr(rva(0x6D1844)) as u32, 7),
    ]);
    v
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { panel_neighbours() }
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let blocks: Vec<u32> = (0..8)
        .map(|n| unsafe { game_read::<u32>(crate::race::objects::CAR_BLOCKS_RVA + 4 * n) })
        .collect();
    let views = view_blocks();
    if blocks.contains(&0) || views.contains(&0) {
        return t.skip("a block is null (no race loaded)");
    }
    let regions = regions();
    let mut kept = regions.clone();
    kept.extend([at(MODE), at(LAST)]);
    kept.extend([
        (game_addr(BY_PLACE) as u32, 0x30),
        (game_addr(NUMBERS) as u32, 0x30),
    ]);
    kept.extend(blocks.iter().map(|&b| (b + 0x24, 4)));
    kept.extend([
        (views[0] + 0x630, 8),
        (views[0] + 0x654, 4),
        (views[0] + 0x8C, 4),
    ]);
    let saved = capture(&kept);
    for (mode, place, last) in [
        (2u32, 3u32, 6u32),
        (2, 1, 6),
        (2, 6, 6),
        (1, 3, 6),
        (2, 0, 0),
    ] {
        check_state(
            &mut t,
            &format!("mode {mode} place {place} last {last}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                let w = |a: u32, v: u32| (a as *mut u32).write_unaligned(v);
                for &(a, len) in &regions {
                    std::ptr::write_bytes(a as *mut u8, 0xAA, len);
                }
                game_write(MODE, mode);
                game_write(LAST, last);
                for k in 0..12 {
                    game_write(BY_PLACE + 4 * k, (k as u32 * 3) % 8);
                    game_write(NUMBERS + 4 * k, 7 * k as u32 + 1);
                }
                for (n, &b) in blocks.iter().enumerate() {
                    w(b + 0x24, 0x100 + n as u32);
                }
                w(views[0] + 0x654, place);
                w(views[0] + 0x8C, 0x1234);
                w(views[0] + 0x630, 0x55);
                w(views[0] + 0x634, 0x66);
            },
            &regions,
            &rust,
        );
    }
    let refused = unsafe {
        game_write(MODE, 2u32);
        game_write(BY_PLACE + 4 * 4, 8u32);
        (views[0] as *mut u8)
            .add(0x654)
            .cast::<u32>()
            .write_unaligned(3);
        panel_neighbours().is_err()
    };
    restore(&kept, &saved);
    t.check(refused, || "car 8 not refused".into());
    t.finish()
}
