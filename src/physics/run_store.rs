//! The race's clocks and the words the cars' controls pass through: plain fields held by the
//! game's thread (`race::owner_shared::RaceShared::run`). The drawing parts read the scene clock
//! from the copy their scope was opened with (`scene::workers::DrawReads::scene_clock`).

use rc2k_core::physics::gearbox::GearInputs;

/// The race clock, the recorded input words and the small counters of the race's loop.
#[derive(Clone, Debug)]
pub struct PhysicsRun {
    /// The scene's clock, read by everything timed.
    pub scene_clock: u32,
    /// Frames counted by `race::measure_frame_rate`.
    pub frame_rate_count: u32,
    /// The race clock's last reading (`drive::race_clock_tick`).
    pub race_clock_now: u32,
    /// The clock's count the race's time is taken from (`drive::race_clock_tick`, the films).
    pub race_clock_base: u32,
    /// The race's ticks to run (`drive::race_clock_tick` -> `drive::race_step`).
    pub race_tenths: u32,
    /// Three of the six are printed by the developers' wear page (`debug::wear_page`); nothing
    /// writes them.
    pub wear_page_words: [u32; 6],
    /// The four input words recorded and replayed (`drive::record_inputs`, `drive::play_inputs`).
    pub input_words: [u32; 4],
    /// A counter by camera view (`race::frame`); nothing writes them.
    pub view_counters: [u32; 7],
    /// The developers' debug overlay's page (0 .. 7), and the freeze-race flag (`drive::race_step`
    /// skips the physics ticks while set); `input::end_frame` sets both every frame
    /// (`debug::overlay`).
    pub debug: [u32; 2],
    /// The car's control inputs (`drive::read_car_controls`, `drive::steer_inputs`,
    /// `drive::pedal_inputs`), the gearbox's hold and the gear asked for: `joined`, -1 for none.
    pub gear_inputs: GearInputs,
    /// The noise's state (`scene::next_noise`).
    pub noise_state: u32,
}

impl PhysicsRun {
    /// The values the game starts with.
    pub const fn new() -> Self {
        Self {
            scene_clock: 0,
            frame_rate_count: 0,
            race_clock_now: 0,
            race_clock_base: 0,
            race_tenths: 0,
            wear_page_words: [0; 6],
            input_words: [0; 4],
            view_counters: [0x10C, 0x10D, 0x10E, 0x10F, 0x110, 0x111, 0x112],
            debug: [0; 2],
            gear_inputs: GearInputs {
                steer: 0,
                first: 0,
                second: 0,
                hold: 0,
                gear_up: 0,
                gear_down: 0,
                joined: 0,
            },
            noise_state: 0,
        }
    }
}
