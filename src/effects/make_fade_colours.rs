//! `effects::make_fade_colours`: 0x11 colours from `FrameWork.view_light` toward white: each
//! component's inverse (0xFF less it) times a step that grows from 0 by `MarksFloats.sixteenth`
//! (`SnowLight.fade_step`, a float), rounded, made a pixel (alpha 0xFF) and added to the colour (as
//! one dword, carries crossing components), into `SnowLight.fade_colours[0]`.

use crate::effects::rain_store::Weather;
use crate::math::x87;
const FADES: usize = 0x11;

pub(super) fn make_fade_colours(
    view_light: u32,
    weather: &Weather,
    effects: &crate::effects::owner::Effects,
) {
    let l = &weather.snow_light;
    {
        let colour: u32 = view_light;
        let inverses = [
            ((colour >> 16) & 0xFF) ^ 0xFF,
            ((colour >> 8) & 0xFF) ^ 0xFF,
            (colour & 0xFF) ^ 0xFF,
        ];
        for (cell, v) in l.fade_inverses.iter().zip(inverses) {
            cell.set(v);
        }
        l.fade_step.set(0.0);
        for n in 0..FADES {
            let step = l.fade_step.get();
            let inverses = l.fade_inverses.each_ref().map(|d| d.get() as i32);
            let scaled = inverses.map(|v| x87::int_times_to_int(v, step));
            for (cell, v) in l.fade_scaled.iter().zip(scaled) {
                cell.set(v as u32);
            }
            let [r, gr, b] = scaled.map(|v| v as u32 & 0xFF);
            let fade = (0xFF00_0000 | r << 16 | gr << 8 | b).wrapping_add(view_light);
            l.fade_colours[n].set(fade);
            l.fade_step
                .set(x87::add(step, &effects.setup.floats.sixteenth));
        }
    }
}
