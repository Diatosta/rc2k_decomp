//! `scene::draw_carousel_entry` (0x4906D5): one entry of the main menu's carousel at
//! its place in the row.
//!
//! In:  ebx = the entry, ecx = its place in the row (-1 before the front one, 0 the
//!      front one, 1 and 2 after it)
//! Out: every register preserved
//!
//! The object's variables for `scene::draw_menu_object`:
//! - x (`dword_63501C`) = the spacing `dword_75A664` times (place - 1), plus the high
//!   half of the spacing times the turn `g_carouselAngle` inverted (the turn as a
//!   fraction of 2^32);
//! - the angle about y (`dword_635040`) from the clock: its low sixteen bits times 8,
//!   as sixteen bits; the angle about x (`dword_63503C`) = 0;
//! - y and z (`dword_635020`, `dword_635024`) = `dword_75A668`, `dword_75A66C`; the
//!   front entry (place 0) is brought forward by the high half of `dword_75A670` times
//!   the turn inverted, the next one (place 1) by that of the turn itself: the one
//!   leaving the front goes back as the one arriving comes forward;
//! - `dword_623FB0` and `dword_623FB4` = 0;
//! - the entry's kind `dword_75A674[entry]`, kept in `dword_75A658`; a kind above
//!   0x258 has a size, a y and an angle about x of its own in the records of twelve
//!   bytes at `unk_75950E` (by the kind less 0x259);
//! - `g_objectNumber` = the first object of that kind (`scene::find_object_of_kind`
//!   0x422298: objects 1 .. 0xFF, else 0).
//!
//! Then `scene::draw_menu_object`. Five call sites in `scene::draw_carousel`. Refuses an entry
//! of 0x100 or more and a kind past the records.

use crate::errors::{BadInput, Refused};
use crate::menu::lists_store::MENU_LISTS;
use crate::menu::state_store::MENU_STATE;
use crate::physics::tables_store::PHYSICS;
use crate::race::switches_store::FRAME_SWITCHES;
use crate::scene::draw_store::DRAW;
use crate::scene::object_store::OBJECT_WORK;

// ---- game data ----

pub(crate) const ENTRIES: u32 = 0x100;
/// The kinds' own places the original allows (`MENU_LISTS.own_places` holds 73: past them it
/// reads the lists that follow in the image, as the original does).
pub(crate) const OWN: u32 = 0x88;
// g_objectKinds

// ---- Rust version ----

/// What the function sets before it draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    place: [u32; 3],
    /// None: left as it is.
    size: Option<u32>,
    angle_x: u32,
    angle_y: u32,
    kind: u32,
    object: u32,
}

pub(crate) use crate::scene::find_object_of_kind::find_object_of_kind;

/// A kind's own place: its size, y and angle (`unk_75950E`, 12 bytes each). The original
/// reads on past the table's 73 for a kind below `OWN` (fixed: refused).
pub(crate) fn own_place(own: u32) -> Result<[u32; 3], BadInput> {
    let place = MENU_LISTS
        .own_places
        .get(own as usize)
        .ok_or(BadInput("a kind past the 73 own places"))?;
    Ok([place.size.get(), place.y.get(), place.angle.get()])
}

/// Reads only.
fn settings(entry: u32, row: u32) -> Result<Settings, BadInput> {
    if entry >= ENTRIES {
        return Err(BadInput("a carousel entry past the table"));
    }
    let [spacing, y, z, forward] = [
        MENU_STATE.carousel.spacing.get(),
        MENU_STATE.carousel.y.get(),
        MENU_STATE.carousel.z.get(),
        MENU_STATE.carousel.forward.get(),
    ];
    let turn: u32 = MENU_STATE.carousel.angle.get();
    let high = |a: u32, b: u32| ((a as u64 * b as u64) >> 32) as u32;
    let x = high(spacing, !turn)
        .wrapping_add((spacing as i32).wrapping_mul(row.wrapping_sub(1) as i32) as u32);
    let z = match row {
        0 => z.wrapping_sub(high(!turn, forward)),
        1 => z.wrapping_sub(high(turn, forward)),
        _ => z,
    };
    let clock: u32 = PHYSICS.scene_clock.bits();
    let mut set = Settings {
        place: [x, y, z],
        size: None,
        angle_x: 0,
        angle_y: ((clock as u16) << 3) as u32,
        kind: MENU_STATE.carousel.kinds[entry as usize].get(),
        object: 0,
    };
    if set.kind as i32 > 0x258 {
        let own = set.kind - 0x259;
        if own >= OWN {
            return Err(BadInput("a kind past the records of the kinds' own places"));
        }
        let [size, y, angle] = own_place(own)?;
        set.size = Some(size);
        set.place[1] = y;
        set.angle_x = angle;
    }
    set.object = find_object_of_kind(set.kind);
    Ok(set)
}

/// Carousel entry `entry` in row place `row` (ebx and ecx at the original's entry) set
/// up and drawn by `draw`.
pub fn draw_carousel_entry(
    entry: u32,
    row: u32,
    draw: &mut dyn FnMut() -> Result<(), Refused>,
) -> Result<(), Refused> {
    let set = settings(entry, row)?;
    crate::session::write_bits(&OBJECT_WORK.place, set.place);
    if let Some(size) = set.size {
        OBJECT_WORK.size.set_bits(size);
    }
    OBJECT_WORK.angles[0].set_bits(set.angle_x);
    OBJECT_WORK.angles[1].set_bits(set.angle_y);
    crate::session::write_bits(&DRAW.view_offset, [0u32; 2]);
    MENU_STATE.carousel.kind.set(set.kind);
    FRAME_SWITCHES.object_number.set_bits(set.object);
    draw()
}
