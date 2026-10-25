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
use crate::framework::{game_read, game_write, rva};
use crate::physics::car_store::CAR;
use crate::race::course_store::COURSE;
use crate::race::numbers_store::NUMBERS;
use crate::race::vars_store::RACE_VARS;
use crate::race::work_store::RACE_WORK;
use crate::scene::draw_object_faces::{LIST_RVA, LISTED_RVA};
use crate::scene::draw_store::DRAW;
use crate::scene::face_faces_away::face_faces_away;
use crate::scene::face_sort_key::face_sort_key;
use crate::scene::polygon::dword;
use rc2k_core::scene::face_tests::{Face, FaceWorld, dropped, listed_key};

// ---- game data ----

pub const SEE_THROUGH_RVA: usize = rva(0x639970); // g_objectSeeThrough
const OBJECT_RVA: usize = rva(0x6214FC); // g_objectNumber
const OBJECT_KINDS_RVA: usize = rva(0x5CD27C); // dword_5CD27C
const KNOWN_KINDS_RVA: usize = rva(0x5C5790); // dword_5C5790
const KIND_GROUPS_RVA: usize = rva(0x5C90DC); // dword_5C90DC, dword_5C90E0
const KIND_LEN: usize = 0x134;
pub const HIDDEN_GROUPS_RVA: usize = rva(0x5CCE50); // dword_5CCE50, dword_5CCE54
const LONE_GROUP_ON_RVA: usize = rva(0x62DAD0); // dword_62DAD0
const GROUPS_WHICH_RVA: usize = rva(0x6D18D4); // dword_6D18D4
const GROUP_MASK_RVA: usize = rva(0x62150C); // dword_62150C
const GROUP_MASK_2_RVA: usize = rva(0x606280); // dword_606280
const MASK_KIND_RVA: usize = rva(0x621870); // dword_621870
const GROUP_TABLE_RVA: usize = rva(0x621934); // dword_621934
const SEE_THROUGH_ON_RVA: usize = rva(0x63996C); // dword_63996C
const SEE_THROUGH_VALUE_RVA: usize = rva(0x639974); // dword_639974
const BLOCKS: u32 = 8;
const BLOCK_RVA: usize = rva(0x60627C); // dword_60627C
const KIND_BYTES_RVA: usize = rva(0x5CD688); // dword_5CD688
const LIST_ROOM: u32 = 0x1000;

// ---- Rust version ----

unsafe fn byte(at: usize) -> u32 {
    unsafe { (at as *const u8).read() as u32 }
}

/// The game's variables the tests read and write.
struct GameFace;

impl FaceWorld for GameFace {
    fn in_race(&self) -> bool {
        NUMBERS.in_race.bits() != 0
    }
    fn object(&self) -> u32 {
        unsafe { game_read(OBJECT_RVA) }
    }
    fn object_kind(&self, object: u32) -> u32 {
        unsafe { game_read(OBJECT_KINDS_RVA + object as usize * 4) }
    }
    fn known_kind(&self, place: usize) -> u32 {
        unsafe { game_read(KNOWN_KINDS_RVA + place * 4) }
    }
    fn kind_groups(&self, place: usize) -> [u32; 2] {
        unsafe { game_read(KIND_GROUPS_RVA + place * KIND_LEN) }
    }
    fn hidden_groups(&self) -> [u32; 2] {
        unsafe { game_read(HIDDEN_GROUPS_RVA) }
    }
    fn set_hidden_groups(&mut self, groups: [u32; 2]) {
        unsafe { game_write(HIDDEN_GROUPS_RVA, groups) }
    }
    fn other_groups(&self) -> [u32; 2] {
        [15, 16].map(|k| COURSE.group_masks[k].get())
    }
    fn group_mask(&self) -> u32 {
        unsafe { game_read(GROUP_MASK_RVA) }
    }
    fn mask_kind(&self) -> u32 {
        unsafe { game_read(MASK_KIND_RVA) }
    }
    fn group_table(&self, group: u32) -> u32 {
        unsafe { game_read(GROUP_TABLE_RVA + group as usize * 4) }
    }
    fn no_second_mask(&self) -> u32 {
        CAR.camera_switch_active.bits()
    }
    fn second_mask(&self) -> u32 {
        unsafe { game_read(GROUP_MASK_2_RVA) }
    }
    fn lone_group(&self) -> u32 {
        COURSE.group_masks[17].bits()
    }
    fn lone_group_on(&self) -> u32 {
        unsafe { game_read(LONE_GROUP_ON_RVA) }
    }
    fn groups_which(&self) -> u32 {
        unsafe { game_read(GROUPS_WHICH_RVA) }
    }
    fn see_through_on(&self) -> u32 {
        unsafe { game_read(SEE_THROUGH_ON_RVA) }
    }
    fn see_through_value(&self) -> u32 {
        unsafe { game_read(SEE_THROUGH_VALUE_RVA) }
    }
    fn set_see_through(&mut self, value: u32) {
        unsafe { game_write(SEE_THROUGH_RVA, value) }
    }
    fn line_value(&self, extra: u32) -> Result<u32, rc2k_core::Invalid> {
        unsafe {
            let which: u32 = game_read(BLOCK_RVA);
            let block: u32 = if which < BLOCKS {
                RACE_WORK.car_block(which as usize)
            } else {
                0
            };
            if block == 0 {
                return Err(rc2k_core::Invalid("no block for a face of kind 0x1C"));
            }
            Ok(dword(block as usize + extra as usize * 4 + 0x6FC))
        }
    }
    fn kind_byte(&self, kind: u32) -> u32 {
        unsafe { game_read(KIND_BYTES_RVA + kind as usize * 4) }
    }
}

/// The face's bytes the tests read.
unsafe fn face_bytes(face: usize) -> Face {
    unsafe {
        Face {
            kind: byte(face + 6) as u8,
            group: byte(face + 7) as u8,
            flags: byte(face + 8) as u8,
            extra: byte(face + 0xB) as u8,
        }
    }
}

// Converted: `rc2k_core::scene::face_tests::{dropped, listed_key}`
pub unsafe fn list_object_face(face: usize) -> Result<(), Refused> {
    unsafe {
        if face == 0 {
            return Err(BadInput("no face").into());
        }
        let listed: u32 = game_read(LISTED_RVA);
        if listed >= LIST_ROOM {
            return Err(BadInput("no room in the list of faces").into());
        }
        game_write(SEE_THROUGH_RVA, 0x100u32);
        let bytes = face_bytes(face);
        match dropped(&mut GameFace, bytes) {
            Ok(true) => return Ok(()),
            Ok(false) => {}
            Err(why) => return Err(why.into()),
        }
        if face_faces_away(face)? {
            return Ok(());
        }
        let count = dword(face);
        if count as i32 <= 0 {
            return Ok(());
        }
        DRAW.polygon_vertex_count.set_bits(count);
        DRAW.face_sort_key.set_bits(0u32);
        face_sort_key(face, count)?;
        let entry = LIST_RVA + listed as usize * 0x10;
        let key = listed_key(bytes, DRAW.face_sort_key.bits());
        game_write(
            entry,
            [
                RACE_VARS.face_number.bits(),
                key,
                DRAW.entry_third.bits(),
                game_read(SEE_THROUGH_RVA),
            ],
        );
        game_write(LISTED_RVA, listed + 1);
        Ok(())
    }
}
