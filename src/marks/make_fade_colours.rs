//! `marks::make_fade_colours` (0x4C37EF): 0x11 colours from `dword_6415E4` toward white:
//! each component's inverse (0xFF less it) times a step that grows from 0 by
//! `flt_7651C4` (`dword_8DB4A8`, a float), rounded, made a pixel (alpha 0xFF) and added to
//! the colour (as one dword, carries crossing components), into `dword_8DB464`.
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
//!
//! One caller, `sub_4C25F6`.

use crate::marks::snow::SNOW_LIGHT;
use crate::marks::tuning::MARKS_FLOATS;
use crate::math::x87;
use crate::race::frame_store::FRAME_WORK;
const FADES: usize = 0x11;

/// The Rust version (see the header).
pub unsafe fn make_fade_colours() {
    let l = &SNOW_LIGHT;
    {
        let colour: u32 = FRAME_WORK.view_light.bits();
        let inverses = [
            ((colour >> 16) & 0xFF) ^ 0xFF,
            ((colour >> 8) & 0xFF) ^ 0xFF,
            (colour & 0xFF) ^ 0xFF,
        ];
        for (cell, v) in l.fade_inverses.iter().zip(inverses) {
            cell.set(v);
        }
        l.fade_step.set_bits(0);
        for n in 0..FADES {
            let step = l.fade_step.bits();
            let inverses = l.fade_inverses.each_ref().map(|d| d.get() as i32);
            let scaled = inverses.map(|v| x87::int_times_to_int(v, step));
            for (cell, v) in l.fade_scaled.iter().zip(scaled) {
                cell.set(v as u32);
            }
            let [r, gr, b] = scaled.map(|v| v as u32 & 0xFF);
            let fade =
                (0xFF00_0000 | r << 16 | gr << 8 | b).wrapping_add(FRAME_WORK.view_light.bits());
            l.fade_colours[n].set(fade);
            l.fade_step
                .set_bits(x87::add(step, MARKS_FLOATS.sixteenth.bits()));
        }
    }
}
