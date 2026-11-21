//! The race panel: the HUD record the race frame fills for the HUD (read by the `hud::` drawers
//! by field) and the variables around it: the two views' values (place, times, split gaps,
//! motion), the overlay's miles and parts, the start lights, the marks, the HUD flags and the
//! texture counter. A plain record (`session::DrawPlain`): the game's thread reads and writes it
//! in short borrows; the drawing parts get a copy of the one field they read (`PanelRef`).

/// The first view's values for the panel (`panel_values_plain`,
/// `panel_motion_plain`).
#[derive(Clone, Copy)]
pub struct ViewA {
    /// The speed shown, rounded.
    pub whole: u32,
    /// The engine sound's speed.
    pub speed: u32,
    /// The car's gear.
    pub gear: u32,
    /// The place (`race::panel_players` and `race::panel_neighbours` set it, `panel_values_plain`
    /// the car's split gap 0).
    pub place: u32,
    /// The stage time in tenths (the car's `stage_time` x 10).
    pub time: u32,
    /// The car's `split_gaps[2]` (`race::panel_neighbours` sets it from the car's other
    /// gap figure).
    pub split2: u32,
    /// The car behind's number (the car's split gap 4 in `panel_values_plain`).
    pub back: u32,
    /// Minus the car's split gap 1.
    pub negated: u32,
    /// The car ahead's number (the car's split gap 3 in `panel_values_plain`).
    pub front: u32,
    /// The map picture's two shifts and the heading (`panel_motion_plain`).
    pub motion: [u32; 3],
}

/// The second view's values for the panel.
#[derive(Clone, Copy)]
pub struct ViewB {
    pub whole: u32,
    pub speed: u32,
    pub gear: u32,
    pub place: u32,
    pub time: u32,
    pub split2: u32,
    /// Minus the car's split gap 1.
    pub negated: u32,
    /// As the first view's.
    pub motion: [u32; 3],
}

/// One view's start lights (`race::countdown_lights`, `drive::countdown_beep`).
#[derive(Clone, Copy)]
pub struct Lights {
    /// The lit count (-1 before the race: no extra texture).
    pub lit: u32,
    /// The start-lights icon's width (`hud::draw_icon`).
    pub width: u32,
    /// The icon's alpha.
    pub alpha: u32,
    /// How far the icon drops below its row.
    pub drop: u32,
}

/// One view's marks (`race::panel_marks`).
#[derive(Clone, Copy)]
pub struct Marks {
    /// The first (at most 4) marks, each + 1.
    pub marks: [u32; 4],
    /// 400 plus the figure the car keeps for it.
    pub base: u32,
}

#[derive(Clone, Copy)]
pub struct Panel {
    /// The HUD record's first dword (the first car's split gap 6, `panel_values_plain`).
    pub split_6: u32,
    /// The first view's values.
    pub view_a: ViewA,
    /// The second view's values.
    pub view_b: ViewB,
    /// The stage's length in miles (`race::init_overlay`).
    pub overlay_miles: u32,
    /// Each view's distance shown (`panel_values_plain`; cleared by `race::init_overlay`).
    pub distance: [u32; 2],
    /// The number of parts of the stage bar (`race::init_overlay`).
    pub overlay_parts: u32,
    /// The cars of a network race, whose markers the stage bar shows beside the own one; 0
    /// outside a network race (set by `net::set_up_race`, cleared by `race::init_overlay`).
    pub network_cars: u32,
    /// The first view's car among them (its own marker is the stage bar's usual one).
    pub network_own: u32,
    /// Each car's distance shown (`panel_values_plain`, the same figure as `distance`), for the
    /// markers of a network race's other cars.
    pub car_distances: [u32; 8],
    /// The two views' start lights.
    pub lights: [Lights; 2],
    /// The two views' marks.
    pub marks: [Marks; 2],
    /// Each view's start value (`race::race_shell`).
    pub start_values: [u32; 2],
    /// The HUD's layout (`hud::overlay_pass`).
    pub players: u32,
    /// Each view's HUD flags (the car block's `option_bits`; bit 0x40 flat, 8 the mirror); the third and
    /// fourth are the split screen's third and fourth players'.
    pub hud_flags: [u32; 4],
    /// The HUD batch's texture (`hud::add_polygon`, `hud::set_batch_texture`).
    pub shown_texture: u32,
    /// The first view's HUD flags' bit 0x40, the flat layout (`race::draw_race_scene`).
    pub flat: u32,
    /// The texture changes counted this frame (`race::frame_views`,
    /// `race::update_mud_texture`).
    pub texture_changes: u32,
    /// Each player's values for a split screen of three or four players' HUD, in the first
    /// view's form (`race::frame_draw::panel_values_plain`; `hud::layout_many`).
    pub parts: [ViewA; 4],
    /// 1 while a split screen of three players shows the overview map in the part no player has
    /// (`race::overview_map`).
    pub overview_on: u32,
    /// 1 once the overview picture is drawn (cleared by `race::init_overlay`).
    pub overview_drawn: u32,
    /// The overview picture's fit: the map pixel at its top left corner (x, y) and the map pixels a
    /// picture pixel covers.
    pub overview_fit: [f32; 3],
    /// Each of the first four cars' place on the map, in map pixels (x, y).
    pub overview_cars: [[f32; 2]; 4],
    /// The cars of a network race whose markers the map disc shows beside the own car (bit n for
    /// car n; none outside a network race), and each car's place on the map from the first view's
    /// car, in map pixels (x right, y down).
    pub disc_cars: u32,
    pub disc_offsets: [[f32; 2]; 8],
    /// The same for the second to fourth views' discs: in a split screen the other players' cars,
    /// each disc placed from its own view's car.
    pub disc_cars_more: [u32; 3],
    pub disc_offsets_more: [[[f32; 2]; 8]; 3],
    /// The players of a split screen of three or four (0 otherwise): each part's stage bar shows
    /// all of their markers, from `car_distances`.
    pub split_players: u32,
    /// The third and fourth views' marks (a split screen of three or four): the pace-note signs
    /// their players' co-drivers call.
    pub marks_more: [Marks; 2],
}

impl ViewA {
    /// A view's values all zero.
    pub const ZERO: Self = Self {
        whole: 0,
        speed: 0,
        gear: 0,
        place: 0,
        time: 0,
        split2: 0,
        back: 0,
        negated: 0,
        front: 0,
        motion: [0; 3],
    };
}

impl ViewB {
    const ZERO: Self = Self {
        whole: 0,
        speed: 0,
        gear: 0,
        place: 0,
        time: 0,
        split2: 0,
        negated: 0,
        motion: [0; 3],
    };
}

impl Lights {
    /// A view's start lights before the race: nothing lit, no icon.
    const FIRST: Self = Self {
        lit: 0xFFFF_FFFF,
        width: 0,
        alpha: 0,
        drop: 0,
    };
}

impl Marks {
    const ZERO: Self = Self {
        marks: [0; 4],
        base: 0,
    };
}

impl Panel {
    /// View `view`'s marks (the first four views'; the third and fourth are the split screen's).
    pub fn marks_of_view(&self, view: usize) -> &Marks {
        match view {
            0 | 1 => &self.marks[view],
            _ => &self.marks_more[(view - 2).min(1)],
        }
    }

    /// The hud's panel at the start.
    pub const fn new() -> Self {
        Panel {
            split_6: 0,
            view_a: ViewA::ZERO,
            view_b: ViewB::ZERO,
            overlay_miles: 0x4270_0000,
            distance: [0x40A0_0000; 2],
            overlay_parts: 6,
            network_cars: 0,
            network_own: 0,
            car_distances: [0; 8],
            lights: [Lights::FIRST; 2],
            marks: [Marks::ZERO; 2],
            start_values: [0; 2],
            players: 0,
            hud_flags: [0; 4],
            shown_texture: 0,
            flat: 0,
            texture_changes: 0,
            parts: [ViewA::ZERO; 4],
            overview_on: 0,
            overview_drawn: 0,
            overview_fit: [0.0; 3],
            overview_cars: [[0.0; 2]; 4],
            disc_cars: 0,
            disc_offsets: [[0.0; 2]; 8],
            disc_cars_more: [0; 3],
            disc_offsets_more: [[[0.0; 2]; 8]; 3],
            split_players: 0,
            marks_more: [Marks::ZERO; 2],
        }
    }
}

/// The panel as one reader finds it: the game's thread's own (read through the store, short
/// borrows), or a drawing part's copy.
#[derive(Clone, Copy)]
pub enum PanelRef<'a> {
    Store(&'a crate::memory::session::DrawPlain<Panel>),
    Copy(&'a Panel),
}

impl PanelRef<'_> {
    /// Whether the first view's HUD is the flat layout.
    pub fn flat(self) -> u32 {
        match self {
            Self::Store(store) => store.borrow().flat,
            Self::Copy(record) => record.flat,
        }
    }
}

impl<'a> From<&'a crate::memory::session::DrawPlain<Panel>> for PanelRef<'a> {
    fn from(store: &'a crate::memory::session::DrawPlain<Panel>) -> Self {
        Self::Store(store)
    }
}

impl<'a> From<&'a crate::memory::session::PlainRead<Panel>> for PanelRef<'a> {
    fn from(copy: &'a crate::memory::session::PlainRead<Panel>) -> Self {
        Self::Copy(copy)
    }
}
