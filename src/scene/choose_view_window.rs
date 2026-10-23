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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, state_entry};

const FRAMED_RVA: usize = rva(0x621514); // dword_621514
const WIDE_RVA: usize = rva(0x621480); // dword_621480
pub const MODE_RVA: usize = rva(0x636526); // word_636526
const WINDOW_RVA: [usize; 2] = [rva(0x62408C), rva(0x624090)];
const OFFSET_RVA: [usize; 2] = [rva(0x623FB0), rva(0x623FB4)];
const SCALE_RVA: usize = rva(0x624080); // dword_624080
const SCREEN_RVA: [usize; 2] = [rva(0x4F494C), rva(0x4F4950)];

/// The Rust version (see the header): what it leaves in the registers.
pub unsafe fn choose_view_window() -> Left {
    // Converted: `rc2k_core::scene::view_window::choose`.
    use rc2k_core::scene::view_window::{ViewSettings, choose};
    unsafe {
        let settings = ViewSettings {
            framed: game_read::<u32>(FRAMED_RVA) != 0,
            wide: game_read::<u32>(WIDE_RVA) != 0,
            mode: game_read(MODE_RVA),
        };
        let window = choose(settings, || {
            [game_read(SCREEN_RVA[0]), game_read(SCREEN_RVA[1])]
        });
        game_write(WINDOW_RVA[0], window.size[0]);
        game_write(WINDOW_RVA[1], window.size[1]);
        game_write(OFFSET_RVA[0], window.offset[0]);
        game_write(OFFSET_RVA[1], window.offset[1]);
        game_write(SCALE_RVA, window.scale);
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

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
