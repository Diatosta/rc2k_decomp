//! `scene::list_object_face` (0x4504CB): decides whether a face of an object is drawn,
//! and lists it if so.
//!
//! Takes: the face; its number within the object is in `dword_623620`
//!
//! `g_objectSeeThrough` = 0x100, then five tests, each of which can drop the face
//! ("its group" is the low five bits of the face's byte +7):
//! 1. 0x452292, only outside a race (`dword_5C5CEC` at 0): the object's kind
//!    `dword_5CD27C[object]` is looked for in the 0x22 entries of `dword_5C5790`, as
//!    an entry or as an entry plus 1 (`scene::find_object_kind`); if it is there, two group numbers
//!    of that place (`dword_5C90DC`, `dword_5C90E0` + place * 0x134) go to
//!    `dword_5CCE50` and `dword_5CCE54`, and a face of either group is dropped.
//! 2. 0x4505BD, only for objects from 0x50 up whose number divides by 0x14: dropped
//!    unless the group's bit is set in `dword_62150C`; with `dword_621870` at 5,
//!    dropped if `dword_621934[group]` is 0; else, with `dword_71A66C` at 0, dropped
//!    unless the group's bit is set in `dword_606280`; then by which group it is: the
//!    group `dword_5CCE60` is dropped while `dword_62DAD0` is 0; the groups
//!    `dword_5CCE50` and `dword_5CCE54` are dropped while `dword_6D18D4` is 1; the
//!    groups `dword_5CCE58` and `dword_5CCE5C` while it is 0.
//! 3. 0x450688: a face with bits 2 or 3 of its byte +8 set, while `dword_63996C`
//!    is not 0: dropped if `dword_639974` is 0, else that is its see-through value.
//! 4. 0x4506B7: a face of kind (byte +6) 0x1C is dropped outside a race, or if its
//!    byte +0xB is 0 or above 6, or if the dword that byte picks at +0x6FC of the
//!    block `off_71B3B8[dword_60627C]` is 0x80000000 or under 10; a face of another
//!    kind than 0 is dropped unless its byte +0xB is `dword_5CD688[kind]` (the drawing's
//!    `marks`: `scene::car_marks`).
//! 5. 0x45071B: dropped if `scene::face_faces_away`.
//!
//! A face that is left and has a vertex gets its key (`scene::face_sort_key`) and the next
//! entry of the list: its number, the key plus bit 30 unless bit 0 of its byte +8 is set,
//! `dword_624130`, the see-through value. One caller, `scene::list_object_faces`. Refuses an
//! object's number or a list's length past their tables, no block for kind 0x1C, and what its
//! parts refuse.

use crate::errors::{BadInput, Refused};
use crate::race::kind_store::KIND_TABLES;
use crate::race::light_records_store::LIGHT_RECORDS;
use crate::scene::face_faces_away::face_faces_away;
use crate::scene::face_list::FaceList;
use crate::scene::face_sort_key::face_sort_key;
use crate::scene::owner::scene;
use rc2k_core::scene::face_tests::{Face, FaceWorld, dropped, listed_key};

// ---- game data ----

const BLOCKS: u32 = 8;
const LIST_ROOM: u32 = 0x1000;

// ---- Rust version ----

/// The game's variables the tests read and write.
struct GameFace<'a>(
    &'a crate::race::owner::Race,
    crate::scene::reading::Reading,
    &'a crate::scene::car_marks::CarMarks,
    &'a crate::scene::object_place::ObjectPlace,
    &'a crate::race::car_draw_state::CarDrawState,
    &'a crate::physics::car_store::Car,
    &'a crate::race::view_vars_store::ViewVars,
);

impl FaceWorld for GameFace<'_> {
    fn in_race(&self) -> bool {
        self.0.course.numbers.in_race.bits() != 0
    }
    fn object(&self) -> u32 {
        self.4.object_number.bits()
    }
    fn object_kind(&self, object: u32) -> u32 {
        scene().objects.tables.object_kind(object)
    }
    fn known_kind(&self, place: usize) -> u32 {
        KIND_TABLES.known_object_kinds[place].get()
    }
    fn kind_groups(&self, place: usize) -> [u32; 2] {
        // `dword_5C90DC`, `dword_5C90E0`: groups [6] and [7] of the place's record.
        self.1.keep(
            LIGHT_RECORDS
                .record_ptr(place as u32)
                .offset(0x70)
                .read("a kind's record"),
        )
    }
    fn hidden_groups(&self) -> [u32; 2] {
        crate::session::read_first(&self.4.group_masks[13..15])
    }
    fn set_hidden_groups(&mut self, groups: [u32; 2]) {
        crate::session::write_first(&self.4.group_masks[13..15], &groups);
    }
    fn other_groups(&self) -> [u32; 2] {
        [15, 16].map(|k| self.4.group_masks[k].get())
    }
    fn group_mask(&self) -> u32 {
        self.4.shown_groups.bits()
    }
    fn mask_kind(&self) -> u32 {
        self.0.shared.switches.mask_kind.bits()
    }
    fn group_table(&self, group: u32) -> u32 {
        let draw_switches = &scene().draw_switches;
        // The table is typed by its first dword only.
        self.1.keep(crate::session::read_on_in(
            draw_switches,
            std::slice::from_ref(&draw_switches.group_table),
            group as usize,
            "the groups' table",
        ))
    }
    fn no_second_mask(&self) -> u32 {
        self.5.camera_switch_active.bits()
    }
    fn second_mask(&self) -> u32 {
        self.4.group_mask.bits()
    }
    fn lone_group(&self) -> u32 {
        self.4.group_masks[17].bits()
    }
    fn lone_group_on(&self) -> u32 {
        scene().lights.work.marks_mask.bits()
    }
    fn groups_which(&self) -> u32 {
        self.4.groups_which.bits()
    }
    fn see_through_on(&self) -> u32 {
        self.3.fading.bits()
    }
    fn see_through_value(&self) -> u32 {
        self.3.fade[0].bits()
    }
    fn set_see_through(&mut self, value: u32) {
        self.3.see_through.set_bits(value);
    }
    fn line_value(&self, extra: u32) -> Result<u32, rc2k_core::Invalid> {
        {
            let which: u32 = self.6.drawn_car.bits();
            let block: u32 = if which < BLOCKS {
                self.0.shared.work.car_block(which as usize)
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
        self.2.kind_byte(kind)
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
pub fn list_object_face(
    ctx: &crate::race::view_scratch::DrawCtx<'_>,
    face: usize,
    list: &mut FaceList,
    marks: &crate::scene::car_marks::CarMarks,
) -> Result<(), Refused> {
    let race = ctx.race;
    if face == 0 {
        return Err(BadInput("no face").into());
    }
    if list.len() as u32 >= LIST_ROOM {
        return Err(BadInput("no room in the list of faces").into());
    }
    ctx.view.object.see_through.set_bits(0x100u32);
    let bytes = face_bytes(face)?;
    let mut world = GameFace(
        race,
        crate::scene::reading::Reading::new(),
        marks,
        &ctx.view.object,
        &ctx.view.cars,
        &ctx.view.car,
        &ctx.view.vars,
    );
    let answer = dropped(&mut world, bytes);
    world.1.finish()?;
    match answer {
        Ok(true) => return Ok(()),
        Ok(false) => {}
        Err(why) => return Err(why.into()),
    }
    if face_faces_away(ctx, face)? {
        return Ok(());
    }
    let count: u32 = crate::scene::object_faces::word(face as u32)?;
    if count as i32 <= 0 {
        return Ok(());
    }
    ctx.view.polygon.polygon_vertex_count.set_bits(count);
    ctx.view.polygon.face_sort_key.set_bits(0u32);
    face_sort_key(&ctx.view.polygon, &race.course, face, count)?;
    // The original's room was 0x1000 entries but its list 0x400 long, the rest written over
    // the next tables (fixed: `wiki/Fixed-bugs.md`).
    let key = listed_key(bytes, ctx.view.polygon.face_sort_key.bits());
    list.push([
        ctx.view.face.face_number.bits(),
        key,
        ctx.view.polygon.entry_third.bits(),
        ctx.view.object.see_through.bits(),
    ]);
    Ok(())
}
