//! `scene::list_object_face` (0x4504CB): decides whether a face of an object is drawn,
//! and lists it if so.
//!
//! Takes: the face; its number within the object is the view's `face_number`.
//!
//! The face's see-through value starts at 0x100, then five tests (in
//! `rc2k_core::scene::face_tests::dropped`) can each drop the face ("its group" is the low
//! five bits of the face's byte +7):
//! 1. Only outside a race: the object's kind is looked for among the known object kinds,
//!    as an entry or as an entry plus 1 (`scene::find_object_kind`); if it is there, two
//!    group numbers of that kind's light record become the hidden groups, and a face of
//!    either group is dropped.
//! 2. Only for objects from 0x50 up whose number divides by 0x14: dropped unless the
//!    group's bit is set in the shown groups; in group-mask kind 5, dropped if the group's
//!    table entry is 0; else, with the camera switch off, dropped unless the group's bit is
//!    set in the second mask; then by which group it is: the lone group is dropped while its
//!    mask is off, the hidden groups while `groups_which` is 1, the other groups while it
//!    is 0.
//! 3. A face with bits 2 or 3 of its byte +8 set, while the object is fading: dropped if
//!    the fade value is 0, else that is its see-through value.
//! 4. A face of kind (byte +6) 0x1C is dropped outside a race, or if its byte +0xB is 0 or
//!    above 6, or if the wake count of that car part is 0x80000000 or under 10; a face of
//!    another kind than 0 is dropped unless its byte +0xB is the kind byte the car's marks
//!    hold for it (`scene::car_marks`).
//! 5. Dropped if `scene::face_faces_away`.
//!
//! A face that is left and has a vertex gets its key (`scene::face_sort_key`) and the next
//! entry of the list: its number, the key plus bit 30 unless bit 0 of its byte +8 is set,
//! the polygon's `entry_third` and the see-through value. One caller, `scene::list_object_faces`.
//! Refuses an object's number or a list's length past their tables, no block for kind 0x1C,
//! and what its parts refuse.

use crate::errors::{BadInput, Refused};
use crate::race::kind_store::KIND_TABLES;
use crate::race::light_records_store::LIGHT_RECORDS;
use crate::scene::face_faces_away::face_faces_away;
use crate::scene::face_list::FaceList;
use crate::scene::face_sort_key::face_sort_key;
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
    &'a crate::scene::owner::Scene,
);

impl FaceWorld for GameFace<'_> {
    fn in_race(&self) -> bool {
        self.0.course.numbers.in_race.is_set()
    }
    fn object(&self) -> u32 {
        self.4.object_number.get()
    }
    fn object_kind(&self, object: u32) -> u32 {
        self.7.objects.tables.object_kind(object)
    }
    fn known_kind(&self, place: usize) -> u32 {
        KIND_TABLES.known_object_kinds[place].get()
    }
    fn kind_groups(&self, place: usize) -> [u32; 2] {
        // Groups 6 and 7 of the kind's light record.
        self.1.keep(
            LIGHT_RECORDS
                .records
                .get(place)
                .map(|record| [record.groups[6].get(), record.groups[7].get()])
                .ok_or(crate::errors::BadInput("a kind's record")),
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
        self.4.shown_groups.get()
    }
    fn mask_kind(&self) -> u32 {
        self.0.shared.switches.mask_kind.get()
    }
    fn group_table(&self, group: u32) -> u32 {
        let draw_switches = &self.7.draw_switches;
        // The table is typed by its first dword only.
        self.1.keep(crate::session::read_on_in(
            draw_switches,
            std::slice::from_ref(&draw_switches.group_table),
            group as usize,
            "the groups' table",
        ))
    }
    fn no_second_mask(&self) -> u32 {
        self.5.camera_switch_active.get()
    }
    fn second_mask(&self) -> u32 {
        self.4.group_mask.get()
    }
    fn lone_group(&self) -> u32 {
        self.4.group_masks[17].get()
    }
    fn lone_group_on(&self) -> u32 {
        self.7.lights.work.marks_mask.get()
    }
    fn groups_which(&self) -> u32 {
        self.4.groups_which.get()
    }
    fn see_through_on(&self) -> u32 {
        self.3.fading.get()
    }
    fn see_through_value(&self) -> u32 {
        self.3.fade[0].get()
    }
    fn set_see_through(&mut self, value: u32) {
        self.3.see_through.set(value);
    }
    fn line_value(&self, extra: u32) -> Result<u32, rc2k_core::Invalid> {
        {
            let which: u32 = self.6.drawn_car.get();
            if which >= BLOCKS || !self.0.shared.work.has_car_block(which as usize) {
                return Err(rc2k_core::Invalid("no block for a face of kind 0x1C"));
            }
            // The core asks for 1 ..= 6: the six parts' wake counts, after `state_bits`.
            self.0
                .shared
                .work
                .car_or_end(which as usize)
                .part_wake_counts
                .get((extra as usize).wrapping_sub(1))
                .map(crate::session::Dword::get)
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
    ctx.view.object.see_through.set(0x100u32);
    let bytes = face_bytes(face)?;
    let mut world = GameFace(
        race,
        crate::scene::reading::Reading::new(),
        marks,
        &ctx.view.object,
        &ctx.view.cars,
        &ctx.view.car,
        &ctx.view.vars,
        ctx.scene,
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
    ctx.view.polygon.polygon_vertex_count.set(count);
    ctx.view.polygon.face_sort_key.set(0u32);
    face_sort_key(&ctx.view.polygon, &race.course, face, count)?;
    // The original's room was 0x1000 entries but its list 0x400 long, the rest written over
    // the next tables (fixed: `wiki/Fixed-bugs.md`).
    let key = listed_key(bytes, ctx.view.polygon.face_sort_key.get());
    list.push([
        ctx.view.face.face_number.get(),
        key,
        ctx.view.polygon.entry_third.get(),
        ctx.view.object.see_through.get(),
    ]);
    Ok(())
}
