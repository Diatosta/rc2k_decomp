//! The race panel: the HUD record the race frame fills for the HUD (read by the `hud::` drawers
//! by field) and the variables around it: the two views' values (place, times, split gaps,
//! motion), the overlay's miles and parts, the start lights, the marks, the HUD flags and the
//! texture counter.

use crate::memory::cells::ZeroCells;
use crate::memory::session::{PlainBytes, PlainDword, PlainFloat};

/// The first view's values for the panel (0x30 bytes; `panel_values_plain`,
/// `panel_motion_plain`).
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct ViewA {
    /// The speed shown, rounded (the car's +0x718).
    pub whole: PlainDword,
    /// The engine sound's speed (the car's +0xAB0).
    pub speed: PlainDword,
    /// The gear (the car's +0x1D4).
    pub gear: PlainDword,
    /// The place (`race::panel_players` and `race::panel_neighbours` set it, `panel_values_plain`
    /// the car's split gap 0).
    pub place: PlainDword,
    /// The stage time in tenths (the car's +0x88 x 10).
    pub time: PlainDword,
    /// The car's +0xA14 (split gap 2; `race::panel_neighbours`: its +0x634).
    pub split2: PlainDword,
    /// The car behind's number (the car's split gap 4 in `panel_values_plain`).
    pub back: PlainDword,
    /// Minus the car's split gap 1.
    pub negated: PlainDword,
    /// The car ahead's number (the car's split gap 3 in `panel_values_plain`).
    pub front: PlainDword,
    /// The map picture's two shifts and the heading (`panel_motion_plain`).
    pub motion: [PlainDword; 3],
}

/// The second view's values for the panel (0x28 bytes).
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct ViewB {
    pub whole: PlainDword,
    pub speed: PlainDword,
    pub gear: PlainDword,
    pub place: PlainDword,
    pub time: PlainDword,
    pub split2: PlainDword,
    /// Minus the car's split gap 1.
    pub negated: PlainDword,
    /// As the first view's.
    pub motion: [PlainDword; 3],
}

/// One view's start lights (0x10 bytes; `race::countdown_lights`, `drive::countdown_beep`).
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct Lights {
    /// The lit count (-1 before the race: no extra texture).
    pub lit: PlainDword,
    /// The start-lights icon's width (`hud::draw_icon`).
    pub width: PlainDword,
    /// The icon's alpha.
    pub alpha: PlainDword,
    /// How far the icon drops below its row.
    pub drop: PlainDword,
}

/// One view's marks (0x14 bytes; `race::panel_marks`).
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct Marks {
    /// The first (at most 4) marks, each + 1.
    pub marks: [PlainDword; 4],
    /// 400 + the car's +0xFA0.
    pub base: PlainDword,
}

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct Panel {
    /// The HUD record's first dword (the first car's split gap 6, `panel_values_plain`).
    pub split_6: PlainDword,
    /// The first car's split gap 5.
    pub split_5: PlainDword,
    /// Unused (a gap of 0x8 bytes).
    pub unused_8: PlainBytes<8>,
    /// The first view's values.
    pub view_a: ViewA,
    /// The second view's values.
    pub view_b: ViewB,
    /// The stage's length in miles (`race::init_overlay`).
    pub overlay_miles: PlainDword,
    /// Each view's distance shown (`panel_values_plain`; cleared by `race::init_overlay`).
    pub distance: [PlainDword; 2],
    /// The number of parts of the stage bar (`race::init_overlay`).
    pub overlay_parts: PlainDword,
    /// The cars of a network race, whose markers the stage bar shows beside the own one; 0
    /// outside a network race (set by `net::set_up_race`, cleared by `race::init_overlay`).
    pub network_cars: PlainDword,
    /// The first view's car among them (its own marker is the stage bar's usual one).
    pub network_own: PlainDword,
    /// Each car's distance shown (`panel_values_plain`, the same figure as `distance`), for the
    /// markers of a network race's other cars.
    pub car_distances: [PlainDword; 8],
    /// Unused (a gap of 0xc bytes): the places of the overlay's three arrays
    /// (`overlay_part_miles`, `overlay_part_kinds_a`, `overlay_part_kinds_b`); nothing reads them.
    pub unused_78: PlainBytes<12>,
    /// The two views' start lights.
    pub lights: [Lights; 2],
    /// The two views' marks.
    pub marks: [Marks; 2],
    /// Each view's start value (`race::race_shell`).
    pub start_values: [PlainDword; 2],
    /// Unused (a gap of 0x20 bytes).
    pub unused_d4: PlainBytes<32>,
    /// The HUD's layout (`hud::overlay_pass`).
    pub players: PlainDword,
    /// Each view's HUD flags (the car block's +0x220; bit 0x40 flat, 8 the mirror); the third and
    /// fourth are the split screen's third and fourth players'.
    pub hud_flags: [PlainDword; 4],
    /// The HUD batch's texture (`hud::add_polygon`, `hud::set_batch_texture`).
    pub shown_texture: PlainDword,
    /// The first view's HUD flags' bit 0x40, the flat layout (`race::draw_race_scene`).
    pub flat: PlainDword,
    /// Unused (a gap of 0x2 bytes).
    pub unused_108: PlainBytes<2>,
    /// The texture changes counted this frame (`race::frame_views`,
    /// `race::update_mud_texture`).
    pub texture_changes: PlainDword,
    /// Each player's values for a split screen of three or four players' HUD, in the first
    /// view's form (`race::frame_draw::panel_values_plain`; `hud::layout_many`).
    pub parts: [ViewA; 4],
    /// 1 while a split screen of three players shows the overview map in the part no player has
    /// (`race::overview_map`).
    pub overview_on: PlainDword,
    /// 1 once the overview picture is drawn (cleared by `race::init_overlay`).
    pub overview_drawn: PlainDword,
    /// The overview picture's fit: the map pixel at its top left corner (x, y) and the map pixels a
    /// picture pixel covers.
    pub overview_fit: [PlainFloat; 3],
    /// Each of the first four cars' place on the map, in map pixels (x, y).
    pub overview_cars: [[PlainFloat; 2]; 4],
    /// The cars of a network race whose markers the map disc shows beside the own car (bit n for
    /// car n; none outside a network race), and each car's place on the map from the first view's
    /// car, in map pixels (x right, y down).
    pub disc_cars: PlainDword,
    pub disc_offsets: [[PlainFloat; 2]; 8],
    /// The same for the second to fourth views' discs: in a split screen the other players' cars,
    /// each disc placed from its own view's car.
    pub disc_cars_more: [PlainDword; 3],
    pub disc_offsets_more: [[[PlainFloat; 2]; 8]; 3],
    /// The players of a split screen of three or four (0 otherwise): each part's stage bar shows
    /// all of their markers, from `car_distances`.
    pub split_players: PlainDword,
}

impl Panel {
    /// The hud's panel at the start.
    pub const fn new() -> Self {
        Panel {
            overlay_miles: PlainDword::new(0x42700000),
            distance: [const { PlainDword::new(0x40A00000) }; 2],
            overlay_parts: PlainDword::new(6),
            unused_78: PlainBytes::new([
                0xC, 0x15, 0x6D, 0, 0xC, 0x16, 0x6D, 0, 0xC, 0x17, 0x6D, 0,
            ]),
            lights: [const {
                Lights {
                    lit: PlainDword::new(0xFFFFFFFF),
                    ..ZeroCells::ZERO
                }
            }; 2],
            unused_d4: PlainBytes::new([
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xA0, 0x40, 0, 0, 0xA0, 0x40, 0, 0, 0xA0, 0x40, 0, 0,
                0xA0, 0x40, 0, 0, 0xA0, 0x40, 0, 0, 0xA0, 0x40,
            ]),
            ..ZeroCells::ZERO
        }
    }
}
