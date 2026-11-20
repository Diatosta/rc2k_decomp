//! The scene's light work: the two masks the car's parts are switched by, and the draw mode and
//! light-turn switch. A plain, `Copy` record:
//! held by the scene owner (`scene::owner::Lights::work`) and copied into the drawing parts'
//! reads when a drawing scope opens.

/// The plain light colours, two colours of three channels (`scene::fill_light_tables`).
pub const PLAIN_LIGHT_COLOURS: [u32; 8] = [0, 0, 0, 0xFF, 0xFF, 0xFF, 0, 0];

#[derive(Clone, Copy)]
pub struct LightWork {
    /// The lamp parts' mask (`race::car_parts`, `race::draw_race_scene`, `race::loadlevel`).
    pub lamps_mask: u32,
    /// The lone-group switch / marks mask (`race::car_parts`, `physics::drop_broken_parts`,
    /// `scene::list_object_face`, `race::draw_race_scene`, `race::loadlevel`).
    pub marks_mask: u32,
    /// The draw mode: 0 normal (set by `scene::draw_car_object`, the carousel and the
    /// highlighted object); 1 lowers each projected point by a fixed amount
    /// (`scene::project_point`) and, with 2, skips the sky
    /// (`race::frame_draw::view::draw_sky`); non-0 changes how `scene::draw_object_face` and
    /// `scene::draw_scene_object` draw. Nothing sets 1 or 2 (probably).
    pub draw_mode: u32,
    /// `race::place_object_matrix`'s switch: 0 turns the object's matrix by the angles in
    /// the object work angles (y, x, z); non-0 uses the material rules (facing the view, turned
    /// by the view's angles) (`race::frame_draw::scene_objects`). Starts at 1.
    pub light_turn_off: u32,
}

impl LightWork {
    /// The store with its first values (both masks start at zero).
    pub const fn new() -> Self {
        Self {
            lamps_mask: 0,
            marks_mask: 0,
            draw_mode: 0,
            light_turn_off: 1,
        }
    }
}
