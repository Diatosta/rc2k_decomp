//! `drive::put_back_lead_car_values` (0x456556): the eight values
//! `drive::keep_lead_car_values` kept (0x64165C ..) back into car block 0 (0x70FA70 ..,
//! `drive/lead_car_values.rs`; `push`/`pop`).
//!
//! In:  nothing
//! Out: every register kept; no flags read (`drive::share_all_cars` returns next)
//!
//! Caller: `drive::share_all_cars` (0x456484), last.

use crate::drive::lead_car_values::{PAIRS, move_pairs};
use crate::framework::{game_addr, rva};

fn game() -> Vec<(u32, u32)> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    PAIRS.iter().map(|&(f, t)| (a(t), a(f))).collect()
}

/// The Rust version.
pub unsafe fn put_back_lead_car_values() {
    unsafe { move_pairs(&game()) }
}
