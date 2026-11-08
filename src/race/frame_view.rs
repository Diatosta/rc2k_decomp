//! The cars as the race's drawing sees them: a snapshot of what the physics owns in each car
//! block, taken once at the start of a frame's drawing (`take`, from `draw_race_scene`). The
//! physics does not run while a frame is drawn, so the values are the blocks' own; what the
//! drawing keeps in the blocks itself (the parts' matrices, the lights' states, the map's
//! scroll, the skid slots, ...) and the values the two hand to each other (the notices, the
//! parts' damage limits) are not here and are still read from the blocks.
//!
//! With `RC2K_FRAME_VIEW_CHECK` the snapshot is compared with the blocks at the end of the
//! drawing: a field the drawing changed is logged (it would not belong here).

use crate::errors::BadInput;
use crate::physics::car_store::Car;
use crate::pointers::Cells;
use crate::race::blocks_store::BLOCK_COUNT;
use crate::session::{Dword, Float};
use std::sync::atomic::{AtomicBool, Ordering};

macro_rules! car_view {
    ($($(#[$doc:meta])* $field:ident: $t:ty,)*) => {
        /// One car's physics-owned fields (named and typed as in `physics::car_store::Car`).
        #[repr(C)]
        #[derive(rc2k_derive::Cells)]
        pub struct CarView {
            $($(#[$doc])* pub $field: $t,)*
        }

        impl CarView {
            const fn new() -> Self {
                // Every field a cell or an array of cells: zero bits.
                crate::pointers::cells_of_bytes([0u8; core::mem::size_of::<CarView>()])
            }

            /// Every field from `car`.
            fn take(&self, car: &Car) {
                $(self.$field.copy_from(&car.$field);)*
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
    unknown_fa4: Dword,
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

/// Two cells (or arrays of them) with the same bits.
fn same<T: Cells>(a: &T, b: &T) -> bool {
    let n = core::mem::size_of::<T>();
    let (mut x, mut y) = (vec![0; n], vec![0; n]);
    crate::pointers::load_cells(a, 0, &mut x).is_ok()
        && crate::pointers::load_cells(b, 0, &mut y).is_ok()
        && x == y
}

static CARS: [CarView; BLOCK_COUNT] = [const { CarView::new() }; BLOCK_COUNT];

/// A snapshot is held (between `take` and `release`).
static HELD: AtomicBool = AtomicBool::new(false);

/// The snapshot held while it lives: `take` at its making, `release` when dropped (on every
/// way out of the drawing).
pub struct Held;

impl Held {
    pub fn take() -> Self {
        take();
        Held
    }
}

impl Drop for Held {
    fn drop(&mut self) {
        release();
    }
}

/// The snapshot of every car block (on the game's thread, before any view is drawn).
fn take() {
    let blocks = &crate::race::blocks_store::CAR_BLOCKS.cars;
    for (view, car) in CARS.iter().zip(blocks) {
        view.take(car);
    }
    HELD.store(true, Ordering::Release);
}

/// The drawing is over: the snapshot no longer read (and, with `RC2K_FRAME_VIEW_CHECK`, compared
/// with the blocks).
fn release() {
    HELD.store(false, Ordering::Release);
    static CHECK: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if !*CHECK.get_or_init(|| std::env::var_os("RC2K_FRAME_VIEW_CHECK").is_some()) {
        return;
    }
    let blocks = &crate::race::blocks_store::CAR_BLOCKS.cars;
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

/// The snapshot of the block at `address` (as `trap::car_at`; the work car or any other address
/// refused).
pub fn car_at(address: u32) -> Result<&'static CarView, BadInput> {
    car(Car::block_number(address).ok_or(BadInput("no car block's place for a snapshot"))?)
}
