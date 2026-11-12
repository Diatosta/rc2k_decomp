//! The HUD's dials: a layout's discs (rev counter and speedometer dials, `hud::draw_disc`)
//! and needles (`hud::draw_needle`). Each is drawn from the shape's variables, which a
//! set-up first fills from fields of the panel (`race::panel_store::Panel`) and from variables of
//! the layout's own (the originals: `disc_setup_layout0` 0x4B6BFD, `disc_setup_layout1_first` 0x4B6B5B, `disc_setup_layout1_second` 0x4B6BAC,
//! `disc_setup_layout2_first` 0x4B6AA4, `disc_setup_layout2_second` 0x4B6B0A; `needle_setup_layout0` 0x4B6DB9, `needle_setup_layout1_first` 0x4B6E67,
//! `needle_setup_layout1_second` 0x4B6E9A, `needle_setup_layout2_first` 0x4B6DEC, `needle_setup_layout2_second` 0x4B6E34). The layouts are the
//! HUD's three (`Panel::players`).

use crate::errors::Refused;
use crate::hud::owner::{Hud, Pen};
use crate::hud::store::HudBlock;
use crate::race::owner_frame::Frame;
use crate::race::panel_store::Panel;
use crate::session::PlainFloat;
use crate::session::PlainLoose;

/// A layout's own variables of a dial, in the order its set-up copies them.
type Vars = fn(&HudBlock) -> &[PlainFloat];

/// How one disc is set up.
pub struct DiscStyle {
    /// The texture shift (two dwords) and turn: a view's `motion`.
    shift: fn(&Panel) -> &[PlainLoose; 3],
    /// The layout's variables: size, texture scale, centre x, centre y; then, when
    /// `lifted_centre`, a second centre y.
    vars: Vars,
    /// Whether there is a second centre y, which replaces the first when `PANEL.flat` is 0.
    lifted_centre: bool,
}

/// How one needle is set up.
pub struct NeedleStyle {
    /// The needle's value: a view's `whole`.
    value: fn(&Panel) -> &PlainLoose,
    /// The layout's variables: centre x, centre y; then, when `lifted_centre`, a second
    /// centre y; then the size.
    vars: Vars,
    /// Whether there is a second centre y, which replaces the first when `PANEL.flat` is 0.
    lifted_centre: bool,
}

pub static DISC_0: DiscStyle = DiscStyle {
    shift: |p| &p.view_a.motion,
    vars: |h| &h.disc_setup_layout0,
    lifted_centre: false,
};
pub static DISC_1A: DiscStyle = DiscStyle {
    shift: |p| &p.view_a.motion,
    vars: |h| &h.disc_setup_layout1_first,
    lifted_centre: false,
};
pub static DISC_1B: DiscStyle = DiscStyle {
    shift: |p| &p.view_b.motion,
    vars: |h| &h.disc_setup_layout1_second,
    lifted_centre: false,
};
pub static DISC_2A: DiscStyle = DiscStyle {
    shift: |p| &p.view_a.motion,
    vars: |h| &h.disc_setup_layout2_first,
    lifted_centre: true,
};
pub static DISC_2B: DiscStyle = DiscStyle {
    shift: |p| &p.view_b.motion,
    vars: |h| &h.disc_setup_layout2_second,
    lifted_centre: false,
};

pub static NEEDLE_0: NeedleStyle = NeedleStyle {
    value: |p| &p.view_a.whole,
    vars: |h| &h.needle_setup_layout0,
    lifted_centre: false,
};
pub static NEEDLE_1A: NeedleStyle = NeedleStyle {
    value: |p| &p.view_a.whole,
    vars: |h| &h.needle_setup_layout1_first,
    lifted_centre: false,
};
pub static NEEDLE_1B: NeedleStyle = NeedleStyle {
    value: |p| &p.view_b.whole,
    vars: |h| &h.needle_setup_layout1_second,
    lifted_centre: false,
};
pub static NEEDLE_2A: NeedleStyle = NeedleStyle {
    value: |p| &p.view_a.whole,
    vars: |h| &h.needle_setup_layout2_first,
    lifted_centre: true,
};
pub static NEEDLE_2B: NeedleStyle = NeedleStyle {
    value: |p| &p.view_b.whole,
    vars: |h| &h.needle_setup_layout2_second,
    lifted_centre: false,
};

/// Copies a variable's value to another cell.
fn copy(from: &PlainFloat, to: &PlainFloat) {
    to.set(from.get());
}

/// The disc's set-up: the shape's variables filled from the panel and the style.
fn set_up_disc(frame: &Frame, hud: &Hud, style: &DiscStyle) {
    let own = (style.vars)(&hud.block);
    let [u, v, turn] = (style.shift)(&frame.panel_and_overlay.get().panel);
    hud.block.disc_texture_shift[0].set(f32::from_bits(u.get()));
    hud.block.disc_texture_shift[1].set(f32::from_bits(v.get()));
    hud.block.disc_turn.set(f32::from_bits(turn.get()));
    copy(&own[0], &hud.block.disc_size);
    copy(&own[1], &hud.block.disc_texture_scale);
    copy(&own[2], &hud.block.disc_centre[0]);
    copy(&own[3], &hud.block.disc_centre[1]);
    if style.lifted_centre && frame.panel_and_overlay.get().panel.flat.get() == 0 {
        copy(&own[4], &hud.block.disc_centre[1]);
    }
}

/// The needle's set-up: the shape's variables filled from the panel and the style.
fn set_up_needle(frame: &Frame, hud: &Hud, style: &NeedleStyle) {
    let own = (style.vars)(&hud.block);
    let value = (style.value)(&frame.panel_and_overlay.get().panel);
    hud.block.needle_value.set(value.get());
    copy(&own[0], &hud.block.needle_centre[0]);
    copy(&own[1], &hud.block.needle_centre[1]);
    let size = if style.lifted_centre {
        if frame.panel_and_overlay.get().panel.flat.get() == 0 {
            copy(&own[2], &hud.block.needle_centre[1]);
        }
        3
    } else {
        2
    };
    copy(&own[size], &hud.block.needle_size);
}

/// A disc's set-up and the disc.
pub fn disc(frame: &Frame, hud: &Hud, pen: Pen, style: &DiscStyle) -> Result<(), Refused> {
    set_up_disc(frame, hud, style);
    crate::hud::draw_disc::draw_disc(frame, hud, pen)
}

/// A needle's set-up and the needle.
pub fn needle(frame: &Frame, hud: &Hud, pen: Pen, style: &NeedleStyle) -> Result<(), Refused> {
    set_up_needle(frame, hud, style);
    crate::hud::draw_needle::draw_needle(frame, hud, pen)
}
