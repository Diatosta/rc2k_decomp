//! The HUD's dials: a layout's discs (rev counter and speedometer dials, `hud::draw_disc`)
//! and needles (`hud::draw_needle`). Each is drawn from the shape's variables, which a
//! set-up first fills from fields of the HUD's record and from variables of the layout's
//! own (the originals: `disc_0` 0x4B6BFD, `disc_1a` 0x4B6B5B, `disc_1b` 0x4B6BAC,
//! `disc_2a` 0x4B6AA4, `disc_2b` 0x4B6B0A; `needle_0` 0x4B6DB9, `needle_1a` 0x4B6E67,
//! `needle_1b` 0x4B6E9A, `needle_2a` 0x4B6DEC, `needle_2b` 0x4B6E34). The layouts are the
//! HUD's three (the record's +0xF4).

use crate::errors::Refused;
use crate::hud::store::{HUD, Hud};
use crate::pointers::GamePtr;
use crate::race::panel_store::PANEL;
use crate::session::{Cell32, Float};

/// A layout's own variables of a dial, in the order its set-up copies them.
type Vars = fn(&Hud) -> &[Float];

/// How one disc is set up.
pub struct DiscStyle {
    /// The record's field of the texture shift's first dword; the second and the turn
    /// follow it.
    shift_field: u32,
    /// The layout's variables: size, texture scale, centre x, centre y; then, when
    /// `lifted_centre`, a second centre y.
    vars: Vars,
    /// Whether there is a second centre y, which replaces the first when `PANEL.flat` is 0.
    lifted_centre: bool,
}

/// How one needle is set up.
pub struct NeedleStyle {
    /// The record's field of the needle's value.
    value_field: u32,
    /// The layout's variables: centre x, centre y; then, when `lifted_centre`, a second
    /// centre y; then the size.
    vars: Vars,
    /// Whether there is a second centre y, which replaces the first when `PANEL.flat` is 0.
    lifted_centre: bool,
}

pub static DISC_0: DiscStyle = DiscStyle {
    shift_field: 0x34,
    vars: |h| &h.disc_0,
    lifted_centre: false,
};
pub static DISC_1A: DiscStyle = DiscStyle {
    shift_field: 0x34,
    vars: |h| &h.disc_1a,
    lifted_centre: false,
};
pub static DISC_1B: DiscStyle = DiscStyle {
    shift_field: 0x5C,
    vars: |h| &h.disc_1b,
    lifted_centre: false,
};
pub static DISC_2A: DiscStyle = DiscStyle {
    shift_field: 0x34,
    vars: |h| &h.disc_2a,
    lifted_centre: true,
};
pub static DISC_2B: DiscStyle = DiscStyle {
    shift_field: 0x5C,
    vars: |h| &h.disc_2b,
    lifted_centre: false,
};

pub static NEEDLE_0: NeedleStyle = NeedleStyle {
    value_field: 0x10,
    vars: |h| &h.needle_0,
    lifted_centre: false,
};
pub static NEEDLE_1A: NeedleStyle = NeedleStyle {
    value_field: 0x10,
    vars: |h| &h.needle_1a,
    lifted_centre: false,
};
pub static NEEDLE_1B: NeedleStyle = NeedleStyle {
    value_field: 0x40,
    vars: |h| &h.needle_1b,
    lifted_centre: false,
};
pub static NEEDLE_2A: NeedleStyle = NeedleStyle {
    value_field: 0x10,
    vars: |h| &h.needle_2a,
    lifted_centre: true,
};
pub static NEEDLE_2B: NeedleStyle = NeedleStyle {
    value_field: 0x40,
    vars: |h| &h.needle_2b,
    lifted_centre: false,
};

/// A dword of the HUD's record, or a refusal.
fn field(record: GamePtr, offset: u32) -> Result<u32, Refused> {
    Ok(record.offset(offset).read::<u32>("a hud record's field")?)
}

/// Copies a variable's bits to another cell.
fn copy(from: &dyn Cell32, to: &dyn Cell32) {
    to.set_bits(from.bits());
}

/// The disc's set-up: the shape's variables filled from the record and the style.
fn set_up_disc(record: GamePtr, style: &DiscStyle) -> Result<(), Refused> {
    let own = (style.vars)(&HUD);
    let from = style.shift_field;
    HUD.disc_texture_shift[0].set_bits(field(record, from)?);
    HUD.disc_texture_shift[1].set_bits(field(record, from + 4)?);
    HUD.disc_turn.set_bits(field(record, from + 8)?);
    copy(&own[0], &HUD.disc_size);
    copy(&own[1], &HUD.disc_texture_scale);
    copy(&own[2], &HUD.disc_centre[0]);
    copy(&own[3], &HUD.disc_centre[1]);
    if style.lifted_centre && PANEL.flat.get() == 0 {
        copy(&own[4], &HUD.disc_centre[1]);
    }
    Ok(())
}

/// The needle's set-up: the shape's variables filled from the record and the style.
fn set_up_needle(record: GamePtr, style: &NeedleStyle) -> Result<(), Refused> {
    let own = (style.vars)(&HUD);
    HUD.needle_value.set_bits(field(record, style.value_field)?);
    copy(&own[0], &HUD.needle_centre[0]);
    copy(&own[1], &HUD.needle_centre[1]);
    let size = if style.lifted_centre {
        if PANEL.flat.get() == 0 {
            copy(&own[2], &HUD.needle_centre[1]);
        }
        3
    } else {
        2
    };
    copy(&own[size], &HUD.needle_size);
    Ok(())
}

/// A disc's set-up and the disc.
pub fn disc(record: GamePtr, style: &DiscStyle) -> Result<(), Refused> {
    set_up_disc(record, style)?;
    crate::hud::draw_disc::draw_disc()
}

/// A needle's set-up and the needle.
pub fn needle(record: GamePtr, style: &NeedleStyle) -> Result<(), Refused> {
    set_up_needle(record, style)?;
    crate::hud::draw_needle::draw_needle()
}
