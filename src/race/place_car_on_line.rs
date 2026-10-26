//! `race::place_car_on_line` (0x44666F): the work car set between checkpoint
//! `dword_6365E4` and the next, `dword_6365EC` / 65536 of the way: their faces' first
//! points blended into `dword_5CCDA0` / `C0` (+ `dword_613FC4`) / `E0`, their headings
//! (`dword_9ED808`) into `dword_5CCD60`, then the car's x, z, heading and y (+0x134,
//! +0x13C, +0x9C0, +0x138); the first face's number in `dword_5CCD28`. Both checkpoints
//! held below `g_checkpointCount` (unsigned).
//!
//! In:  nothing
//! Out: eax = y, ecx = z, edx = the heading product's high half, ebx and esi = the second
//!      point's number and address, ebp = the first point's address; edi preserved
//!
//! See `race/start_place.rs`. Callers: `sub_4464AD` (race init's camera start) and four
//! more (0x429479, 0x437687, 0x43F0CA, 0x43FB1E). Refuses no checkpoints (the original
//! reads entry -1) or more than the lists hold. Shadow mode: the Rust version, the
//! variables put back, the original's, compared. Self-test: faces, points and checkpoints
//! of the test's (`TestLine`), headings of the test's in the game's table, places and
//! fractions inside and past the line; the refusal on the Rust version alone; all put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::best_line::{CHECK_COUNT_RVA, TestLine};
use crate::race::start_place::*;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::place_car_on_line", rva(0x44666F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    let g = unsafe { place_car_on_line(r) }?;
    Ok(left_changed(r, &g))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &place_regions(), &rust);
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
    let regions = place_regions();
    let headings = (game_addr(LINE_HEADINGS_RVA) as u32, 4 * 32);
    let mut kept = regions.clone();
    kept.extend([
        headings,
        at(LINE_PLACE_RVA),
        at(LINE_FRACTION_RVA),
        at(DROP_RVA),
    ]);
    let saved = capture(&kept);
    let line = TestLine::new(32, 30, &|k| (k * 7 + 1) % 32, &|_, f| f.fill(0x3C));
    let table: Vec<u8> = (0..32u32)
        .flat_map(|k| (k.wrapping_mul(0x9E37_79B9) >> 8).to_le_bytes())
        .collect();
    for (count, place, fraction, drop) in [
        (30u32, 0u32, 0u32, 0u32),
        (30, 0, 0x8000, 0x5_0000),
        (30, 5, 0x1_0000, 0),
        (30, 12, 0xFFFF_C000, 0xFFFF_0000),
        (30, 28, 0x4321, 7),
        (30, 29, 0x8000, 0),
        (30, 30, 0x8000, 0),
        (30, 0xFFFF_FFFF, 0x7FFF_FFFF, 0),
        (30, 7, 0x8000_0000, 0),
        (1, 0, 0x8000, 0),
        (1, 9, 0x8000, 0),
    ] {
        check_state(
            &mut t,
            &format!("count {count}, place {place:#x}, fraction {fraction:#x}, drop {drop:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&regions, &vec![0x5A; 4 * regions.len()]);
                restore(&[headings], &table);
                game_write(CHECK_COUNT_RVA, count);
                game_write(LINE_PLACE_RVA, place);
                game_write(LINE_FRACTION_RVA, fraction);
                game_write(DROP_RVA, drop);
            },
            &regions,
            &rust,
        );
    }
    unsafe {
        game_write(CHECK_COUNT_RVA, 0u32);
        let refused = place_car_on_line(&Regs::SENTINEL).is_err();
        t.check(refused, || "no checkpoints: not refused".to_string());
    }
    drop(line);
    restore(&kept, &saved);
    t.finish()
}
