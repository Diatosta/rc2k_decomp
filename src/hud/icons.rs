//! The icons of the three layouts: a scale (one of the three icon scales), then one or two
//! `hud::draw_icon` calls, each with the horizontal shift, the row and the view whose
//! start lights it shows. Between the two the texture number goes up by one.

use crate::errors::Refused;
use crate::hud::draw_icon::draw_icon;
use crate::hud::owner::{Hud, Pen};
use crate::race::owner_frame::Frame;

/// One icon: the shift, the row and the view (`Panel::lights`).
pub struct IconDraw {
    pub shift: u32,
    pub row: u32,
    pub view: usize,
}

/// A layout's icons.
pub struct IconStyle {
    /// Which of the HUD block's icon scales.
    pub scale: usize,
    pub draws: &'static [IconDraw],
}

pub(super) static LAYOUT_0: IconStyle = IconStyle {
    scale: 0,
    draws: &[IconDraw {
        shift: 0,
        row: 0xC8,
        view: 0,
    }],
};
pub(super) static LAYOUT_1: IconStyle = IconStyle {
    scale: 2,
    draws: &[
        IconDraw {
            shift: 0xFFFF_FF38,
            row: 0xC8,
            view: 0,
        },
        IconDraw {
            shift: 0xC8,
            row: 0xC8,
            view: 1,
        },
    ],
};
pub(super) static LAYOUT_2: IconStyle = IconStyle {
    scale: 1,
    draws: &[
        IconDraw {
            shift: 0,
            row: 0x3C,
            view: 0,
        },
        IconDraw {
            shift: 0,
            row: 0x177,
            view: 1,
        },
    ],
};

/// Draws `style`'s icons.
pub fn icons(frame: &Frame, hud: &Hud, pen: Pen, style: &IconStyle) -> Result<(), Refused> {
    hud.block
        .icon_scale
        .set(hud.block.icon_scales[style.scale].get());
    for (n, d) in style.draws.iter().enumerate() {
        if n > 0 {
            hud.block
                .texture
                .set(hud.block.texture.get().wrapping_add(1));
        }
        hud.block.icon_shift.set(d.shift);
        let lights = &frame.panel_and_overlay.get().panel.lights[d.view];
        draw_icon(frame, hud, pen, lights, d.row)?;
    }
    Ok(())
}
