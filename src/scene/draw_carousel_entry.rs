//! `scene::draw_carousel_entry`: one entry of the main menu's carousel at its place in the row.
//!
//! The object's variables for `scene::draw_menu_object`:
//! - x = the carousel's spacing times (place - 1), plus the high half of the spacing times the
//!   carousel's turn inverted (the turn as a fraction of 2^32);
//! - the angle about y from the clock: its low sixteen bits times 8, as sixteen bits; the
//!   angle about x = 0;
//! - y and z = the carousel's y and z; the front entry (place 0) is brought forward by the high
//!   half of the carousel's depth step times the turn inverted, the next one (place 1) by that
//!   of the turn itself: the one leaving the front goes back as the one arriving comes
//!   forward;
//! - the view window's offsets = 0;
//! - the entry's kind from the carousel's kinds, kept as the shown kind; a kind above 0x258
//!   has a size, a y and an angle about x of its own in the records of twelve bytes (by the
//!   kind less 0x259);
//! - the current object = the first object of that kind (`scene::find_object_of_kind`:
//!   objects 1 ..= 0xFF, else 0).
//!
//! Then `scene::draw_menu_object`. Five call sites in `scene::draw_carousel`. Refuses an entry
//! of 0x100 or more and a kind past the records.

use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::race::owner_shared::RaceShared;

// ---- game data ----

pub(crate) const ENTRIES: u32 = 0x100;
/// The kinds that have their own places (`own_places` holds 73 records: a kind
/// below this limit but past them is refused).
pub(crate) const OWN: u32 = 0x88;

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

/// A kind's own place: its size, y and angle (12 bytes each in the menu lists' own places).
/// A kind below `OWN` but past the table's 73 records is refused.
pub(crate) fn own_place(
    lists: &crate::menu::lists_store::MenuLists,
    own: u32,
) -> Result<[u32; 3], BadInput> {
    let place = *lists
        .read()
        .own_places
        .get(own as usize)
        .ok_or(BadInput("a kind past the 73 own places"))?;
    Ok([place.size, place.y, place.angle])
}

/// Reads only.
fn settings(
    shared: &RaceShared,
    scene: &crate::scene::owner::Scene,
    menus: &Menus,
    entry: u32,
    row: u32,
) -> Result<Settings, BadInput> {
    let screens = &menus.screens;
    if entry >= ENTRIES {
        return Err(BadInput("a carousel entry past the table"));
    }
    let [spacing, y, z, forward] = [
        screens.carousel.read().spacing,
        screens.carousel.read().y,
        screens.carousel.read().z,
        screens.carousel.read().forward,
    ];
    let turn: u32 = screens.carousel.read().angle;
    let high = |a: u32, b: u32| ((a as u64 * b as u64) >> 32) as u32;
    let x = high(spacing, !turn)
        .wrapping_add((spacing as i32).wrapping_mul(row.wrapping_sub(1) as i32) as u32);
    let z = match row {
        0 => z.wrapping_sub(high(!turn, forward)),
        1 => z.wrapping_sub(high(turn, forward)),
        _ => z,
    };
    let clock: u32 = shared.run().scene_clock;
    let mut set = Settings {
        place: [x, y, z],
        size: None,
        angle_x: 0,
        angle_y: ((clock as u16) << 3) as u32,
        kind: screens.carousel.read().kinds[entry as usize],
        object: 0,
    };
    if set.kind as i32 > 0x258 {
        let own = set.kind - 0x259;
        if own >= OWN {
            return Err(BadInput("a kind past the records of the kinds' own places"));
        }
        let [size, y, angle] = own_place(&menus.lists, own)?;
        set.size = Some(size);
        set.place[1] = y;
        set.angle_x = angle;
    }
    set.object = find_object_of_kind(set.kind, scene);
    Ok(set)
}

/// Carousel entry `entry` in row place `row` set
/// up and drawn by `draw`.
pub fn draw_carousel_entry(
    shared: &RaceShared,
    view: &crate::race::view_scratch::ViewScratch,
    menus: &Menus,
    scene: &crate::scene::owner::Scene,
    entry: u32,
    row: u32,
    draw: &mut dyn FnMut() -> Result<(), Refused>,
) -> Result<(), Refused> {
    let set = settings(shared, scene, menus, entry, row)?;
    view.object.borrow_mut().place = set.place;
    if let Some(size) = set.size {
        view.object.borrow_mut().size = size;
    }
    view.object.borrow_mut().angles[0] = set.angle_x;
    view.object.borrow_mut().angles[1] = set.angle_y;
    view.polygon.window.borrow_mut().view_offset = [0; 2];
    menus.screens.carousel.write().kind = set.kind;
    view.cars.borrow_mut().object_number = set.object;
    draw()
}
