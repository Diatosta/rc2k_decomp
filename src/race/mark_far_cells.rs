//! `race::mark_far_cells` (0x454A5B): `g_courseGrid`'s cells with their top bits cleared,
//! then each cell turned by `g_viewTurnMatrix` and marked (bit 31) when it is out of sight
//! or too far (see `race/frame_draw.rs`; x87 throughout, compared as the original's
//! `fcomp` / `fnstsw` / `shr ah, 1`).
//!
//! In:  nothing
//! Out: eax and ecx the last cell's column and row less the half, ebx 0x400, esi
//!      `g_viewTurnMatrix`, edi past the grid (a `retn` follows)
//!
//! Writes `flt_63F180` / `flt_63F184` (the turned cell), `dword_62400C` (its length) and
//! `dword_624020` (`dword_63F17C` as a float) on the way. Caller:
//! `race::prepare_course_view` (0x4548DD). Shadow mode: by state. Self-test: the grid with
//! patterns (top bits set and not), halves 4 and 16, limits 16 and 3, on the game's view
//! turn; all put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::race::frame_draw::*;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::mark_far_cells", rva(0x454A5B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(all_regs(&unsafe { mark_far_cells(r) }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &far_regions(), &rust);
}

const HALF: usize = rva(0x63E15C); // dword_63E15C
const LIMIT: usize = rva(0x63F17C); // dword_63F17C

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = far_regions();
    let mut kept = regions.clone();
    kept.extend([at(HALF), at(LIMIT)]);
    let saved = capture(&kept);
    let grid = game_addr(GRID_RVA) as u32;
    for (half, limit, seed) in [(4u32, 16u32, 1u32), (16, 16, 7), (16, 3, 0x9E37), (8, 0, 3)] {
        check_state(
            &mut t,
            &format!("half {half} limit {limit} seed {seed:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&kept, &saved);
                game_write(HALF, half);
                game_write(LIMIT, limit);
                for k in 0..0x400u32 {
                    let v = k.wrapping_mul(seed).rotate_left(k % 31) ^ (k << 28);
                    ((grid + 4 * k) as *mut u32).write_unaligned(v);
                }
            },
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
