//! `drive::choose_shown_cars` (0x4560EA): which car blocks the two views show
//! (`off_63DBB0`, `off_63DBB4` = 0x70F3EC + n 0xFE0 for `dword_63DBA8`, `dword_63DBAC`),
//! `dword_63DB98` (the views' number, from `dword_63DB94`, by `dword_63DB88` when
//! `dword_5C5CE0` is 2, also setting `dword_6238A4`), the replay's car when
//! `dword_52A910` is set (`dword_63DBA0`, then the second view 0), and the eight of
//! `dword_641680` .. -1 but the shown ones' 0 (the first, and the second when there are
//! two views). By the look of it the split screen's set-up (a reading).
//!
//! In:  nothing
//! Out: eax `dword_63DBA0` (with `dword_52A910` set) or the second view's block; the
//!      rest kept; no flags read (callers go on with `mov`, `lea`, calls)
//!
//! Callers (5): race init's loop (`sub_4608BB`), `sub_460835`, `sub_460B2F`,
//! `sub_43A649`, `sub_45429B`.
//! Refuses (before anything) a replay car (`dword_63DBA0`) above 7 with `dword_52A910`
//! set (it would write past `dword_641680`'s eight).
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{EAX, Regs, game_addr, rva};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::drive::shown_cars::{ShownCars, ShownSetup};

/// The variables (IDA addresses), in the order the core uses them.
const VARIABLES: [usize; 19] = [
    0x63DBA8, 0x63DBAC, 0x63DB94, 0x63DB98, 0x5C5CE0, 0x63DB88, 0x6238A4, 0x52A910, 0x63DBA0,
    0x63DBB0, 0x63DBB4, 0x641680, 0x641684, 0x641688, 0x64168C, 0x641690, 0x641694, 0x641698,
    0x64169C,
];
const FIRST_BLOCK: usize = 0x70F3EC; // byte_70F3EC: car block 0's start (an address)

unsafe fn get(v: &[u32], k: usize) -> u32 {
    unsafe { (v[k] as *const u32).read_unaligned() }
}

unsafe fn put(v: &[u32], k: usize, x: u32) {
    unsafe { (v[k] as *mut u32).write_unaligned(x) }
}

/// The Rust version on the variables at `v` (`VARIABLES`' order).
// Converted: `rc2k_core::drive::shown_cars::ShownCars::choose`.
pub unsafe fn choose_at(v: &[u32]) -> Result<Left, Refused> {
    unsafe {
        let setup = ShownSetup {
            players: get(v, 2),
            screen_mode: get(v, 4),
            layout: get(v, 5),
            replay: get(v, 7),
            replay_car: get(v, 8),
        };
        let split = get(v, 6);
        let mut shown = ShownCars {
            split,
            ..ShownCars::default()
        };
        let eax = shown.choose(&setup, game_addr(rva(FIRST_BLOCK)) as u32)?;
        put(v, 0, shown.cars[0]);
        put(v, 1, shown.cars[1]);
        put(v, 3, shown.views);
        if shown.split != split {
            put(v, 6, shown.split);
        }
        put(v, 9, shown.blocks[0]);
        put(v, 10, shown.blocks[1]);
        for (k, &x) in shown.listed.iter().enumerate() {
            put(v, 11 + k, x);
        }
        Ok(Left::nothing().with(EAX, eax))
    }
}

fn game() -> Vec<u32> {
    VARIABLES
        .iter()
        .map(|&x| game_addr(rva(x)) as u32)
        .collect()
}

/// The Rust version.
pub unsafe fn choose_shown_cars() -> Result<Left, Refused> {
    unsafe { choose_at(&game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { choose_shown_cars() }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
