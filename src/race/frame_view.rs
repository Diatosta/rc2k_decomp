//! The cars as the race's drawing sees them: a snapshot of what the physics owns in each car
//! block, taken once at the start of a frame's drawing (`take`, from `draw_race_scene`). The
//! physics does not run while a frame is drawn, so the values are the blocks' own; what the
//! drawing keeps in the blocks itself (the parts' matrices, the lights' states, the map's
//! scroll, the skid slots, ...) and the values the two hand to each other (the notices, the
//! parts' damage limits) are not here and are still read from the blocks.
//!
//! With `RC2K_FRAME_VIEW_CHECK` the snapshot is compared with the blocks at the end of the
//! drawing: a field the drawing changed is logged (it would not belong here).

use crate::cells::Cells;
use crate::errors::BadInput;
use crate::physics::car_store::Car;
use crate::race::blocks_store::BLOCK_COUNT;
use crate::race::owner::Race;
use crate::session::{Dword, Float};
use std::sync::atomic::{AtomicBool, Ordering};

macro_rules! car_view {
    ($($(#[$doc:meta])* $field:ident: $t:ty,)*) => {
        /// One car's physics-owned fields (named as in `physics::car_store::Car`, on atomics:
        /// the drawing parts read them).
        #[repr(C)]
        #[derive(rc2k_derive::Cells)]
        pub struct CarView {
            $($(#[$doc])* pub $field: $t,)*
        }

        impl CarView {
            const fn new() -> Self {
                // Every field a cell or an array of cells: zero bits.
                crate::cells::zeroed_cells()
            }

            /// Every field from `car`.
            fn take(&self, car: &Car) {
                $(crate::cells::SameShape::copy_cells_to(&car.$field, &self.$field);)*
            }

            /// The fields that differ from `car`'s.
            fn changed(&self, car: &Car) -> Vec<&'static str> {
                let mut out = Vec::new();
                $(if !same(&self.$field, &car.$field) {
                    out.push(stringify!($field));
                })*
                out
            }
        }
    };
}

car_view! {
    position: [Dword; 3],
    car_kind: Dword,
    crash_matrix: [Float; 16],
    car_inverse_matrix: [Float; 16],
    camera_view: Dword,
    camera_mode_is_2_or_3: Dword,
    camera_switch_share: Float,
    option_bits: Dword,
    parts_present: Dword,
    driver_kind: Dword,
    lights_on: Dword,
    electrics_damage: Float,
    engine_speed: Float,
    engine_sound_speed_int: Dword,
    gear: Dword,
    checkpoint: Dword,
    car_heading: Dword,
    stage_time: Dword,
    time_left: Dword,
    run_ticks: Dword,
    splits_passed: Dword,
    split_times: [Dword; 16],
    split_gaps: [Dword; 7],
    pace_gaps_eased: [Dword; 2],
    mode2_place: Dword,
    false_start_count: Dword,
    car_event_position: Dword,
    car_event_count: Dword,
    car_event_codes: [Dword; 8],
    two_car_place: Dword,
    eased_value: Dword,
    step_clock: Dword,
    view_covered: Dword,
    tyre_change_state: Dword,
    light_colour: Dword,
    start_slot_value: Dword,
    drive_speed_a: Float,
    drive_speed_b: Float,
    brake: Dword,
    electrical_failures: [Dword; 7],
    steer_angle_visual: Dword,
    driven_wheel_turns: [Dword; 2],
    wheel_spring_force: [Dword; 4],
    wheel_squeeze: [Float; 4],
    skid_grip: [Float; 2],
    wheel_ground_x: [Dword; 4],
    wheel_ground_height: [Dword; 4],
    wheel_ground_z: [Dword; 4],
    surface_noise_a: Dword,
    surface_noise_b: Dword,
}

/// Two cells (or arrays of them, of either kind) with the same bits.
fn same<A: Cells, B: Cells>(a: &A, b: &B) -> bool {
    crate::cells::flatten(a) == crate::cells::flatten(b)
}

static CARS: [CarView; BLOCK_COUNT] = [const { CarView::new() }; BLOCK_COUNT];

/// A snapshot is held (between `take` and `release`).
static HELD: AtomicBool = AtomicBool::new(false);

/// The snapshot held while it lives: `take` at its making, `release` when dropped (on every
/// way out of the drawing).
pub struct Held<'a>(&'a Race);

impl<'a> Held<'a> {
    pub fn take(race: &'a Race) -> Self {
        take(race);
        Held(race)
    }
}

impl Drop for Held<'_> {
    fn drop(&mut self) {
        release(self.0);
    }
}

/// The snapshot of every car block (on the game's thread, before any view is drawn).
fn take(race: &Race) {
    let blocks = &race.car_blocks().cars;
    for (view, car) in CARS.iter().zip(blocks) {
        view.take(car);
    }
    HELD.store(true, Ordering::Release);
}

/// The drawing is over: the snapshot no longer read (and, with `RC2K_FRAME_VIEW_CHECK`, compared
/// with the blocks).
fn release(race: &Race) {
    HELD.store(false, Ordering::Release);
    static CHECK: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if !*CHECK.get_or_init(|| std::env::var_os("RC2K_FRAME_VIEW_CHECK").is_some()) {
        return;
    }
    let blocks = &race.car_blocks().cars;
    for (n, (view, car)) in CARS.iter().zip(blocks).enumerate() {
        let changed = view.changed(car);
        if !changed.is_empty() {
            crate::framework::log(&format!(
                "frame_view: the drawing changed car {n}'s {}",
                changed.join(", ")
            ));
        }
    }
}

/// Car `n`'s snapshot. Refused outside a frame's drawing (no snapshot held) or past the blocks.
pub fn car(n: usize) -> Result<&'static CarView, BadInput> {
    if !HELD.load(Ordering::Acquire) {
        return Err(BadInput("a car's snapshot read outside the race's drawing"));
    }
    CARS.get(n).ok_or(BadInput("a car past the blocks"))
}
