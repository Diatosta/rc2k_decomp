//! The best-line checkpoint place and the camera shake: plain values only the game's thread
//! reads and writes (`scene::owner::Scene::line_and_shake`).

/// The checkpoint the car is on, on the best line, and the camera's shake. The shake figures are
/// the bits of x87 floats, so a NaN's image is kept.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LineAndShake {
    /// The checkpoint the car is on, on the best line (`race::start_place`,
    /// `physics::reset_car`, `race::best_line`).
    pub line_place: u32,
    /// The fraction on to the next checkpoint, / 65536 (`race::start_place`,
    /// `physics::reset_car_on_line`).
    pub line_fraction: u32,
    /// The camera shake's first component, added to view A's / view B's camera offset
    /// (`physics::take_first_view`, `physics::take_second_view`). Never written (0 from the
    /// start): the shake is all in `camera_shake_y`. Probably x.
    pub camera_shake_x: u32,
    /// The camera shake's second component: the car's bump value (`vertical_speed_change`) times
    /// the physics table's scale, held within plus / minus its limit, 0 when the car stands still;
    /// added to the camera offsets. Probably y, the vertical (`physics::camera_frame`).
    pub camera_shake_y: u32,
    /// The camera shake's third component; never written (0 from the start), added to the
    /// camera offsets. Probably z.
    pub camera_shake_z: u32,
    /// The shake's strength for the sound: a random number / 65536 / 4096 times 3 times
    /// `camera_shake_y`; 0 outside the shaking camera modes. Read by `drive::set_listener`:
    /// added to the steering angle / 0x20000000 to make `listener_scale`
    /// (`physics::camera_frame`).
    pub camera_shake_listener: u32,
}

impl LineAndShake {
    /// The store with its first values (all zero).
    pub const fn new() -> Self {
        Self {
            line_place: 0,
            line_fraction: 0,
            camera_shake_x: 0,
            camera_shake_y: 0,
            camera_shake_z: 0,
            camera_shake_listener: 0,
        }
    }
}
