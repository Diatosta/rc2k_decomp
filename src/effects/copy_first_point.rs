//! `effects::copy_first_point`: `KeptView.vector[0]` .. `KeptView.vector[2]` into
//! `ShapeTilt.point[0]` .. `ShapeTilt.point[2]`, and `KeptView.length` into `ShapeTilt.length`.

use crate::effects::windscreen_drop_vars_store::WindscreenDrops;

pub(super) fn copy_first_point(drops: &WindscreenDrops, effects: &crate::effects::owner::Effects) {
    let kept = effects.frame.kept_view.borrow();
    let mut tilt = drops.tilt.borrow_mut();
    for (n, to) in tilt.point.iter_mut().enumerate() {
        *to = kept.vector[n];
    }
    tilt.length = kept.length;
}
