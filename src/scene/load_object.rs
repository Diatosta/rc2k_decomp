//! `scene::load_object`: loads the shapes of one object slot from its kind's file
//! (`loading_slot` is the slot, `loading_kind` its kind).
//!
//! For a kind that is not 0 the slot's kind is noted, the file's names are made and the `.ro3`
//! one is read into the object file room (at most 0x40000 bytes); nothing more if nothing was
//! read. For a slot up to 0x40 the file's textures are listed. Then the shapes are worked out
//! from the file (`rc2k_core::scene::object_shape`) and put into the tables one after another
//! (`scene::add_object_shape`):
//!
//! - a slot under 0x50: the whole file;
//! - a car's slot (0x50 on, 0x14 slots a car: `loading_car` holds the car, the part and the
//!   car's first slot). The kind's place among the known kinds picks a light record whose
//!   eleven group numbers become the parts' masks.
//!   - part 0, the car's first slot, loads the whole car: five slots for the rest and the first
//!     four groups, and eleven more a group each, every one of those measured and its middle
//!     and half extents kept in six tables by slot (`part_extents`). The slot's number is left
//!     at the car's first + 0xF.
//!   - part 0xA takes the rest again, part 0xB the whole file; the other parts nothing (part 0
//!     loaded them).
//!
//! At the end three fatal errors: the faces' memory used up (0x80600011), the points' memory used
//! up (0x80600010), and 0xC000 faces or more (0x80600012).

use crate::errors::{FatalError, Refused};
use crate::files::read_whole_file::read_whole_file_plain_bytes;
use crate::memory::session::TextPlace;
use crate::physics::figures::GROUND_DROP;
use crate::scene::add_object_shape::add_object_shape;
use crate::scene::list_object_textures::list_object_textures;
use crate::scene::measure_object::measure_object;
use crate::scene::objects::{FACES, FILE_ROOM, SLOTS};
use crate::scene::set_object_file_names::set_object_file_names;
use rc2k_core::scene::object_shape::{
    PARTS, Parsed, ShapeRequest, car_masks, first_part_slots, parse_shape,
};
use rc2k_core::scene::objects::{Measures, measure};

// ---- game data ----

const KNOWN_KINDS: u32 = 0x22;
/// Where each part's group number is in the kind's light record.
const GROUPS: [u32; PARTS] = [
    0x00, 0x04, 0x08, 0x0C, 0x14, 0x34, 0x38, 0x2C, 0x30, 0x28, 0x3C,
];
/// The first slot a car's object is loaded into is 0x50; a car has 0x14 slots.
const FIRST_CAR_SLOT: u32 = 0x50;

const NO_POINT_MEMORY: FatalError = FatalError {
    code: 0x8060_0010,
    message: "out of object point memory",
};
const NO_FACE_MEMORY: FatalError = FatalError {
    code: 0x8060_0011,
    message: "out of object poly memory",
};
const NO_FACE_ROOM: FatalError = FatalError {
    code: 0x8060_0012,
    message: "out of poly ptr space - max 40960",
};

/// `scene::find_object_kind`: the place of a kind, or of the kind before
/// it, among the known ones; 0x22 if it is none of them.
fn find_object_kind(race: &crate::race::owner::Race, kind: u32) -> u32 {
    (0..KNOWN_KINDS)
        .find(|&n| {
            let known: u32 = race.kind_tables().known_object_kinds[n as usize];
            kind == known || kind == known.wrapping_add(1)
        })
        .unwrap_or(KNOWN_KINDS)
}

/// A shape worked out from the file, ready to be stored as `slot`.
struct Planned {
    slot: u32,
    parsed: Parsed,
    /// The middle and half extents of a measured part, when the shape has points to measure.
    measures: Option<Measures>,
    measured: bool,
}

pub fn load_object(
    scene: &crate::scene::owner::Scene,
    race: &crate::race::owner::Race,
) -> Result<(), Refused> {
    let kind: u32 = scene.object_work().loading_kind;
    if kind == 0 {
        return Ok(());
    }
    let slot: u32 = scene.object_work().loading_slot;
    if slot >= SLOTS {
        return Err(crate::errors::BadInput("an object's slot past the tables").into());
    }
    scene.objects.tables.borrow_mut().object_kinds[slot as usize] = kind;
    set_object_file_names(scene, kind)?;
    let name = scene.file_numbers().object_ro3_number;
    // The file read is the `.ro3` one, whatever the name's last character was.
    scene
        .path_text(name as usize + 7)
        .write_text_at(0, b"3", "an object's file name")?;
    let bytes_read = read_whole_file_plain_bytes(
        &scene
            .path_text(scene.file_numbers().object_ro3_path as usize)
            .text("an object's file path")?,
        crate::scene::objects::file(race),
    )?;
    if bytes_read == 0 {
        return Ok(());
    }
    scene
        .path_text(name as usize + 7)
        .write_text_at(0, b"3", "an object's file name")?;
    if slot <= 0x40 {
        list_object_textures(race, scene)?;
    }
    let file = crate::scene::objects::file(race).read_at(0, FILE_ROOM, "an object's file")?;
    let request = |group_mask: u32, all_points: bool| ShapeRequest {
        group_mask,
        all_points,
        turned: (0x190..0x258).contains(&kind),
        ground_drop: GROUND_DROP,
    };
    if slot < FIRST_CAR_SLOT {
        let planned = plan(&file, slot, request(u32::MAX, true), false)?;
        store(race, scene, &planned)?;
        return check(race, scene);
    }
    let known = find_object_kind(race, kind);
    // The kind's light record, from its groups; a kind none of the known ones takes the place
    // after the 34 records, which reads on into the gear figures (`group_at`).
    let mut groups = [u32::MAX; PARTS];
    for (group, &offset) in groups.iter_mut().zip(&GROUPS) {
        *group = race
            .light_records()
            .group_at(race.physics.gear(), known, offset)?;
    }
    let masks = car_masks(groups, known);
    let (car, part) = (
        (slot - FIRST_CAR_SLOT) / 0x14,
        (slot - FIRST_CAR_SLOT) % 0x14,
    );
    let first = car * 0x14 + FIRST_CAR_SLOT;
    scene.object_work_mut().loading_car = [car, part, first];
    match part {
        0 => {
            for car_slot in first_part_slots(&masks) {
                let planned = plan(
                    &file,
                    first + car_slot.offset,
                    request(car_slot.mask, false),
                    car_slot.measured,
                )?;
                store(race, scene, &planned)?;
            }
        }
        0xA => {
            let planned = plan(&file, slot, request(masks.rest, false), false)?;
            store(race, scene, &planned)?;
        }
        0xB => {
            let planned = plan(&file, slot, request(u32::MAX, true), false)?;
            store(race, scene, &planned)?;
        }
        _ => {}
    }
    check(race, scene)
}

/// Works out the shape of `slot` from the file; a measured part's points are moved so that
/// their middle is at 0.
fn plan(file: &[u8], slot: u32, request: ShapeRequest, measured: bool) -> Result<Planned, Refused> {
    let mut parsed = parse_shape(file, request)?;
    let mut measures = None;
    if let Parsed::Shape(shape) = &mut parsed {
        shape.measure_reach(crate::math::x87::int_hypot);
        if measured {
            measures = measure(&mut shape.points);
        }
    }
    Ok(Planned {
        slot,
        parsed,
        measures,
        measured,
    })
}

/// Stores a planned shape and, for a measured part, its extents.
fn store(
    race: &crate::race::owner::Race,
    scene: &crate::scene::owner::Scene,
    planned: &Planned,
) -> Result<(), Refused> {
    add_object_shape(race, scene, planned.slot, &planned.parsed)?;
    if !planned.measured {
        return Ok(());
    }
    match &planned.measures {
        Some(m) => scene
            .loaded_slots_mut()
            .measures
            .copy_from_slice(&[m.middle, m.half, m.lowest, m.highest].concat()),
        // A file of no points, or none taken: the object's own points are measured, which
        // leaves the half extents as they were.
        None => measure_object(&race.course, scene)?,
    }
    let mut slots = scene.loaded_slots_mut();
    for table in 0..6 {
        slots.part_extents[table][planned.slot as usize] = slots.measures[table];
    }
    Ok(())
}

/// The three fatal errors at the end.
fn check(
    race: &crate::race::owner::Race,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let course = &race.course;
    if scene.object_work().face_cursor > course.lists.get().face_memory_room.get() {
        return Err(NO_FACE_MEMORY.into());
    }
    if scene.objects.tables.borrow().point_count.wrapping_mul(12)
        > course.lists.get().object_points_room.get()
    {
        return Err(NO_POINT_MEMORY.into());
    }
    if scene.texture_levels().face_count >= FACES {
        return Err(NO_FACE_ROOM.into());
    }
    Ok(())
}
