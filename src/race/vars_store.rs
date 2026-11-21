//! The race's variables before the draw state: the race's settings as the race code reads them
//! (stage, direction, car kinds, tyres, run kind, split, settings words; `race::settings_copy`
//! fills them), the checkpoints and the best line's work, and the mud texture's state. Cells
//! read and written on the game's thread; the drawing parts read them through the lent store.

use crate::memory::session::{ConstFloat, PlainDword, PlainFloat};

/// A factor of the view's scale (`scene::set_view_window`); nothing writes it.
pub const SCALE_FACTOR: ConstFloat = ConstFloat::new(6.1035157e-06);

/// The fade texture coordinates (`scene::draw_polygon`, `race::skidmarks`); nothing writes them.
pub const FADE_UV: [u32; 2] = [0x3F78_0000, 0x3D00_0000];

/// The race variables the drawing parts read, copied when a drawing scope opens.
#[derive(Clone, Copy)]
pub struct DrawnVars {
    settings_words: [u32; 5],
    run_kind: u32,
    mud_thrown_count: u32,
    checkpoint_count: u32,
    split: u32,
    scene_switch: u32,
    stage_number: u32,
}

/// The race variables as one reader finds them: the game's thread's own (read through the store
/// when used), or a drawing part's copy.
#[derive(Clone, Copy)]
pub enum DrawnVarsRef<'a> {
    Store(&'a RaceVars),
    Copy(&'a DrawnVars),
}

impl DrawnVarsRef<'_> {
    /// The graphics option `n` (`RaceVars::settings_words`).
    pub fn settings_word(self, n: usize) -> u32 {
        match self {
            Self::Store(v) => v.settings_words[n].get(),
            Self::Copy(v) => v.settings_words[n],
        }
    }

    /// The race's run kind (`RaceVars::run_kind`).
    pub fn run_kind(self) -> u32 {
        match self {
            Self::Store(v) => v.run_kind.get(),
            Self::Copy(v) => v.run_kind,
        }
    }

    /// The first texture's thrown-mud count (`RaceVars::mud_thrown_counts`).
    pub fn mud_thrown_count(self) -> u32 {
        match self {
            Self::Store(v) => v.mud_thrown_counts[0].get(),
            Self::Copy(v) => v.mud_thrown_count,
        }
    }

    /// The checkpoint count (`RaceVars::checkpoint_count`).
    pub fn checkpoint_count(self) -> u32 {
        match self {
            Self::Store(v) => v.checkpoint_count.get(),
            Self::Copy(v) => v.checkpoint_count,
        }
    }
    /// The screen split / players (`RaceVars::split`).
    pub fn split(self) -> u32 {
        match self {
            Self::Store(v) => v.split.get(),
            Self::Copy(v) => v.split,
        }
    }

    /// The scene switch (`RaceVars::scene_switch`).
    pub fn scene_switch(self) -> u32 {
        match self {
            Self::Store(v) => v.scene_switch.get(),
            Self::Copy(v) => v.scene_switch,
        }
    }

    /// The stage number (`RaceVars::stage_number`).
    pub fn stage_number(self) -> u32 {
        match self {
            Self::Store(v) => v.stage_number.get(),
            Self::Copy(v) => v.stage_number,
        }
    }
}

pub struct RaceVars {
    /// A scene switch (`race::show_replay_still`; read by `physics::car_lighting`,
    /// `race::skid_laying`).
    pub scene_switch: PlainDword,
    /// `drive::pause_until_key`'s wait (0x1E, counted down).
    pub pause_wait: PlainDword,
    /// A scale put back at the race's end (`race::race_end`).
    pub kept_scale: PlainFloat,
    /// The race's tick count (`drive::race_tick`).
    pub race_ticks: PlainDword,
    /// Each car's tyres (the players' `settings.cars[n].tyre`); the car's gravel
    /// texture switch (`scene::load_texture_picture` reads eight).
    pub car_tyres: [PlainDword; 8],
    /// The best line's face total (`race::best_line`).
    pub line_face_total: PlainDword,
    /// The checkpoint count.
    pub checkpoint_count: PlainDword,
    /// The last checkpoint's number.
    pub last_checkpoint: PlainDword,
    /// The best line's face kind.
    pub line_face_kind: PlainDword,
    /// The replay's camera counts, 1 and 2.
    pub replay_camera_counts: [PlainDword; 2],
    /// The surface shares, as the whole numbers `race::surface_shares` leaves (it adds them
    /// as floats on its own).
    pub surface_shares: [PlainDword; 16],
    /// The map's anchor count.
    pub map_anchor_count: PlainDword,
    /// The stage number (`settings.stage`).
    pub stage_number: PlainDword,
    /// The stage's direction (`settings.direction`).
    pub direction: PlainDword,
    /// Each car's kind: [0] the current (`settings.car_kind_current`), [1] the first
    /// (`car_kind_first`), then the others.
    pub race_car_kinds: [PlainDword; 8],
    /// Each car's colours word (`net::set_up_race`; the stored blocks' gearbox kind; the
    /// players' `settings.cars[n].gearing`).
    pub car_colours: [PlainDword; 8],
    /// The textures' mip setting (`scene::set_filters`, `scene::try_allocate_textures`).
    pub mip_setting: PlainDword,
    /// The race's run kind (`settings.run_kind`; 2: the cars by place; splits, views,
    /// hidden kinds).
    pub run_kind: PlainDword,
    /// `settings.kept`: the screen split / players (`race::frame`, `hud::overlay_pass`, the
    /// gear choice).
    pub split: PlainDword,
    /// The graphics options (`settings.lens_flare`, `reflections`, `car_shadows`,
    /// `car_detail`, `headlights`; the ini keys `[shaunlensflare]`, `[shaunreflections]`,
    /// `[shauncarshadow]`, `[shauncardetail]`, `[shaunheadlights]`): [0] the lens flare (through
    /// `listener_figures[0]`, which gates and fades `effects::draw_lens_flare`), [1] the faces'
    /// shine, [2] car shadows, [3] see-through windows 0 / 1 / 2 (none, the viewed car, all cars),
    /// [4] all cars' lamps light objects.
    pub settings_words: [PlainDword; 5],
    /// The standings summary's block id (`settings.rivals`).
    pub rivals: PlainDword,
    /// Whether no replay is kept (`settings.no_replay`).
    pub no_replay: PlainDword,
    /// The race's level (`results.race_level`).
    pub race_level: PlainDword,
    /// The race's stage, less 1 (`results.race_stage`).
    pub race_stage: PlainDword,
    /// The cars' start numbers (`net::set_up_race`).
    pub start_numbers: [PlainDword; 8],
    /// The mud file's path, "files\b\mudfile.0xx" (`race::mud`; the digits start at byte 0x11).
    pub mud_path: crate::memory::plain_buffer::PlainBuffer<20>,
    /// [0] turned into [1] (`race::mud_texture_turn`).
    pub mud_turn_a: [PlainDword; 2],
    /// The same.
    pub mud_turn_b: [PlainDword; 2],
    /// The texture's thrown mud units (`race::update_mud_texture`).
    pub mud_throws: PlainDword,
    /// The texture's splash units (`race::update_mud_texture`).
    pub mud_splashes: PlainDword,
    /// The mud's splash colours.
    pub mud_colours: [PlainDword; 256],
    /// The extra throws' running sum (`race::throw_mud_texel`).
    pub mud_extra_sum: PlainDword,
    /// The packed mud state (`race::mud_texture_turn`).
    pub mud_packed: PlainDword,
    /// The splash's column count (`race::splash_mud`).
    pub mud_column: PlainDword,
    /// The splash's row count (`race::splash_mud`).
    pub mud_row: PlainDword,
    /// The texture locked (`race::update_mud_texture`).
    pub mud_locked: PlainDword,
    /// The first of the mud state's 18 dwords.
    pub mud_frames: PlainDword,
    /// The mud texture worked on.
    pub mud_texture: PlainDword,
    /// Each texture's thrown mud.
    pub mud_thrown_in: [PlainFloat; 4],
    /// Each texture's splashes.
    pub mud_splashes_in: [PlainFloat; 4],
    /// Each texture's thrown count.
    pub mud_thrown_counts: [PlainDword; 4],
    /// Each texture's splash count.
    pub mud_splash_counts: [PlainDword; 4],
    /// [0] the drive's, [1] the slip's (`physics::accumulate_mud`).
    pub mud_scale_a: [PlainFloat; 2],
    /// The same, the second.
    pub mud_scale_b: [PlainFloat; 2],
}

impl RaceVars {
    /// What the drawing parts read, as it stands.
    pub fn drawn(&self) -> DrawnVars {
        DrawnVars {
            settings_words: std::array::from_fn(|n| self.settings_words[n].get()),
            run_kind: self.run_kind.get(),
            mud_thrown_count: self.mud_thrown_counts[0].get(),
            checkpoint_count: self.checkpoint_count.get(),
            split: self.split.get(),
            scene_switch: self.scene_switch.get(),
            stage_number: self.stage_number.get(),
        }
    }

    pub const fn new() -> Self {
        Self {
            scene_switch: PlainDword::new(0),
            pause_wait: PlainDword::new(0),
            kept_scale: PlainFloat::new(384.0),
            race_ticks: PlainDword::new(0),
            car_tyres: [const { PlainDword::new(0) }; 8],
            line_face_total: PlainDword::new(0),
            checkpoint_count: PlainDword::new(0),
            last_checkpoint: PlainDword::new(0),
            line_face_kind: PlainDword::new(0),
            replay_camera_counts: [const { PlainDword::new(0) }; 2],
            surface_shares: [const { PlainDword::new(0) }; 16],
            map_anchor_count: PlainDword::new(0),
            stage_number: PlainDword::new(0),
            direction: PlainDword::new(0),
            race_car_kinds: [
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
            ],
            car_colours: [const { PlainDword::new(0) }; 8],
            mip_setting: PlainDword::new(0),
            run_kind: PlainDword::new(0),
            split: PlainDword::new(0),
            settings_words: [
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(0),
                PlainDword::new(0),
            ],
            rivals: PlainDword::new(0),
            no_replay: PlainDword::new(0),
            race_level: PlainDword::new(0),
            race_stage: PlainDword::new(0),
            start_numbers: [
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
                PlainDword::new(1),
            ],
            mud_path: crate::memory::plain_buffer::PlainBuffer::new([
                0x66, 0x69, 0x6C, 0x65, 0x73, 0x5C, 0x62, 0x5C, 0x6D, 0x75, 0x64, 0x66, 0x69, 0x6C,
                0x65, 0x2E, 0x30, 0x78, 0x78, 0,
            ]),
            mud_turn_a: [const { PlainDword::new(0) }; 2],
            mud_turn_b: [const { PlainDword::new(0) }; 2],
            mud_throws: PlainDword::new(0),
            mud_splashes: PlainDword::new(0),
            mud_colours: [const { PlainDword::new(0) }; 256],
            mud_extra_sum: PlainDword::new(0),
            mud_packed: PlainDword::new(0),
            mud_column: PlainDword::new(0),
            mud_row: PlainDword::new(0),
            mud_locked: PlainDword::new(0),
            mud_frames: PlainDword::new(0),
            mud_texture: PlainDword::new(0),
            mud_thrown_in: [const { PlainFloat::new(0.0) }; 4],
            mud_splashes_in: [const { PlainFloat::new(0.0) }; 4],
            mud_thrown_counts: [const { PlainDword::new(0) }; 4],
            mud_splash_counts: [const { PlainDword::new(0) }; 4],
            mud_scale_a: [const { PlainFloat::new(0.0) }; 2],
            mud_scale_b: [PlainFloat::new(0.075), PlainFloat::new(0.008)],
        }
    }
}
