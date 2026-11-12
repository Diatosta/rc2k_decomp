//! The race panel: the HUD record the race frame fills for the HUD (`g_hudRecord`, IDA 0x6D13FE;
//! the original's HUD walks it by offset, the `hud::` drawers read its fields) and the variables
//! around it: the two views' values (place, times, split gaps, motion), the overlay's miles and
//! parts, the start lights, the marks, the HUD flags and the texture counter. Names are readings
//! unless IDA's.
//! Most of its dwords lie at addresses that are not a multiple of 4 (the piece starts at 0x6D13FE): they are `Loose32` cells and the piece needs no alignment.

#[cfg(test)]
use crate::race::owner::race;
use crate::session::{Bytes, PlainBytes, PlainLoose};

/// A dword at an address that is not a multiple of 4: plain unaligned volatile loads and stores, as the image's were.
#[repr(transparent)]
#[derive(rc2k_derive::Cells)]
pub struct Loose32(Bytes<4>);

impl Loose32 {
    pub const fn new(value: u32) -> Self {
        Self(Bytes::new(value.to_le_bytes()))
    }

    pub fn get(&self) -> u32 {
        self.0.get_u32(0)
    }

    pub fn set(&self, value: u32) {
        self.0.set_u32(0, value);
    }
}

/// The first view's values for the panel (0x30 bytes from 0x6D140E; `race::panel_values`, `race::panel_motion`).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ViewA {
    /// The speed shown, rounded (`flt_6D140E`; the car's +0x718).
    pub whole: PlainLoose,
    /// The engine sound's speed (the car's +0xAB0).
    pub speed: PlainLoose,
    /// The gear (the car's +0x1D4).
    pub gear: PlainLoose,
    /// The place (`dword_6D141A`; `race::panel_players` and `race::panel_neighbours` set it, `race::panel_values` the car's split gap 0).
    pub place: PlainLoose,
    /// The stage time in tenths (the car's +0x88 x 10).
    pub time: PlainLoose,
    /// The car's +0xA14 (split gap 2; `race::panel_neighbours`: its +0x634).
    pub split2: PlainLoose,
    /// The car behind's number (`dword_6D1426`; the car's split gap 4 in `race::panel_values`).
    pub back: PlainLoose,
    /// Minus the car's split gap 1 (`dword_6D142A`).
    pub negated: PlainLoose,
    /// The car ahead's number (`dword_6D142E`; the car's split gap 3 in `race::panel_values`).
    pub front: PlainLoose,
    /// `flt_6D1432` ..: the map picture's two shifts and the heading (`race::panel_motion`).
    pub motion: [PlainLoose; 3],
}

/// The second view's values for the panel (0x28 bytes from 0x6D143E).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ViewB {
    pub whole: PlainLoose,
    pub speed: PlainLoose,
    pub gear: PlainLoose,
    /// `dword_6D144A`.
    pub place: PlainLoose,
    pub time: PlainLoose,
    pub split2: PlainLoose,
    /// Minus the car's split gap 1 (`dword_6D1456`).
    pub negated: PlainLoose,
    /// `flt_6D145A` ..: as the first view's.
    pub motion: [PlainLoose; 3],
}

/// One view's start lights (0x10 bytes from 0x6D1482 and 0x6D1492; `race::countdown_lights`, `drive::countdown_beep`).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Lights {
    /// The lit count (`unk_6D1482`; -1 before the race: no extra texture).
    pub lit: PlainLoose,
    /// The start-lights icon's width (`hud::draw_icon`).
    pub width: PlainLoose,
    /// The icon's alpha.
    pub alpha: PlainLoose,
    /// How far the icon drops below its row.
    pub drop: PlainLoose,
}

/// One view's marks (0x14 bytes from 0x6D14A2 and 0x6D14B6; `race::panel_marks`).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Marks {
    /// The first (at most 4) marks, each + 1.
    pub marks: [PlainLoose; 4],
    /// 400 + the car's +0xFA0.
    pub base: PlainLoose,
}

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Panel {
    /// +0x0 (0x6D13FE): `g_hudRecord`: the HUD record's first dword (the first car's split gap 6,
    /// `race::panel_values`).
    pub split_6: PlainLoose,
    /// +0x4 (0x6D1402): `unk_6D1402`: the first car's split gap 5.
    pub split_5: PlainLoose,
    /// +0x8 (0x6D1406): not named or used (a gap of 0x8 bytes).
    pub unused_8: PlainBytes<8>,
    /// +0x10 (0x6D140E): `unk_6D140E` .. : the first view's values.
    pub view_a: ViewA,
    /// +0x40 (0x6D143E): `unk_6D143E` .. : the second view's values.
    pub view_b: ViewB,
    /// +0x68 (0x6D1466): `g_overlayMiles`: the stage's length in miles (`race::init_overlay`).
    pub overlay_miles: PlainLoose,
    /// +0x6c (0x6D146A): `unk_6D146A`, `unk_6D146E`: each view's distance shown (`race::panel_values`; cleared by `race::init_overlay`).
    pub distance: [PlainLoose; 2],
    /// +0x74 (0x6D1472): `g_overlayParts`: the number of parts of the stage bar (`race::init_overlay`).
    pub overlay_parts: PlainLoose,
    /// +0x78 (0x6D1476): not named or used (a gap of 0xc bytes): the three arrays' places of the
    /// overlay (`g_overlayPartMiles`, `g_overlayPartKindsA`, `g_overlayPartKindsB`), kept as
    /// IDA's addresses, as names (nothing reads them).
    pub unused_78: PlainBytes<12>,
    /// +0x84 (0x6D1482): `unk_6D1482`, `unk_6D1492`: the two views' start lights.
    pub lights: [Lights; 2],
    /// +0xa4 (0x6D14A2): `unk_6D14A2` .. `unk_6D14C6`: the two views' marks.
    pub marks: [Marks; 2],
    /// +0xcc (0x6D14CA): `dword_6D14CA`, `dword_6D14CE`: each view's start value (`race::race_shell`).
    pub start_values: [PlainLoose; 2],
    /// +0xd4 (0x6D14D2): not named or used (a gap of 0x20 bytes).
    pub unused_d4: PlainBytes<32>,
    /// +0xf4 (0x6D14F2): `dword_6D14F2`: the HUD's layout (the record's +0xF4; `hud::overlay_pass`).
    pub players: PlainLoose,
    /// +0xf8 (0x6D14F6): `dword_6D14F6`, `dword_6D14FA`: each view's HUD flags (the car block's +0x220; bit 0x40 flat, 8 the mirror).
    pub hud_flags: [PlainLoose; 2],
    /// +0x100 (0x6D14FE): `dword_6D14FE`: the HUD batch's texture (`hud::add_polygon`, `hud::set_batch_texture`).
    pub shown_texture: PlainLoose,
    /// +0x104 (0x6D1502): `dword_6D1502`: the first view's HUD flags' bit 0x40, the flat layout (`race::draw_race_scene`).
    pub flat: PlainLoose,
    /// +0x108 (0x6D1506): not named or used (a gap of 0x2 bytes).
    pub unused_108: PlainBytes<2>,
    /// +0x10a (0x6D1508): `dword_6D1508`: the texture changes counted this frame (`race::frame_views`, `race::update_mud_texture`).
    pub texture_changes: PlainLoose,
}

/// IDA 0x6D13FE, the first byte of the piece.
pub const START: usize = 0x6D13FE;
/// IDA 0x6D150C, the end of the piece.
pub const END: usize = 0x6D150C;

const _: () = assert!(END - START == core::mem::size_of::<Panel>());

/// The piece's host address (the alignment checks).
#[cfg(test)]
pub fn piece_addr() -> usize {
    core::ptr::from_ref(&race().frame.panel_and_overlay.get().panel).addr()
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn fields_are_at_the_games_offsets() {
        assert_eq!(piece_addr() % core::mem::align_of::<Panel>(), 0);

        assert_eq!(offset_of!(Panel, split_6), 0x0);
        assert_eq!(offset_of!(Panel, view_a), 0x10);
        assert_eq!(offset_of!(Panel, view_b), 0x40);
        assert_eq!(offset_of!(Panel, overlay_miles), 0x68);
        assert_eq!(offset_of!(Panel, lights), 0x84);
        assert_eq!(offset_of!(Panel, marks), 0xa4);
        assert_eq!(offset_of!(Panel, start_values), 0xcc);
        assert_eq!(offset_of!(Panel, players), 0xf4);
        assert_eq!(offset_of!(Panel, flat), 0x104);
        assert_eq!(offset_of!(Panel, texture_changes), 0x10a);

        // The three places at +0x78 hold the image's addresses of the overlay's arrays (its
        // first values, `every_store_starts_as_the_games`): 0x100 bytes apart from the
        // overlay's first byte on.
        let unused = &race().frame.panel_and_overlay.get().panel.unused_78;
        for k in 0..3 {
            let name = unused.get_u32(4 * k) as usize;
            assert_eq!(name, crate::race::overlay_store::START + 0x100 * k);
        }
    }
}
