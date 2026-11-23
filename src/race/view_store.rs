//! The race's views: which are on, their cars and blocks, the drawn view, the race's cars and
//! players, the frame clock, and the constants the face drawing reads (the surface kinds, the
//! view's slope and near value, the corner order tables). Held on the game's thread; the drawing
//! parts read a copy of the few fields they use (`DrawnView`).

use crate::memory::session::{ConstFloat, PlainDword};

/// The face's surface kinds by kind byte (`race::course_faces`, `physics::wheel_surfaces`,
/// `race::best_line`); nothing writes them.
pub const SURFACE_KINDS: [u32; 4] = [3, 5, 4, 0];

/// The face's slope, the cone to the sides of the view's 4:3 (`race::course_faces`); nothing
/// writes it.
pub const SLOPE: ConstFloat = ConstFloat::new(1.2);

/// The face's near value (`race::course_faces`); nothing writes it.
pub const NEAR: ConstFloat = ConstFloat::new(1048576.0);

/// The fourth corner of the face drawn before any face was: the starting value
/// (`race::face_work::last_fourth_corner`).
pub const FIRST_FOURTH_CORNER: u32 = 0;

/// The level of the engine gauges from which they count (`drive::share_car_state`); nothing
/// writes it.
pub const LEVEL_FROM: ConstFloat = ConstFloat::new(75.0);

/// The turned corner orders, a byte table read by byte (`race::course_faces`).
pub const TURNED_CORNER_ORDER: [u32; 14] = [
    0x2030001, 0x5040706, 0x1000302, 0x6070405, 0x3020100, 0x7060504, 0x7060504, 0x10203,
    0x3020100, 0x7060504, 0x3020100, 0x7060504, 0x3020100, 0x7060504,
];

/// Byte `n` of the turned corner orders, none past them.
pub fn turned_corner_order_byte(n: usize) -> Option<u8> {
    TURNED_CORNER_ORDER
        .get(n / 4)
        .map(|word| word.to_le_bytes()[n % 4])
}

/// The corner texture offsets, four by order (`race::course_faces`).
pub const CORNER_UV_OFFSETS: [u32; 32] = [
    0, 4, 8, 0xC, 4, 0, 0xC, 8, 8, 0xC, 0, 4, 0xC, 8, 4, 0, 8, 0, 0xC, 4, 0xC, 4, 8, 0, 0, 8, 4,
    0xC, 4, 0xC, 0, 8,
];

/// The race's views and counts.
pub struct RaceView {
    /// Not 0 while a replay of a network race is viewed: the headlights then light the road
    /// for the drawn view's car (`race::frame_draw::lights`), as a network race's for its own.
    pub lights_follow_view: PlainDword,
    /// The lamps wanted; only the debug watch list reads it.
    pub lamps_wanted: PlainDword,
    /// Each view on (`race::draw_race_scene`, `race::show_replay_still`).
    pub view_on: [PlainDword; 2],
    /// The two players' option bits (`drive::replay_cameras`, `race::settings_copy`).
    pub player_options: [PlainDword; 2],
    /// The race's ticks run while below it (`drive::race_tick`; `race::credits_race`).
    pub after_credits: PlainDword,
    /// The view switch (`drive::choose_shown_cars`, `drive::replay_cameras`).
    pub view_switch: PlainDword,
    /// The race's cars.
    pub race_cars: PlainDword,
    /// The players (`settings.race_mode`; `[shaunnumplayers]`).
    pub players: PlainDword,
    /// The views shown (`race::draw_race_scene`'s one view; the second player's).
    pub views: PlainDword,
    /// The view being drawn.
    pub drawn_view: PlainDword,
    /// This machine's car (`net::lets_race`).
    pub local_car: PlainDword,
    /// Each view's car.
    pub view_cars: [PlainDword; 2],
    /// Each view's car block (by handle: `blocks_store::block_ref`; read through `view_block`).
    pub view_blocks: [PlainDword; 2],
    /// The first car of the group of players a replay's layout splits the screen between (the
    /// cars a group shows are the ones from it on: view `n` shows car `split_first + n`); 0 when
    /// no group is shown (`drive::choose_shown_cars`).
    pub split_first: PlainDword,
    /// The clock at the frame's start (`race::show_frame`).
    pub clock_base: PlainDword,
    /// The frame's time (`race::show_frame`).
    pub frame_time: PlainDword,
}

/// What the drawing parts read of the views, copied when a drawing scope opens.
#[derive(Clone, Copy)]
pub struct DrawnView {
    players: u32,
    race_cars: u32,
    lights_follow_view: u32,
    view_cars: [u32; 2],
    view_blocks: [u32; 2],
    split_first: u32,
}

/// The race's views as one reader finds them: the game's thread's own (read through the store
/// when used), or a drawing part's copy.
#[derive(Clone, Copy)]
pub enum DrawnViewRef<'a> {
    Store(&'a RaceView),
    Copy(&'a DrawnView),
}

impl<'a> From<&'a RaceView> for DrawnViewRef<'a> {
    fn from(view: &'a RaceView) -> Self {
        Self::Store(view)
    }
}

impl DrawnViewRef<'_> {
    /// The players whose parts the screen is split between (`RaceView::split_parts`).
    pub fn players(self) -> u32 {
        match self {
            Self::Store(v) => v.split_parts(),
            Self::Copy(v) => v.players,
        }
    }

    /// The race's cars (`RaceView::race_cars`).
    pub fn race_cars(self) -> u32 {
        match self {
            Self::Store(v) => v.race_cars.get(),
            Self::Copy(v) => v.race_cars,
        }
    }

    /// Not 0 while a network race's replay lights the road for the car followed
    /// (`RaceView::lights_follow_view`).
    pub fn lights_follow_view(self) -> u32 {
        match self {
            Self::Store(v) => v.lights_follow_view.get(),
            Self::Copy(v) => v.lights_follow_view,
        }
    }

    /// The car view `n` shows (`RaceView::view_car_number`).
    pub(super) fn view_car_number(self, n: usize) -> Result<u32, crate::errors::BadInput> {
        match self {
            Self::Store(v) => v.view_car_number(n),
            Self::Copy(v) => match v.view_cars.get(n) {
                Some(car) => Ok(*car),
                None if n < 4 => Ok(v.split_first + n as u32),
                None => Err(crate::errors::BadInput("a view past the four")),
            },
        }
    }

    /// The number of view `n`'s car block (`RaceView::view_number`).
    pub(super) fn view_number(self, n: usize) -> Result<usize, crate::errors::BadInput> {
        match self {
            Self::Store(v) => v.view_number(n),
            Self::Copy(v) => match v.view_blocks.get(n) {
                Some(block) => crate::race::blocks_store::number_of(*block),
                None if n < 4 => Ok(v.split_first as usize + n),
                None => Err(crate::errors::BadInput("a view past the four")),
            },
        }
    }

    /// The car of drawn view `drawn` (`RaceView::drawn_view_car`).
    pub fn drawn_view_car(self, drawn: u32) -> Result<u32, crate::errors::BadInput> {
        self.view_car_number(drawn as usize)
    }
}

/// The dwords of the record the surface kinds were the start of, from the face's depth test on
/// (layout dword 5), at their starting values; the fields still held are read from the store
/// (`RaceView::surface_kind`).
const AFTER_KINDS: [u32; 159] = [
    0, 0, 0, 0, 0, 0, 0, 0x3F99999A, 0x49800000, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x3020100, 0x7060504, 0x2030001, 0x5040706, 0x1000302,
    0x6070405, 0x3020100, 0x7060504, 0x7060504, 0x10203, 0x3020100, 0x7060504, 0x3020100,
    0x7060504, 0x3020100, 0x7060504, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 8, 0xC, 4, 0, 0xC, 8, 8, 0xC,
    0, 4, 0xC, 8, 4, 0, 8, 0, 0xC, 4, 0xC, 4, 8, 0, 0, 8, 4, 0xC, 4, 0xC, 0, 8, 0, 0x42960000, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xC0, 0xC0,
];

impl RaceView {
    pub const fn new() -> Self {
        Self {
            lights_follow_view: PlainDword::new(0),
            lamps_wanted: PlainDword::new(0),
            view_on: [const { PlainDword::new(0) }; 2],
            player_options: [const { PlainDword::new(0) }; 2],
            after_credits: PlainDword::new(0),
            view_switch: PlainDword::new(0),
            race_cars: PlainDword::new(0),
            players: PlainDword::new(1),
            views: PlainDword::new(0),
            drawn_view: PlainDword::new(0),
            local_car: PlainDword::new(0),
            view_cars: [PlainDword::new(0), PlainDword::new(1)],
            view_blocks: [
                PlainDword::new(crate::race::blocks_store::block_handle(0)),
                PlainDword::new(crate::race::blocks_store::block_handle(1)),
            ],
            split_first: PlainDword::new(0),
            clock_base: PlainDword::new(0),
            frame_time: PlainDword::new(0),
        }
    }

    /// How many players' parts the screen is split between: the views when there are more than
    /// two (a group of players of a replay, which may be more than the replay's players), else the
    /// players.
    pub fn split_parts(&self) -> u32 {
        let views = self.views.get();
        if views > 2 { views } else { self.players.get() }
    }

    /// What the drawing parts read, as it stands.
    pub fn drawn(&self) -> DrawnView {
        DrawnView {
            players: self.split_parts(),
            race_cars: self.race_cars.get(),
            lights_follow_view: self.lights_follow_view.get(),
            view_cars: std::array::from_fn(|n| self.view_cars[n].get()),
            view_blocks: std::array::from_fn(|n| self.view_blocks[n].get()),
            split_first: self.split_first.get(),
        }
    }
}

impl RaceView {
    /// The face's surface kind `n`. The index is a byte of the stage's texture record, not held to
    /// the four kinds: past them the dwords of the record's old layout after the table are read,
    /// in order, to its end.
    pub fn surface_kind(&self, n: usize) -> Result<u32, crate::errors::BadInput> {
        if let Some(kind) = SURFACE_KINDS.get(n) {
            return Ok(*kind);
        }
        // The dword's place in the old record, whose first dword was `lights_follow_view`.
        let place = n + 1;
        let pair = |cells: &[PlainDword; 2], first: usize| cells[place - first].get();
        let word = match place {
            99 => self.lamps_wanted.get(),
            106 | 107 => pair(&self.view_on, 106),
            108 | 109 => pair(&self.player_options, 108),
            110 => self.after_credits.get(),
            111 => self.view_switch.get(),
            113 => self.race_cars.get(),
            114 => self.players.get(),
            115 => self.views.get(),
            116 => self.drawn_view.get(),
            117 => self.local_car.get(),
            119 | 120 => pair(&self.view_cars, 119),
            121 | 122 => pair(&self.view_blocks, 121),
            155 => self.clock_base.get(),
            156 => self.frame_time.get(),
            _ => match AFTER_KINDS.get(n - 4) {
                Some(word) => *word,
                None => {
                    crate::memory::host_hooks::log(&format!(
                        "pointers: the surface kinds (a byte index past the table): dword {n}, past the {} dwords listed",
                        4 + AFTER_KINDS.len()
                    ));
                    return Err(crate::errors::BadInput(
                        "a dword past the fields of its record",
                    ));
                }
            },
        };
        Ok(word)
    }

    /// The number of view `n`'s car block, refused for none.
    pub(super) fn view_number(&self, n: usize) -> Result<usize, crate::errors::BadInput> {
        match self.view_blocks.get(n) {
            Some(block) => crate::race::blocks_store::number_of(block.get()),
            // The further views of a split screen show their own player's car.
            None if n < 4 => Ok(self.split_first.get() as usize + n),
            None => Err(crate::errors::BadInput("a view past the four")),
        }
    }

    /// The car view `n` shows: the first two views' (`view_cars`), and with a split screen of
    /// more than two players each further view's own player's. One past the four is refused.
    pub(super) fn view_car_number(&self, n: usize) -> Result<u32, crate::errors::BadInput> {
        match self.view_cars.get(n) {
            Some(car) => Ok(car.get()),
            None if n < 4 => Ok(self.split_first.get() + n as u32),
            None => Err(crate::errors::BadInput("a view past the four")),
        }
    }

    /// The car of drawn view `drawn` (`view_cars[drawn]`). The drawn view is a part's own view
    /// number (modulo 2 with a mirror's slot), or for the players' scenes `drawn_view`, which
    /// only `race::draw_race_scene` writes (the player 0 .. 3). One past the four is refused.
    pub fn drawn_view_car(&self, drawn: u32) -> Result<u32, crate::errors::BadInput> {
        self.view_car_number(drawn as usize)
    }
}

#[cfg(test)]
mod surface_kind_tests {
    use super::*;

    /// The four kinds are read by index, the dwords after them as the old record held them, and
    /// an index far past the record is refused.
    #[test]
    fn surface_kind_reads_the_kinds_and_refuses_far_past_them() {
        let view = RaceView::new();
        for (n, kind) in SURFACE_KINDS.iter().enumerate() {
            assert_eq!(view.surface_kind(n).ok(), Some(*kind), "index {n}");
        }
        // The record's players dword (layout dword 114) is dword 113 of the reach.
        view.players.set(7);
        assert_eq!(view.surface_kind(113).ok(), Some(7));
        // The first corner-order dword, a constant of the record's start (layout dword 44).
        assert_eq!(view.surface_kind(43).ok(), Some(0x2030001));
        // The last dword listed (layout dword 163), then one past it.
        assert_eq!(view.surface_kind(162).ok(), Some(0xC0));
        assert!(view.surface_kind(163).is_err());
        assert!(view.surface_kind(1 << 20).is_err());
    }
}
