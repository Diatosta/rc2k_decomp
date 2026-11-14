//! `scene::build_texture_catalogue`: gets memory for every texture that is needed, making the
//! textures smaller group by group until it is had.
//!
//! The listing is one list of 432 steps of a few kinds, which `scene/texture_steps.rs` holds
//! as a table (made by `tools/texture_catalogue`): log a text or a number; count a reduction in
//! the counters (0x100 of them); raise the level a tag's textures are needed at
//! (`scene::raise_texture_needs`: a higher level is a smaller texture); try
//! (`scene::try_allocate_textures`), which ends the list when it succeeds; release what a
//! failed try took (`gfx::release_all_textures`); and a group of steps skipped when a flag is
//! set.
//!
//! Before the list the counters are zeroed and the eight step flags set: 1 for the first, 0
//! for the others. So the list reads: try as things are; else your cars 2 .. 8 down from 256 to
//! 128, try; and so on through the other cars, the scenery, the sky, the front end, each time
//! one group one step smaller. After a successful try: "Allocated textures OK -", the needs
//! counted once more for the log, a rule. After the last failed one: "Cant allocate enough
//! textures" and fatal error 0x80600403. One caller, `scene::set_up_textures`.

use crate::errors::{FatalError, Refused};
use crate::logfile::log_int::log_int;
use crate::scene::count_texture_needs::count_texture_needs_in_game;
use crate::scene::raise_texture_needs::raise_texture_needs;
use crate::scene::texture_steps::{DONE, FAILED, STEPS};
use crate::scene::try_allocate_textures::try_allocate_textures_in_game;
use std::ffi::CStr;

// ---- game data ----

/// `TEXTURE_PLACES.counters` (0x100 dwords) and `step_flags` (which of the eight cars keep
/// their size).
const NOT_ENOUGH: FatalError = FatalError {
    code: 0x8060_0403,
    message: "",
};

/// A step of the list.
pub enum Step {
    Text(&'static CStr, bool),
    Number(u32, bool),
    Raise {
        level: u32,
        tag: u32,
    },
    /// One more reduction of this kind.
    Count(u32),
    Try,
    Release,
    CountNeeds,
    /// The steps, unless the flag (an index into `step_flags`) is set.
    Unless(usize, &'static [Step]),
}

/// What the function calls.
pub enum Call {
    Text(&'static CStr, bool),
    Number(u32, bool),
    Raise {
        level: u32,
        tag: u32,
    },
    /// Answers whether the memory was had.
    Try,
    Release,
    CountNeeds,
}

/// True when a try succeeded.
fn run(
    scene: &crate::scene::owner::Scene,
    steps: &'static [Step],
    call: &mut dyn FnMut(Call) -> Result<bool, Refused>,
) -> Result<bool, Refused> {
    let textures = &scene.textures;
    for step in steps {
        match *step {
            Step::Text(text, newline) => {
                call(Call::Text(text, newline))?;
            }
            Step::Number(number, newline) => {
                call(Call::Number(number, newline))?;
            }
            Step::Raise { level, tag } => {
                call(Call::Raise { level, tag })?;
            }
            Step::Count(index) => {
                let counter = crate::scene::texture_places_store::cell_at(
                    textures.places.get(),
                    crate::scene::texture_places_store::PlacesTable::Counters,
                    index,
                )?;
                counter.set(counter.get().wrapping_add(1));
            }
            Step::Try => {
                if call(Call::Try)? {
                    return Ok(true);
                }
            }
            Step::Release => {
                call(Call::Release)?;
            }
            Step::CountNeeds => {
                call(Call::CountNeeds)?;
            }
            Step::Unless(flag, inner) => {
                if textures.places.get().step_flags[flag].get() == 0 && run(scene, inner, call)? {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

pub fn build_texture_catalogue(
    call: &mut dyn FnMut(Call) -> Result<bool, Refused>,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let textures = &scene.textures;
    for counter in &textures.places.get().counters {
        counter.set(0);
    }
    for (n, flag) in textures.places.get().step_flags.iter().enumerate() {
        flag.set((n == 0) as u32);
    }
    if run(scene, &STEPS, call)? {
        run(scene, &DONE, call)?;
        return Ok(());
    }
    run(scene, &FAILED, call)?;
    Err(NOT_ENOUGH.into())
}

/// With the game's log.
pub fn build_texture_catalogue_in_game(
    shared: &crate::race::owner_shared::RaceShared,
    scene: &crate::scene::owner::Scene,
    gfx: &crate::gfx::owner::Gfx,
    d3d: &crate::d3d::owner::D3d,
) -> Result<(), Refused> {
    build_texture_catalogue(
        &mut |call| {
            match call {
                Call::Text(text, newline) => crate::logfile::log_write::log_nul(&text, newline, 1),
                Call::Number(number, newline) => log_int(number as i32, newline, 1),
                Call::Raise { level, tag } => {
                    raise_texture_needs(level, tag, scene);
                }
                Call::Try => return Ok(try_allocate_textures_in_game(shared, scene, gfx, d3d)?),
                Call::Release => crate::platform_3d::release_all_textures(gfx),
                Call::CountNeeds => {
                    count_texture_needs_in_game(scene)?;
                }
            }
            Ok(false)
        },
        scene,
    )
}
