//! `scene::choose_view_window` (0x44FE1A): the view's window, then `scene::set_view_window`:
//! without `dword_621514` the screen's size (`g_screenWidth`, `g_screenHeight`) at 0, 0;
//! with it 800 x 600 at 0, 0 when `dword_621480` is set or `word_636526` is 2, else 720 x
//! 540 at 0x50, 0; `dword_624080` 0x100 either way.
//!
//! In:  nothing
//! Out: as `set_view_window` leaves them
//!
//! Callers: `scene::full_view_window`, code at 0x446810. `set_view_window` is its Rust
//! version. Self-test: the original and the Rust version on the game's variables, every
//! way.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::choose_view_window", rva(0x44FE1A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const FRAMED_RVA: usize = rva(0x621514); // dword_621514
const WIDE_RVA: usize = rva(0x621480); // dword_621480
pub const MODE_RVA: usize = rva(0x636526); // word_636526
const WINDOW_RVA: [usize; 2] = [rva(0x62408C), rva(0x624090)];
const OFFSET_RVA: [usize; 2] = [rva(0x623FB0), rva(0x623FB4)];
const SCALE_RVA: usize = rva(0x624080); // dword_624080
const SCREEN_RVA: [usize; 2] = [rva(0x4F494C), rva(0x4F4950)];

/// The Rust version (see the header): what it leaves in the registers.
pub unsafe fn choose_view_window() -> Left {
    unsafe {
        let (w, h, x) = if game_read::<u32>(FRAMED_RVA) == 0 {
            (game_read(SCREEN_RVA[0]), game_read(SCREEN_RVA[1]), 0)
        } else if game_read::<u32>(WIDE_RVA) != 0 || game_read::<u16>(MODE_RVA) == 2 {
            (0x320, 0x258, 0)
        } else {
            (0x2D0, 0x21C, 0x50)
        };
        game_write(WINDOW_RVA[0], w);
        game_write(WINDOW_RVA[1], h);
        game_write(OFFSET_RVA[0], x);
        game_write(OFFSET_RVA[1], 0u32);
        game_write(SCALE_RVA, 0x100u32);
        crate::scene::set_view_window::set_view_window()
    }
}

pub fn regions() -> Vec<Region> {
    let mut r: Vec<Region> = crate::scene::set_view_window::regions().to_vec();
    r.extend(
        [
            WINDOW_RVA[0],
            WINDOW_RVA[1],
            OFFSET_RVA[0],
            OFFSET_RVA[1],
            SCALE_RVA,
        ]
        .map(at),
    );
    r
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { choose_view_window() })
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let mut vars = regions.clone();
    vars.extend(crate::scene::set_view_window::inputs());
    vars.extend([FRAMED_RVA, WIDE_RVA, MODE_RVA].map(at));
    let saved = capture(&vars);
    for case in 0..6u32 {
        check_state(
            &mut t,
            &format!("case {case}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&vars, &saved);
                game_write(FRAMED_RVA, u32::from(case != 0));
                game_write(WIDE_RVA, u32::from(case == 2));
                game_write(MODE_RVA, [0u16, 0, 0, 2, 1, 3][case as usize]);
            },
            &regions,
            &rust,
        );
    }
    restore(&vars, &saved);
    t.finish()
}
