//! `race::course_grid` (0x45497E): the course records around the view, in a 32 x 32 grid
//! of record numbers at `dword_63E17C` (see `race/frame_draw.rs`).
//!
//! In:  nothing
//! Out: eax, ecx (and when the grid is made ebx the record count, esi the last record,
//!      edi past the grid) as it leaves them (a `call` follows)
//!
//! The half size is `race::hold_to_limit(dword_60A87C)` (`dword_71C6D4` is held to 4 .. 16
//! by `sub_4648B4`), the span twice that, so a span of 32 puts cells 0 .. 32 on each axis
//! into a grid 32 wide: column 32 lands on the next row's first, and row 32 past the grid
//! (onto `dword_63F17C` ..), as the original does (`notes/bugs.md`). Refuses (before
//! anything is written) no course records when the grid is to be made (the loop tests its
//! end after the first record). Shadow mode: by state (the grid and what lies past it to
//! 0x1084 bytes, the half, span, corner and the two cell numbers). Self-test: test records
//! (`g_courseRecords` pointed at them), corners, spans 8 and 32, the corner unchanged; all
//! put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{
    Region, at, capture, check_state, dispatch_state, restore, state_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::course_grid", rva(0x45497E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn regions() -> Vec<Region> {
    let mut v: Vec<Region> = [
        0x63E15Cusize,
        0x63E160,
        0x63E174,
        0x63E178,
        0x5CCBF0,
        0x5CCBF4,
    ]
    .iter()
    .map(|&a| at(rva(a)))
    .collect();
    v.push((game_addr(GRID_RVA) as u32, 0x1084));
    v
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { course_grid(r)? }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

const REACH: usize = rva(0x60A87C); // dword_60A87C
const LIMIT: usize = rva(0x71C6D4); // dword_71C6D4
const VIEW_X: usize = rva(0x6062B0); // dword_6062B0
const VIEW_Z: usize = rva(0x6062B4); // dword_6062B4
const RECORDS: usize = rva(0x5CCCB8); // g_courseRecords
const COUNT: usize = rva(0x5CCD18); // g_courseRecordCount

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let n = 200u32;
    let mut records = vec![0u8; 0x20 * n as usize];
    let base = records.as_mut_ptr() as u32;
    for k in 0..n {
        let cell = if k % 7 == 3 {
            u32::MAX
        } else {
            let x = (k * 13 + 1000) % 0x800;
            let z = (k * 29 + 990) % 0x800;
            x | z << 11 | (k & 1) << 30
        };
        unsafe { ((base + 0x20 * k) as *mut u32).write_unaligned(cell) };
    }
    let regions = regions();
    let mut kept = regions.clone();
    kept.extend([
        at(REACH),
        at(LIMIT),
        at(VIEW_X),
        at(VIEW_Z),
        at(RECORDS),
        at(COUNT),
    ]);
    let saved = capture(&kept);
    for (reach, limit, x, z, same) in [
        (4u32, 16u32, 1010u32, 1000u32, false),
        (16, 16, 1020, 1010, false),
        (20, 16, 1005, 1100, false),
        (4, 16, 1010, 1000, true),
        (2, 4, 0x7FF, 0, false),
    ] {
        check_state(
            &mut t,
            &format!("reach {reach} limit {limit} view {x} {z} same {same}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&kept, &saved);
                game_write(REACH, reach);
                game_write(LIMIT, limit);
                game_write(VIEW_X, x << 21);
                game_write(VIEW_Z, z << 21);
                game_write(RECORDS, base);
                game_write(COUNT, n);
                let half = reach.min(limit);
                let at = |v: u32| (((v << 21) as i32) >> 21) as u32;
                let corner = if same {
                    [at(x).wrapping_sub(half), at(z).wrapping_sub(half)]
                } else {
                    [0x1234, 0x5678]
                };
                game_write(rva(0x63E174), corner[0]);
                game_write(rva(0x63E178), corner[1]);
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    drop(records);
    t.finish()
}
