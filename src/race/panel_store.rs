//! The race panel: the HUD record the race frame fills for the HUD (`hud::draw_hud` walks it from `g_hudRecord`, IDA 0x6D13FE, by address) and the variables around it: the two views' values (place, times, split gaps, motion), the overlay's miles and parts, the start lights, the marks, the HUD flags and the texture counter. Names are readings unless IDA's.
//! Most of its dwords lie at addresses that are not a multiple of 4 (the piece starts at 0x6D13FE): they are `Loose32` cells and the piece needs no alignment.

use crate::session::Bytes;

/// A dword at an address that is not a multiple of 4: plain unaligned volatile loads and stores, as the image's were.
#[repr(transparent)]
#[derive(rc2k_derive::Cells)]
pub struct Loose32(Bytes<4>);

impl Loose32 {
    pub fn get(&self) -> u32 {
        self.0.get_u32(0)
    }

    pub fn set(&self, value: u32) {
        self.0.set_u32(0, value);
    }

    pub fn addr(&self) -> u32 {
        self.0.addr()
    }
}

/// The first view's values for the panel (0x30 bytes from 0x6D140E; `race::panel_values`, `race::panel_motion`).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ViewA {
    /// The speed shown, rounded (`flt_6D140E`; the car's +0x718).
    pub whole: Loose32,
    /// The engine sound's speed (the car's +0xAB0).
    pub speed: Loose32,
    /// The gear (the car's +0x1D4).
    pub gear: Loose32,
    /// The place (`dword_6D141A`; `race::panel_players` and `race::panel_neighbours` set it, `race::panel_values` the car's split gap 0).
    pub place: Loose32,
    /// The stage time in tenths (the car's +0x88 x 10).
    pub time: Loose32,
    /// The car's +0xA14 (split gap 2; `race::panel_neighbours`: its +0x634).
    pub split2: Loose32,
    /// The car behind's number (`dword_6D1426`; the car's split gap 4 in `race::panel_values`).
    pub back: Loose32,
    /// Minus the car's split gap 1 (`dword_6D142A`).
    pub negated: Loose32,
    /// The car ahead's number (`dword_6D142E`; the car's split gap 3 in `race::panel_values`).
    pub front: Loose32,
    /// `flt_6D1432` ..: the map picture's two shifts and the heading (`race::panel_motion`).
    pub motion: [Loose32; 3],
}

/// The second view's values for the panel (0x28 bytes from 0x6D143E).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct ViewB {
    pub whole: Loose32,
    pub speed: Loose32,
    pub gear: Loose32,
    /// `dword_6D144A`.
    pub place: Loose32,
    pub time: Loose32,
    pub split2: Loose32,
    /// Minus the car's split gap 1 (`dword_6D1456`).
    pub negated: Loose32,
    /// `flt_6D145A` ..: as the first view's.
    pub motion: [Loose32; 3],
}

/// One view's start lights (0x10 bytes from 0x6D1482 and 0x6D1492; `race::countdown_lights`, `drive::countdown_beep`).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Lights {
    /// The lit count (`unk_6D1482`; -1 before the race: no extra texture).
    pub lit: Loose32,
    pub fade: Loose32,
    pub size: Loose32,
    pub shift: Loose32,
}

/// One view's marks (0x14 bytes from 0x6D14A2 and 0x6D14B6; `race::panel_marks`).
#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Marks {
    /// The first (at most 4) marks, each + 1.
    pub marks: [Loose32; 4],
    /// 400 + the car's +0xFA0.
    pub base: Loose32,
}

#[repr(C)]
#[derive(rc2k_derive::Cells)]
pub struct Panel {
    /// +0x0 (0x6D13FE): `g_hudRecord`: the HUD record's first dword (the first car's split gap 6, `race::panel_values`); `hud::draw_hud` walks the record from here by address.
    pub split_6: Loose32,
    /// +0x4 (0x6D1402): `unk_6D1402`: the first car's split gap 5.
    pub split_5: Loose32,
    /// +0x8 (0x6D1406): not named or used (a gap of 0x8 bytes).
    pub unknown_8: Bytes<8>,
    /// +0x10 (0x6D140E): `unk_6D140E` .. : the first view's values.
    pub view_a: ViewA,
    /// +0x40 (0x6D143E): `unk_6D143E` .. : the second view's values.
    pub view_b: ViewB,
    /// +0x68 (0x6D1466): `g_overlayMiles`: the stage's length in miles (`race::init_overlay`).
    pub overlay_miles: Loose32,
    /// +0x6c (0x6D146A): `unk_6D146A`, `unk_6D146E`: each view's distance shown (`race::panel_values`; cleared by `race::init_overlay`).
    pub distance: [Loose32; 2],
    /// +0x74 (0x6D1472): `g_overlayParts`: the number of parts of the stage bar (`race::init_overlay`).
    pub overlay_parts: Loose32,
    /// +0x78 (0x6D1476): not named or used (a gap of 0xc bytes): the three arrays' places of the
    /// overlay (`g_overlayPartMiles`, `g_overlayPartKindsA`, `g_overlayPartKindsB`), kept as
    /// IDA's addresses, as names (phase 13; nothing reads them, and `moved::point_into_homes`
    /// passes them over).
    pub unknown_78: Bytes<12>,
    /// +0x84 (0x6D1482): `unk_6D1482`, `unk_6D1492`: the two views' start lights.
    pub lights: [Lights; 2],
    /// +0xa4 (0x6D14A2): `unk_6D14A2` .. `unk_6D14C6`: the two views' marks.
    pub marks: [Marks; 2],
    /// +0xcc (0x6D14CA): `dword_6D14CA`, `dword_6D14CE`: each view's start value (`race::race_shell`).
    pub start_values: [Loose32; 2],
    /// +0xd4 (0x6D14D2): not named or used (a gap of 0x20 bytes).
    pub unknown_d4: Bytes<32>,
    /// +0xf4 (0x6D14F2): `dword_6D14F2`: the HUD's layout (the record's +0xF4; `hud::overlay_pass`).
    pub players: Loose32,
    /// +0xf8 (0x6D14F6): `dword_6D14F6`, `dword_6D14FA`: each view's HUD flags (the car block's +0x220; bit 0x40 flat, 8 the mirror).
    pub hud_flags: [Loose32; 2],
    /// +0x100 (0x6D14FE): `dword_6D14FE`: the HUD batch's texture (`hud::add_polygon`, `hud::set_batch_texture`).
    pub shown_texture: Loose32,
    /// +0x104 (0x6D1502): `dword_6D1502`: the first view's HUD flags' bit 0x40, the flat layout (`race::draw_race_scene`).
    pub flat: Loose32,
    /// +0x108 (0x6D1506): not named or used (a gap of 0x2 bytes).
    pub unknown_108: Bytes<2>,
    /// +0x10a (0x6D1508): `dword_6D1508`: the texture changes counted this frame (`race::frame_views`, `race::update_mud_texture`).
    pub texture_changes: Loose32,
}

/// The piece: a field of `race::groups_store::PANEL_AND_OVERLAY` since 2026-10-29 (it left the race
/// block with its group, phase 13).
pub struct InGroup;

impl core::ops::Deref for InGroup {
    type Target = Panel;
    fn deref(&self) -> &Panel {
        &crate::race::groups_store::PANEL_AND_OVERLAY.panel
    }
}

pub static PANEL: InGroup = InGroup;

/// IDA 0x6D13FE, the first byte of the piece.
pub const START: usize = 0x6D13FE;
/// IDA 0x6D150C, the end of the piece.
pub const END: usize = 0x6D150C;

const _: () = assert!(END - START == core::mem::size_of::<Panel>());

/// Where IDA 0x6D13FE is now (its own static).
#[cfg(test)]
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(
        &crate::race::groups_store::PANEL_AND_OVERLAY.panel,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn piece_sits_on_the_race_block() {
        assert_eq!(piece_addr() % (core::mem::align_of::<Panel>()) as u32, 0);
        assert_eq!(piece_addr() as usize, crate::moved::place(START));
        assert_eq!(
            crate::moved::place(END - 1),
            (piece_addr() + (END - START) as u32 - 1) as usize
        );

        assert_eq!(offset_of!(Panel, split_6), 0x6D13FE - START);
        assert_eq!(offset_of!(Panel, view_a), 0x6D140E - START);
        assert_eq!(offset_of!(Panel, view_b), 0x6D143E - START);
        assert_eq!(offset_of!(Panel, overlay_miles), 0x6D1466 - START);
        assert_eq!(offset_of!(Panel, lights), 0x6D1482 - START);
        assert_eq!(offset_of!(Panel, marks), 0x6D14A2 - START);
        assert_eq!(offset_of!(Panel, start_values), 0x6D14CA - START);
        assert_eq!(offset_of!(Panel, players), 0x6D14F2 - START);
        assert_eq!(offset_of!(Panel, flat), 0x6D1502 - START);
        assert_eq!(offset_of!(Panel, texture_changes), 0x6D1508 - START);

        // The three places at +0x78 are the image's addresses of the overlay's arrays, names
        // that resolve to them.
        let image = crate::moved::image_bytes(0x6D1476, 12);
        for (k, miles) in [0x6D150C_usize, 0x6D160C, 0x6D170C].into_iter().enumerate() {
            let name = u32::from_le_bytes(image[4 * k..4 * k + 4].try_into().unwrap());
            assert_eq!(name as usize, miles);
            assert_eq!(
                crate::moved::named_in(
                    name,
                    crate::race::overlay_store::START,
                    crate::race::overlay_store::END - crate::race::overlay_store::START,
                    "an overlay array off the overlay"
                ),
                Ok(crate::race::overlay_store::piece_addr()
                    + (miles - crate::race::overlay_store::START) as u32)
            );
        }
    }
}
