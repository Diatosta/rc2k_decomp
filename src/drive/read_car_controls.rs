//! `drive::read_car_controls` (0x43A35A): the work car's controls for this step:
//! - the keys (`dword_6138C4` .. `D0`) into the car's (`dword_71A43C` .. `48`), the
//!   held-before into `dword_5C694C` and the newly pressed (`dword_71A44C`, bits not held
//!   before); steering, throttle and brake into 0x613890 .. 0x613898, and three bits as
//!   0x61389C (bit 2 times 0x60), 0x6138A0, 0x6138A4;
//! - for a player's car (or in a replay, `dword_52A910`): bit 3 newly pressed flips
//!   `dword_71A05C`, bit 5 changes the view (`drive::next_camera_view` on `g_car`);
//! - for a computer's car (`dword_71C3E0` set, or `dword_62389C` and `dword_71A234` 2):
//!   `drive::start_blend`, `flt_71BFA4` = `flt_71A658`, `drive::drive_computer_car`, and
//!   its six controls (0x71BF88 ..) into 0x613890 .. 0x6138A4.
//!
//! The bytes after its `retn` (0x43A4E5 ..) are another entry that nothing calls.
//!
//! In:  nothing
//! Out: eax as the path leaves it (the last bit's, `next_camera_view`'s, or
//!      `start_blend`'s count); esi `g_car` when the view changed; the rest kept; no
//!      flags read (callers go on with calls)
//!
//! Callers: race init's loop (`sub_4608BB`), `sub_460835`, `sub_460B2F`, `sub_43A1CA`.
//! Refuses what `drive_computer_car` refuses (before anything).

use crate::drive::drive_computer_car::drive_computer_car;
use crate::drive::next_camera_view::next_camera_view;
use crate::drive::start_blend::start_blend;
use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::math::x87;
use rc2k_core::drive::controls::{FLIP_BIT, VIEW_BIT, computer_drives, flipped, read_keys};

fn g(x: u32) -> u32 {
    unsafe { game_read(rva(x as usize)) }
}

fn p(x: u32, v: u32) {
    unsafe { game_write(rva(x as usize), v) }
}

fn a(x: u32) -> u32 {
    game_addr(rva(x as usize)) as u32
}

const CURRENT: u32 = 0x71B3F8; // g_currentCar

/// Does a computer drive this step?
fn computer() -> bool {
    computer_drives(g(0x71C3E0), || g(0x62389C), || g(0x71A234))
}

/// The Rust version: `r` as the original leaves it.
// Converted: `rc2k_core::drive::controls::{read_keys, computer_drives, flipped}`.
pub unsafe fn read_car_controls() -> Result<(), Refused> {
    unsafe {
        if computer() && g(0x623640) == 0 && g(0x719FEC) >= 0x320 {
            // `drive_computer_car`'s refusal, before anything here is written.
            let reaches = crate::math::x87_run::compare_full(g(0x71A0A0), g(0x71BF70)) & 0x100 == 0;
            if reaches {
                return Err(BadInput("no checkpoints for the computer's driving").into());
            }
        }
        p(0x5C694C, g(0x71A440));
        p(0x71A440, 0);
        p(0x71A44C, 0);
        for (from, to) in [
            (0x6138C4, 0x71A43C),
            (0x6138C8, 0x71A440),
            (0x6138CC, 0x71A444),
            (0x6138D0, 0x71A448),
        ] {
            p(to, g(from));
        }
        let keys = read_keys(g(0x5C694C), g(0x71A440));
        let pressed = keys.pressed;
        p(0x71A44C, pressed);
        p(0x613894, g(0x71A43C));
        p(0x613890, g(0x71A444));
        p(0x613898, g(0x71A448));
        p(0x61389C, keys.hand);
        p(0x6138A0, keys.up);
        p(0x6138A4, keys.down);
        let replay = g(0x52A910) != 0;
        if replay || g(CURRENT) < g(0x63DB94) {
            if pressed & FLIP_BIT != 0 {
                p(0x71A05C, flipped(g(0x71A05C)));
            }
            if pressed & VIEW_BIT != 0 {
                next_camera_view(a(0x719FD8));
            }
        }
        if !computer() {
            return Ok(());
        }
        p(0x71C0CC, g(CURRENT));
        start_blend();
        p(0x71BFA4, x87::loaded(g(0x71A658)));
        drive_computer_car()?;
        for (k, to) in [
            0x613890u32,
            0x613894,
            0x613898,
            0x61389C,
            0x6138A0,
            0x6138A4,
        ]
        .into_iter()
        .enumerate()
        {
            p(to, g(0x71BF88 + 4 * k as u32));
        }
        Ok(())
    }
}
