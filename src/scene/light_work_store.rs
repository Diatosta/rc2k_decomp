//! The scene's light work: the two masks the car's parts are switched by. A plain, `Copy` record:
//! held by the scene owner (`scene::owner::Lights::work`) and copied into the drawing parts'
//! reads when a drawing scope opens.

/// The plain light colours, two colours of three channels (`scene::fill_light_tables`).
pub const PLAIN_LIGHT_COLOURS: [u32; 8] = [0, 0, 0, 0xFF, 0xFF, 0xFF, 0, 0];

#[derive(Clone, Copy, Default)]
pub struct LightWork {
    /// The lamp parts' mask (`race::car_parts`, `race::draw_race_scene`, `race::loadlevel`).
    pub lamps_mask: u32,
    /// The lone-group switch / marks mask (`race::car_parts`, `physics::drop_broken_parts`,
    /// `scene::list_object_face`, `race::draw_race_scene`, `race::loadlevel`).
    pub marks_mask: u32,
}

impl LightWork {
    /// The store with its first values (both masks start at zero).
    pub const fn new() -> Self {
        Self {
            lamps_mask: 0,
            marks_mask: 0,
        }
    }
}
