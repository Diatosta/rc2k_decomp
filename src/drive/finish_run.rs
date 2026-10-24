//! `drive::finish_run` (0x459360): the championship's finish routine played. The car
//! lifted (+0x138 + `flt_64BB38` x 65536, `fiadd` / `fistp`); the clock into
//! `dword_5CC3B4`, `5CC3B0`; 0x20 dwords at `unk_5CC1B0` and `dword_5C2538` -1; the start
//! `dword_64BB60`. Each frame: `dword_512A44` 0, `input::update`, the time
//! (`drive::finish_timing`); until the time reaches the length (`flt_64BB3C[n]`, n
//! `dword_75B108`) or Esc or key 0x1C is down: the car's view 6 (+0x684), the first car
//! loaded (`race::load_first_car`), the camera (`drive::finish_camera`,
//! `physics::camera_frame`), the car stored (`drive::store_first_car`), the race's frame
//! (`sub_4541BF`, stream A's: through the game).
//!
//! In:  nothing
//! Out: the registers as the steps leave them; no flags read
//!
//! Caller: `drive::finish_race`. Ungated: it plays for seconds.
//! Refuses a length index past 1 (the table's two), and what its calls refuse. Shadow
//! mode runs the original alone.
//! Self-test: skipped (it plays the routine).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::drive::race_parts::{input_update, race_frame, restart_clock};
use crate::errors::{BadInput, Refused};
use crate::physics::flow::{a, base, c0, g, p, x87_run};

const B: usize = 0x70F3EC;

/// The Rust version. The registers it leaves are read by no caller (liveness: esi after
/// `drive::finish_race`'s call has no read point).
pub unsafe fn finish_run() -> Result<(), Refused> {
    x87_run!(
        "fld dword ptr [{b} + 0x24BB38]",
        "fmul dword ptr [{b} + 0x1C5D98]",
        "fiadd dword ptr [{b} + 0x30F524]",
        "fistp dword ptr [{b} + 0x30F524]";
        b = base()
    );
    restart_clock();
    p(0x5CC3B4, g(0x613830));
    p(0x5CC3B0, g(0x613830));
    for k in 0..0x20 {
        p(0x5CC1B0 + 4 * k, u32::MAX);
    }
    p(0x5C2538, u32::MAX);
    restart_clock();
    p(0x64BB60, g(0x613830));
    loop {
        p(0x512A44, 0);
        input_update();
        unsafe { crate::drive::finish_timing::finish_timing() };
        let n = g(0x75B108);
        if n > 1 {
            return Err(BadInput("a finish length past the table's two").into());
        }
        if !c0(g(0x64BB64), g(0x64BB3C + 4 * n as usize)) {
            break;
        }
        let key = |x: usize| unsafe { (a(x) as *const u8).read() } != 0;
        if key(0x4F6CDD) || key(0x4F6CF8) {
            break;
        }
        p(B + 0x684, 6);
        unsafe {
            crate::race::frame_views::load_first_car()?;
            crate::drive::finish_camera::finish_camera();
            crate::physics::camera_frame::camera_frame()?;
            crate::drive::store_first_car::store_first_car()?;
        }
        race_frame()?;
    }
    Ok(())
}
