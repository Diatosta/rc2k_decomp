//! `scene::list_object_face` (0x4504CB): decides whether a face of an object is drawn,
//! and lists it if so.
//!
//! In:  ebp = the face; its number within the object is in `dword_623620`
//! Out: the original leaves registers changed; its one caller keeps them all, and
//!      the Rust version does not change any
//!
//! `g_objectSeeThrough` = 0x100, then five tests, each of which can drop the face
//! ("its group" is the low five bits of the face's byte +7):
//! 1. `sub_452292`, only outside a race (`dword_5C5CEC` at 0): the object's kind
//!    `dword_5CD27C[object]` is looked for in the 0x22 entries of `dword_5C5790`, as
//!    an entry or as an entry plus 1 (`sub_41F4BD`); if it is there, two group numbers
//!    of that place (`dword_5C90DC`, `dword_5C90E0` + place * 0x134) go to
//!    `dword_5CCE50` and `dword_5CCE54`, and a face of either group is dropped.
//! 2. `sub_4505BD`, only for objects from 0x50 up whose number divides by 0x14: dropped
//!    unless the group's bit is set in `dword_62150C`; with `dword_621870` at 5,
//!    dropped if `dword_621934[group]` is 0; else, with `dword_71A66C` at 0, dropped
//!    unless the group's bit is set in `dword_606280`; then by which group it is: the
//!    group `dword_5CCE60` is dropped while `dword_62DAD0` is 0; the groups
//!    `dword_5CCE50` and `dword_5CCE54` are dropped while `dword_6D18D4` is 1; the
//!    groups `dword_5CCE58` and `dword_5CCE5C` while it is 0.
//! 3. `sub_450688`: a face with bits 2 or 3 of its byte +8 set, while `dword_63996C`
//!    is not 0: dropped if `dword_639974` is 0, else that is its see-through value.
//! 4. `sub_4506B7`: a face of kind (byte +6) 0x1C is dropped outside a race, or if its
//!    byte +0xB is 0 or above 6, or if the dword that byte picks at +0x6FC of the
//!    block `off_71B3B8[dword_60627C]` is 0x80000000 or under 10; a face of another
//!    kind than 0 is dropped unless its byte +0xB is `dword_5CD688[kind]`.
//! 5. `sub_45071B`: dropped if `scene::face_faces_away`.
//!
//! A face that is left and has a vertex gets its key (`scene::face_sort_key`) and the next
//! entry of the list: its number, the key plus bit 30 unless bit 0 of its byte +8 is set,
//! `dword_624130`, the see-through value. One caller, `scene::list_object_faces`. Refuses an
//! object's number or a list's length past their tables, no block for kind 0x1C, and what its
//! parts refuse.

use crate::errors::{BadInput, Refused};
use crate::race::kind_store::KIND_TABLES;
use crate::race::light_records_store::LIGHT_RECORDS;
use crate::race::owner::race;
use crate::scene::face_faces_away::face_faces_away;
use crate::scene::face_sort_key::face_sort_key;
use crate::scene::owner::scene;
use rc2k_core::scene::face_tests::{Face, FaceWorld, dropped, listed_key};

// ---- game data ----

const BLOCKS: u32 = 8;
const LIST_ROOM: u32 = 0x1000;

// ---- Rust version ----

/// The game's variables the tests read and write.
struct GameFace(crate::scene::reading::Reading);

impl FaceWorld for GameFace {
    fn in_race(&self) -> bool {
        race().course.numbers.in_race.bits() != 0
    }
    fn object(&self) -> u32 {
        crate::race::owner::race()
            .shared
            .switches
            .object_number
            .bits()
    }
    fn object_kind(&self, object: u32) -> u32 {
        scene().objects.tables.object_kind(object)
    }
    fn known_kind(&self, place: usize) -> u32 {
        KIND_TABLES.known_object_kinds[place].get()
    }
    fn kind_groups(&self, place: usize) -> [u32; 2] {
        // `dword_5C90DC`, `dword_5C90E0`: groups [6] and [7] of the place's record.
        self.0.keep(
            LIGHT_RECORDS
                .record_ptr(place as u32)
                .offset(0x70)
                .read("a kind's record"),
        )
    }
    fn hidden_groups(&self) -> [u32; 2] {
        crate::session::read_first(&race().course.course.group_masks[13..15])
    }
    fn set_hidden_groups(&mut self, groups: [u32; 2]) {
        crate::session::write_first(&race().course.course.group_masks[13..15], &groups);
    }
    fn other_groups(&self) -> [u32; 2] {
        [15, 16].map(|k| race().course.course.group_masks[k].get())
    }
    fn group_mask(&self) -> u32 {
        crate::race::owner::race().shared.switches.group_mask.bits()
    }
    fn mask_kind(&self) -> u32 {
        crate::race::owner::race().shared.switches.mask_kind.bits()
    }
    fn group_table(&self, group: u32) -> u32 {
        let draw_switches = &scene().draw_switches;
        // The table is typed by its first dword only.
        self.0.keep(crate::session::read_on_in(
            draw_switches,
            std::slice::from_ref(&draw_switches.group_table),
            group as usize,
            "the groups' table",
        ))
    }
    fn no_second_mask(&self) -> u32 {
        crate::race::owner::race()
            .shared
            .car
            .camera_switch_active
            .bits()
    }
    fn second_mask(&self) -> u32 {
        race().frame.view_and_records.view_vars.group_mask.bits()
    }
    fn lone_group(&self) -> u32 {
        race().course.course.group_masks[17].bits()
    }
    fn lone_group_on(&self) -> u32 {
        scene().lights.work.marks_mask.bits()
    }
    fn groups_which(&self) -> u32 {
        race().frame.sound_spots.groups_which.bits()
    }
    fn see_through_on(&self) -> u32 {
        race().physics.ground.object_fading.bits()
    }
    fn see_through_value(&self) -> u32 {
        race().physics.ground.object_fade[0].bits()
    }
    fn set_see_through(&mut self, value: u32) {
        race().physics.ground.object_see_through.set_bits(value);
    }
    fn line_value(&self, extra: u32) -> Result<u32, rc2k_core::Invalid> {
        {
            let which: u32 = race().frame.view_and_records.view_vars.drawn_car.bits();
            let block: u32 = if which < BLOCKS {
                crate::race::owner::race()
                    .shared
                    .work
                    .car_block(which as usize)
            } else {
                0
            };
            if block == 0 {
                return Err(rc2k_core::Invalid("no block for a face of kind 0x1C"));
            }
            // The core asks for 1 ..= 6: the six parts' wake counts, after `state_bits`.
            crate::physics::car_store::Car::at(block)
                .part_wake_counts
                .get((extra as usize).wrapping_sub(1))
                .map(crate::session::Dword::bits)
                .ok_or(rc2k_core::Invalid("a wake count past the six"))
        }
    }
    fn kind_byte(&self, kind: u32) -> u32 {
        let objects = &scene().objects;
        self.0.keep(crate::session::read_on_in(
            &objects.tables,
            &objects.tables.car_mark_textures,
            kind as usize,
            "the car marks' textures",
        ))
    }
}

/// The face's bytes the tests read.
fn face_bytes(face: usize) -> Result<Face, BadInput> {
    let b: [u8; 0xC] = crate::scene::object_faces::array(face as u32)?;
    Ok(Face {
        kind: b[6],
        group: b[7],
        flags: b[8],
        extra: b[0xB],
    })
}

// Converted: `rc2k_core::scene::face_tests::{dropped, listed_key}`
pub fn list_object_face(race: &crate::race::owner::Race, face: usize) -> Result<(), Refused> {
    let scene = scene();
    let physics = &race.physics;
    if face == 0 {
        return Err(BadInput("no face").into());
    }
    let listed: u32 = scene.lights.work.listed_face_count.bits();
    if listed >= LIST_ROOM {
        return Err(BadInput("no room in the list of faces").into());
    }
    physics.ground.object_see_through.set_bits(0x100u32);
    let bytes = face_bytes(face)?;
    let mut world = GameFace(crate::scene::reading::Reading::new());
    let answer = dropped(&mut world, bytes);
    world.0.finish()?;
    match answer {
        Ok(true) => return Ok(()),
        Ok(false) => {}
        Err(why) => return Err(why.into()),
    }
    if face_faces_away(race, face)? {
        return Ok(());
    }
    let count: u32 = crate::scene::object_faces::word(face as u32)?;
    if count as i32 <= 0 {
        return Ok(());
    }
    scene.drawing.scene.polygon_vertex_count.set_bits(count);
    scene.drawing.scene.face_sort_key.set_bits(0u32);
    face_sort_key(&race.course, face, count)?;
    // The room is 0x1000 entries but 0x400 lie before the next table (a slip, kept).
    let entry = listed as usize * 4;
    let key = listed_key(bytes, scene.drawing.scene.face_sort_key.bits());
    for (k, v) in [
        crate::race::owner::race().shared.vars.face_number.bits(),
        key,
        scene.drawing.scene.entry_third.bits(),
        physics.ground.object_see_through.bits(),
    ]
    .into_iter()
    .enumerate()
    {
        crate::session::write_on_in(
            &scene.objects.slots,
            &scene.objects.slots.listed_faces,
            entry + k,
            v,
            "the list of faces",
        )?;
    }
    scene.lights.work.listed_face_count.set_bits(listed + 1);
    Ok(())
}
