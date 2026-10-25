//! `physics::value_notice` (0x45B357): notice 0xCE by `dword_71A24C` against 300 (the
//! sign of the difference): at 300 or more, with no notice shown (`dword_71A700` 0) and
//! 0xCE not the last (`dword_71A6FC`), `dword_71A6F4` = 0xCE and `dword_71A704` = 0;
//! below 300 with 0xCE the last, `dword_71A700` and `dword_71A6FC` 0. What the figure is
//! is not known.
//!
//! In:  nothing
//! Out: every register kept; flags not read (`sub_43B635` returns next, its callers
//!      go on with `inc`, calls)
//!
//! Caller: `sub_43B635` (0x43B9AE), its last call, every physics step.

use crate::physics::car_store::CAR;
use crate::physics::cells::{get, put_changed};
use rc2k_core::physics::notices::Notice;

// Converted: `rc2k_core::physics::notices::Notice::value_notice`.
unsafe fn core(v: &[u32]) {
    unsafe {
        let was = Notice {
            shown: get(v[1]),
            last: get(v[2]),
            id: get(v[3]),
            time: get(v[4]),
            ..Notice::default()
        };
        let mut n = was;
        n.value_notice(get(v[0]));
        put_changed(v[3], was.id, n.id);
        put_changed(v[4], was.time, n.time);
        put_changed(v[1], was.shown, n.shown);
        put_changed(v[2], was.last, n.last);
    }
}

fn game() -> Vec<u32> {
    vec![
        CAR.turned_steps.addr() as u32,
        CAR.notice_shown.addr() as u32,
        CAR.notice_last.addr() as u32,
        CAR.notice_id.addr() as u32,
        CAR.notice_time.addr() as u32,
    ]
}

/// The Rust version.
pub unsafe fn value_notice() {
    unsafe { core(&game()) }
}
