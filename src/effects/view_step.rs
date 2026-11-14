//! `effects::view_step`: the view's step since the last frame, negated: for each of x, y, z
//! `KeptView.step[0]` .. = -(`KeptView.kept_a.scaled[0]` .. less `KeptView.last[0]` ..), then
//! `KeptView.last[0]` .. = `KeptView.kept_a.scaled[0]` ..

use crate::math::x87;

pub fn view_step(effects: &crate::effects::owner::Effects) {
    let k = &effects.frame.kept_view;
    // now: `kept_a`'s scaled vector (`KeptView.kept_a.scaled[0]` ..)
    for n in 0..3 {
        k.step[n].set(x87::negated(x87::sub(&k.kept_a.scaled[n], &k.last[n])));
    }
    for n in 0..3 {
        k.last[n].set(k.kept_a.scaled[n].get());
    }
}
