//! `scene::draw_marked_polygon` (0x44B9FC): `scene::draw_polygon` with a colour set
//! for the time of the call when the scene's flags say so.
//!
//! Out: as `scene::draw_polygon` (the original leaves every register changed; the Rust
//!      version keeps them)
//!
//! `dword_623DB8` = 0, or 0xFFFF00FF with the top bit of `dword_5CCCD4`; the polygon;
//! `dword_623DB8` = 0. The variable is read only by the line drawing at 0x44EB04 and 0x44EB1E
//! (not read: an outline for a developer's view, by the colour). Callers `sub_44CCEF` and
//! `sub_44CD90`.

use crate::errors::BadInput;
use crate::race::owner::Race;

// ---- game data ----
const MARK: u32 = 0xFFFF_00FF;

// ---- Rust version ----

pub fn draw_marked_polygon(
    race: &Race,
    draw: &mut dyn FnMut() -> Result<(), BadInput>,
) -> Result<(), BadInput> {
    let shared = &race.shared;
    {
        let marked = race.course.course.polygon_flags.bits() & 0x8000_0000 != 0;
        shared
            .vars
            .polygon_mark
            .set_bits(if marked { MARK } else { 0 });
        let done = draw();
        shared.vars.polygon_mark.set_bits(0u32);
        done
    }
}
