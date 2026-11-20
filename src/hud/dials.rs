//! The HUD's dials: a layout's discs (rev counter and speedometer dials, `hud::draw_disc`) and
//! needles (`hud::draw_needle`). Each is drawn from the shape's variables, which a set-up first
//! fills from fields of the panel (`race::panel_store::Panel`) and from variables of the layout's
//! own. The layouts are the HUD's three (`Panel::players`).

use crate::errors::Refused;
use crate::hud::block_store::HudBlock;
use crate::hud::owner::Hud;
use crate::hud::pen::Pen;
use crate::memory::session::PlainDword;
use crate::memory::session::PlainFloat;
use crate::race::owner_frame::Frame;
use crate::race::panel_store::Panel;

/// A layout's own variables of a dial, in the order its set-up copies them.
type Vars = fn(&HudBlock) -> &[PlainFloat];

pub struct DiscStyle {
    /// The texture shift (two dwords) and turn: a view's `motion`.
    shift: fn(&Panel) -> &[PlainDword; 3],
    /// The layout's variables: size, texture scale, centre x, centre y; then, when
    /// `lifted_centre`, a second centre y.
    vars: Vars,
    /// Whether there is a second centre y, which replaces the first when `PANEL.flat` is 0.
    lifted_centre: bool,
}

pub struct NeedleStyle {
    /// The needle's value: a view's `whole`.
    value: fn(&Panel) -> &PlainDword,
    /// The layout's variables: centre x, centre y; then, when `lifted_centre`, a second
    /// centre y; then the size.
    vars: Vars,
    /// Whether there is a second centre y, which replaces the first when `PANEL.flat` is 0.
    lifted_centre: bool,
}

pub(super) static DISC_0: DiscStyle = DiscStyle {
    shift: |p| &p.view_a.motion,
    vars: |h| &h.disc_setup_layout0,
    lifted_centre: false,
};
pub(super) static DISC_1A: DiscStyle = DiscStyle {
    shift: |p| &p.view_a.motion,
    vars: |h| &h.disc_setup_layout1_first,
    lifted_centre: false,
};
pub(super) static DISC_1B: DiscStyle = DiscStyle {
    shift: |p| &p.view_b.motion,
    vars: |h| &h.disc_setup_layout1_second,
    lifted_centre: false,
};
pub(super) static DISC_2A: DiscStyle = DiscStyle {
    shift: |p| &p.view_a.motion,
    vars: |h| &h.disc_setup_layout2_first,
    lifted_centre: true,
};
pub(super) static DISC_2B: DiscStyle = DiscStyle {
    shift: |p| &p.view_b.motion,
    vars: |h| &h.disc_setup_layout2_second,
    lifted_centre: false,
};

pub(super) static NEEDLE_0: NeedleStyle = NeedleStyle {
    value: |p| &p.view_a.whole,
    vars: |h| &h.needle_setup_layout0,
    lifted_centre: false,
};
pub(super) static NEEDLE_1A: NeedleStyle = NeedleStyle {
    value: |p| &p.view_a.whole,
    vars: |h| &h.needle_setup_layout1_first,
    lifted_centre: false,
};
pub(super) static NEEDLE_1B: NeedleStyle = NeedleStyle {
    value: |p| &p.view_b.whole,
    vars: |h| &h.needle_setup_layout1_second,
    lifted_centre: false,
};
pub(super) static NEEDLE_2A: NeedleStyle = NeedleStyle {
    value: |p| &p.view_a.whole,
    vars: |h| &h.needle_setup_layout2_first,
    lifted_centre: true,
};
pub(super) static NEEDLE_2B: NeedleStyle = NeedleStyle {
    value: |p| &p.view_b.whole,
    vars: |h| &h.needle_setup_layout2_second,
    lifted_centre: false,
};

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

pub fn disc(frame: &Frame, hud: &Hud, pen: Pen, style: &DiscStyle) -> Result<(), Refused> {
    set_up_disc(frame, hud, style);
    crate::hud::draw_disc::draw_disc(frame, hud, pen)
}

pub fn needle(frame: &Frame, hud: &Hud, pen: Pen, style: &NeedleStyle) -> Result<(), Refused> {
    set_up_needle(frame, hud, style);
    crate::hud::draw_needle::draw_needle(frame, hud, pen)
}

/// The other players' markers on a disc of the two players' layouts: the disc set up again (the
/// later disc's set-up left its variables) and its view's markers drawn at the sprites' scale.
pub(super) fn map_cars(
    frame: &Frame,
    hud: &Hud,
    pen: Pen,
    style: &DiscStyle,
    slot: usize,
) -> Result<(), Refused> {
    set_up_disc(frame, hud, style);
    crate::hud::draw_map_cars::draw_map_cars(frame, hud, pen, slot, true)
}
