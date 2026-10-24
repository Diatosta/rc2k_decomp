//! `drive::set_listener` (0x4565B7): the state the sounds are placed by, from the viewed
//! car (`g_carBlocks[dword_63DB9C]`), car block 0 (0x70F410 ..) and the view: its three
//! whole numbers +0x134 .. +0x13C as floats (0x5C5630 ..), `dword_5C2FD4` = (+0xABC >=
//! 0x10); the view's place negated and its turn's nine (0x5C2FA0 .. 0x5C2FCC); three of
//! 0x6415D0 .. (the first 0 without `dword_6238A8`) into 0x5C56D8 ..; six whole numbers of
//! the car's light record (`g_lightRecords` +0x40 .., by `race::car_file_number` of
//! `dword_70F410`) times 65536 over 1000 (two less `dword_613FC4`) into 0x608C60 ..; car
//! block 0's matrix (0x70FC2C) through `scene::multiply_matrices` with 0x70FC6C and the
//! view matrix into `g_madeObjectMatrix` (and a copy at 0x5EA0CC); the two points of
//! 0x608C60 .. turned by it, each with its x and then -x (`scene::rotate_point`,
//! `fistp`'d into 0x5C568C .. 0x5C56B8); the switches of 0x5C56BC .. 0x5C56E8 by
//! `dword_6415E0`, `dword_70FCEC`, `dword_70FCF0`; `dword_5C5680` from `dword_70F628`
//! (then cleared), `dword_5C5684`; `flt_5C5660` = `dword_70FA3C` / 2^29 + `flt_637644`;
//! two floats copied; with `dword_601024` set, `dword_5C5680` and `dword_5C5684` 0. By
//! the look of it the listener for the 3D sounds (a reading).
//!
//! In:  nothing
//! Out: eax the old `dword_70F628`, ebx 0x10000, ecx 0, edx 0, esi `g_madeObjectMatrix`,
//!      edi 0x5EA10C, ebp `g_madeObjectMatrix`; no flags read
//!
//! Caller: `drive::share_all_cars` (0x456484), second. Refuses (before anything) a viewed
//! car past the eight blocks or with a null block, a car file number above 0x21, and a
//! light record figure whose times 65536 over 1000 leaves 32 bits (the original's `idiv`
//! traps).

use crate::errors::Refused;
use crate::math::x87;
use crate::math::x87_run::{ext_to_int, int_80, int_over_int_plus};
use crate::physics::mirror::Mirror;
use crate::race::mud::car_file_number;
use crate::scene::matrix::multiply_matrices;
use crate::scene::rotate_point::rotate_point;
use rc2k_core::drive::listener;

const TURNED: u32 = 0x5EA10C; // g_turnedMatrix
const LIT: u32 = 0x5EA14C; // g_litMatrix
const VIEW_TURN: u32 = 0x5EA24C; // g_viewTurnMatrix
const VIEW: u32 = 0x5EA2CC; // g_viewMatrix
const MADE: u32 = 0x5EA30C; // g_madeObjectMatrix
const LIGHT_FIGURES: u32 = 0x5C90AC; // g_lightRecords + 0x40

/// The car file's six figures times 65536 over 1000, or the refusal (before anything).
unsafe fn figures(m: &Mirror) -> Result<[i32; 6], Refused> {
    unsafe {
        let viewed = m.get(0x63DB9C);
        listener::check_viewed(viewed, || m.get(0x71B3B8 + 4 * viewed) != 0)?;
        let n = car_file_number(m.get(0x70F410));
        rc2k_core::drive::sharing::check_light_record(n)?;
        let mut out = [0i32; 6];
        for (k, o) in out.iter_mut().enumerate() {
            *o = listener::light_figure(m.get(LIGHT_FIGURES + 0x134 * n + 4 * k as u32))?;
        }
        Ok(out)
    }
}

/// The Rust core on the variables `m` reaches. The registers it leaves are read by no
/// caller (liveness: eax and ebx after `share_all_cars`' call have no read point).
// Converted: `rc2k_core::drive::listener` (the matrices, turns and x87 sums stay).
pub unsafe fn set_listener_at(m: &Mirror) -> Result<(), Refused> {
    unsafe {
        let f = figures(m)?;
        let g = |v: u32| m.get(v);
        let s = m.get(0x71B3B8 + 4 * m.get(0x63DB9C));
        let at = |o: u32| (s.wrapping_add(o) as *const u32).read_unaligned();
        m.put(0x5C5638, x87::from_int(at(0x13C) as i32));
        m.put(0x5C5634, x87::from_int(at(0x138) as i32));
        m.put(0x5C5630, x87::from_int(at(0x134) as i32));
        m.put(0x5C2FD4, u32::from(listener::car_ahead(at(0xABC))));
        for k in 0..3 {
            m.put(0x5C2FA0 + 4 * k, g(0x606298 + 4 * k).wrapping_neg());
        }
        for (k, o) in listener::TURN_OFFSETS.into_iter().enumerate() {
            m.put(0x5C2FAC + 4 * k as u32, g(VIEW_TURN + o));
        }
        m.put(
            0x5C56D8,
            listener::first_figure(g(0x6238A8), || g(0x6415D8)),
        );
        m.put(0x5C56DC, g(0x6415D0));
        m.put(0x5C56E0, g(0x6415D4));
        let f = listener::listener_points(f, g(0x613FC4));
        for (k, &v) in f.iter().enumerate() {
            m.put(0x608C60 + 4 * k as u32, v as u32);
        }
        for k in 0..16 {
            m.put(TURNED + 4 * k, g(0x70FC2C + 4 * k));
        }
        multiply_matrices(
            m.a(TURNED) as usize,
            m.a(0x70FC6C) as usize,
            m.a(LIT) as usize,
        );
        multiply_matrices(m.a(LIT) as usize, m.a(VIEW) as usize, m.a(MADE) as usize);
        for k in 0..16 {
            m.put(0x5EA0CC + 4 * k, g(MADE + 4 * k));
        }
        let made = m.a(MADE);
        for (first, out, negate) in [
            (0, 0x5C568C, true),
            (0, 0x5C5698, false),
            (3, 0x5C56A4, true),
            (3, 0x5C56B0, false),
        ] {
            let p = [
                int_80(f[first], negate),
                int_80(f[first + 1], false),
                int_80(f[first + 2], false),
            ];
            let r = rotate_point(made, p);
            for (k, e) in r.iter().enumerate() {
                m.put(out + 4 * k as u32, ext_to_int(*e) as u32);
            }
        }
        let switches = listener::switches(g(0x6415E0) != 0, g(0x70FCEC), g(0x70FCF0));
        for (at, v) in [
            0x5C56BC, 0x5C56E8, 0x5C56C0, 0x5C56C4, 0x5C56E4, 0x5C56D0, 0x5C56C8, 0x5C56CC,
        ]
        .into_iter()
        .zip(switches)
        {
            m.put(at, v);
        }
        let old = g(0x70F628);
        if old != 0 {
            m.put(0x5C5680, old);
        }
        m.put(0x70F628, 0);
        m.put(0x5C5684, u32::from(g(0x70FA2C) != 0));
        m.put(0x62400C, 0x2000_0000);
        m.put(
            0x5C5660,
            int_over_int_plus(g(0x70FA3C) as i32, 0x2000_0000, g(0x637644)),
        );
        m.put(0x5C5624, x87::loaded(g(0x70F5C4)));
        m.put(0x5C5628, x87::loaded(g(0x7102F4)));
        if g(0x601024) != 0 {
            m.put(0x5C5684, 0);
            m.put(0x5C5680, 0);
        }
        Ok(())
    }
}

/// The Rust version.
pub unsafe fn set_listener() -> Result<(), Refused> {
    unsafe { set_listener_at(&Mirror::game()) }
}
