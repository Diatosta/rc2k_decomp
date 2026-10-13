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
//! A face that is left and has a vertex gets its key (`scene::face_sort_key`) and the
//! next entry of the list: its number, the key plus bit 30 unless bit 0 of its byte
//! +8 is set, `dword_624130`, the see-through value.
//! One caller, `scene::list_object_faces`.
//! Refuses an object's number or a list's length past their tables, no block for
//! kind 0x1C, and what its parts refuse.
//! Shadow mode: the Rust version, its writes put back, the original, compared.

use crate::d3d::draw_fan_unclipped::COUNT_RVA;
use crate::errors::{BadInput, Refused};
use crate::framework::{
    RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::math::normalise_vector::VECTOR_RVA;
use crate::scene::draw_object_faces::{LIST_RVA, LISTED_RVA};
use crate::scene::face_faces_away::face_faces_away;
use crate::scene::face_sort_key::{KEY_RVA, face_sort_key};
use crate::scene::polygon::dword;
use crate::scene::project_point::RECORDS_RVA;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::list_object_face", rva(0x4504CB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub const SEE_THROUGH_RVA: usize = rva(0x639970); // g_objectSeeThrough
const IN_RACE_RVA: usize = rva(0x5C5CEC); // dword_5C5CEC
const OBJECT_RVA: usize = rva(0x6214FC); // g_objectNumber
const OBJECTS: u32 = 0x100;
const OBJECT_KINDS_RVA: usize = rva(0x5CD27C); // dword_5CD27C
const KNOWN_KINDS_RVA: usize = rva(0x5C5790); // dword_5C5790
const KNOWN_KINDS: usize = 0x22;
const KIND_GROUPS_RVA: usize = rva(0x5C90DC); // dword_5C90DC, dword_5C90E0
const KIND_LEN: usize = 0x134;
pub const HIDDEN_GROUPS_RVA: usize = rva(0x5CCE50); // dword_5CCE50, dword_5CCE54
const OTHER_GROUPS_RVA: usize = rva(0x5CCE58); // dword_5CCE58, dword_5CCE5C
const LONE_GROUP_RVA: usize = rva(0x5CCE60); // dword_5CCE60
const LONE_GROUP_ON_RVA: usize = rva(0x62DAD0); // dword_62DAD0
const GROUPS_WHICH_RVA: usize = rva(0x6D18D4); // dword_6D18D4
const GROUP_MASK_RVA: usize = rva(0x62150C); // dword_62150C
const GROUP_MASK_2_RVA: usize = rva(0x606280); // dword_606280
const MASK_KIND_RVA: usize = rva(0x621870); // dword_621870
const GROUP_TABLE_RVA: usize = rva(0x621934); // dword_621934
const NO_MASK_2_RVA: usize = rva(0x71A66C); // dword_71A66C
const SEE_THROUGH_ON_RVA: usize = rva(0x63996C); // dword_63996C
const SEE_THROUGH_VALUE_RVA: usize = rva(0x639974); // dword_639974
const BLOCKS_RVA: usize = rva(0x71B3B8); // off_71B3B8
const BLOCKS: u32 = 8;
const BLOCK_RVA: usize = rva(0x60627C); // dword_60627C
const KIND_BYTES_RVA: usize = rva(0x5CD688); // dword_5CD688
const FACE_NUMBER_RVA: usize = rva(0x623620); // dword_623620
const ENTRY_THIRD_RVA: usize = rva(0x624130); // dword_624130
const LIST_ROOM: u32 = 0x1000;

// ---- Rust version ----

unsafe fn byte(at: usize) -> u32 {
    unsafe { (at as *const u8).read() as u32 }
}

/// The four tests of the function's own. Ok(true): the face is dropped.
unsafe fn dropped(face: usize) -> Result<bool, BadInput> {
    unsafe {
        let group = byte(face + 7) & 0x1F;
        let object: u32 = game_read(OBJECT_RVA);
        let in_race = game_read::<u32>(IN_RACE_RVA) != 0;
        if !in_race {
            if object >= OBJECTS {
                return Err(BadInput("an object's number past the tables"));
            }
            let kind: u32 = game_read(OBJECT_KINDS_RVA + object as usize * 4);
            let known: [u32; KNOWN_KINDS] = game_read(KNOWN_KINDS_RVA);
            let place = known
                .iter()
                .position(|&entry| kind == entry || kind == entry.wrapping_add(1));
            if let Some(place) = place {
                let groups: [u32; 2] = game_read(KIND_GROUPS_RVA + place * KIND_LEN);
                game_write(HIDDEN_GROUPS_RVA, groups);
                if groups.contains(&group) {
                    return Ok(true);
                }
            }
        }
        if object >= 0x50 && object.is_multiple_of(0x14) {
            let bit = |mask: usize| game_read::<u32>(mask) >> group & 1 != 0;
            if !bit(GROUP_MASK_RVA) {
                return Ok(true);
            }
            if game_read::<u32>(MASK_KIND_RVA) == 5 {
                if game_read::<u32>(GROUP_TABLE_RVA + group as usize * 4) == 0 {
                    return Ok(true);
                }
            } else if game_read::<u32>(NO_MASK_2_RVA) == 0 && !bit(GROUP_MASK_2_RVA) {
                return Ok(true);
            }
            if group == game_read::<u32>(LONE_GROUP_RVA) && game_read::<u32>(LONE_GROUP_ON_RVA) == 0
            {
                return Ok(true);
            }
            let which: u32 = game_read(GROUPS_WHICH_RVA);
            if game_read::<[u32; 2]>(HIDDEN_GROUPS_RVA).contains(&group) {
                if which == 1 {
                    return Ok(true);
                }
            } else if game_read::<[u32; 2]>(OTHER_GROUPS_RVA).contains(&group) && which == 0 {
                return Ok(true);
            }
        }
        if (byte(face + 8) >> 2) & 3 != 0 && game_read::<u32>(SEE_THROUGH_ON_RVA) != 0 {
            let value: u32 = game_read(SEE_THROUGH_VALUE_RVA);
            if value == 0 {
                return Ok(true);
            }
            game_write(SEE_THROUGH_RVA, value);
        }
        let (kind, extra) = (byte(face + 6), byte(face + 0xB));
        if kind == 0x1C {
            if !in_race || extra == 0 || extra > 6 {
                return Ok(true);
            }
            let which: u32 = game_read(BLOCK_RVA);
            let block: u32 = if which < BLOCKS {
                game_read(BLOCKS_RVA + which as usize * 4)
            } else {
                0
            };
            if block == 0 {
                return Err(BadInput("no block for a face of kind 0x1C"));
            }
            let value = dword(block as usize + extra as usize * 4 + 0x6FC);
            if value == 0x8000_0000 || value < 0xA {
                return Ok(true);
            }
        } else if kind != 0 && extra != game_read::<u32>(KIND_BYTES_RVA + kind as usize * 4) {
            return Ok(true);
        }
        Ok(false)
    }
}

pub unsafe fn list_object_face(face: usize) -> Result<(), Refused> {
    unsafe {
        if face == 0 {
            return Err(BadInput("no face").into());
        }
        let listed: u32 = game_read(LISTED_RVA);
        if listed >= LIST_ROOM {
            return Err(BadInput("no room in the list of faces").into());
        }
        // The tests write three variables; a refusal puts them back.
        let regions = [
            at(SEE_THROUGH_RVA),
            (game_addr(HIDDEN_GROUPS_RVA) as u32, 8),
        ];
        let before = capture(&regions);
        game_write(SEE_THROUGH_RVA, 0x100u32);
        match dropped(face) {
            Ok(true) => return Ok(()),
            Ok(false) => {}
            Err(why) => {
                restore(&regions, &before);
                return Err(why.into());
            }
        }
        if face_faces_away(face)? {
            return Ok(());
        }
        let count = dword(face);
        if count as i32 <= 0 {
            return Ok(());
        }
        game_write(COUNT_RVA, count);
        game_write(KEY_RVA, 0u32);
        face_sort_key(face, count)?;
        let entry = LIST_RVA + listed as usize * 0x10;
        let key = (((byte(face + 8) & 1) ^ 1) << 30).wrapping_add(game_read(KEY_RVA));
        game_write(
            entry,
            [
                game_read::<u32>(FACE_NUMBER_RVA),
                key,
                game_read(ENTRY_THIRD_RVA),
                game_read(SEE_THROUGH_RVA),
            ],
        );
        game_write(LISTED_RVA, listed + 1);
        Ok(())
    }
}

// ---- hook ----

/// Everything a call may write, as the list stands before it.
pub fn regions() -> Vec<Region> {
    let listed = unsafe { game_read::<u32>(LISTED_RVA) }.min(LIST_ROOM - 1) as usize;
    vec![
        at(SEE_THROUGH_RVA),
        (game_addr(HIDDEN_GROUPS_RVA) as u32, 8),
        at(COUNT_RVA),
        at(KEY_RVA),
        at(LISTED_RVA),
        (game_addr(LIST_RVA + listed * 0x10) as u32, 0x10),
        // What `scene::face_faces_away` works in.
        (game_addr(rva(0x6396A0)) as u32, 60),
        (game_addr(VECTOR_RVA) as u32, 12),
    ]
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let face = r.ebp as usize;
    if !shadow() {
        if let Err(e) = unsafe { list_object_face(face) } {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        }
        return;
    }
    let regions = regions();
    let before = capture(&regions);
    let mine = unsafe { list_object_face(face) }.map(|()| capture(&regions));
    restore(&regions, &before);
    unsafe { call_original(r) };
    let Ok(mine) = mine else {
        return;
    };
    let theirs = capture(&regions);
    if mine != theirs {
        HOOK.mismatch(|| {
            format!(
                "object {:#x}, face {:x?}: original {theirs:x?}; rust {mine:x?}",
                unsafe { game_read::<u32>(OBJECT_RVA) },
                unsafe { (face as *const [u32; 4]).read_unaligned() }
            )
        });
    }
}

// ---- self-test: faces and switches drawn by lot ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // The objects the test uses, and the kinds' places it looks at.
    const TEST_OBJECTS: [u32; 4] = [0x3C, 0x51, 0x64, 0xF0];
    let known: [u32; KNOWN_KINDS] = unsafe { game_read(KNOWN_KINDS_RVA) };
    let mut inputs: Vec<Region> = [
        IN_RACE_RVA,
        OBJECT_RVA,
        OTHER_GROUPS_RVA,
        OTHER_GROUPS_RVA + 4,
        LONE_GROUP_RVA,
        LONE_GROUP_ON_RVA,
        GROUPS_WHICH_RVA,
        GROUP_MASK_RVA,
        GROUP_MASK_2_RVA,
        MASK_KIND_RVA,
        NO_MASK_2_RVA,
        SEE_THROUGH_ON_RVA,
        SEE_THROUGH_VALUE_RVA,
        BLOCK_RVA,
        BLOCKS_RVA + 7 * 4,
        FACE_NUMBER_RVA,
        ENTRY_THIRD_RVA,
        RECORDS_RVA,
        // The key's factor for three vertices.
        rva(0x5C6950) + 3 * 4,
    ]
    .map(at)
    .to_vec();
    inputs.extend(TEST_OBJECTS.map(|object| at(OBJECT_KINDS_RVA + object as usize * 4)));
    inputs.push((game_addr(GROUP_TABLE_RVA) as u32, 0x20 * 4));
    inputs.push((game_addr(KIND_BYTES_RVA) as u32, 8 * 4));
    inputs.push((game_addr(KIND_BYTES_RVA + 0x1C * 4) as u32, 4));
    inputs.push((game_addr(KIND_GROUPS_RVA + 3 * KIND_LEN) as u32, 8));
    // What the function writes: the list's first three entries can be the place.
    inputs.extend([SEE_THROUGH_RVA, COUNT_RVA, KEY_RVA, LISTED_RVA].map(at));
    inputs.push((game_addr(HIDDEN_GROUPS_RVA) as u32, 8));
    inputs.push((game_addr(LIST_RVA) as u32, 3 * 0x10));
    inputs.push((game_addr(rva(0x6396A0)) as u32, 60));
    inputs.push((game_addr(VECTOR_RVA) as u32, 12));
    let saved_inputs = capture(&inputs);

    let f = |value: f32| value.to_bits();
    let mut records = vec![[0u32; 8]; 0x2000];
    for (n, place) in [
        [0.0f32, 0.0, 9000.0],
        [100.0, 0.0, 9000.0],
        [0.0, 100.0, 9500.0],
    ]
    .into_iter()
    .enumerate()
    {
        records[0x1000 + 0x50 + n][4..7].copy_from_slice(&place.map(f));
    }
    let mut block = vec![0u32; 0x720 / 4];
    block[0x6FC / 4 + 1..0x6FC / 4 + 7].copy_from_slice(&[0, 9, 10, 0x8000_0000, 0x7FFF_FFFF, 500]);

    let mut seed = 0x0BAD_5EEDu32;
    let mut lot = move |below: u32| {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (seed >> 10) % below
    };
    for case in 0..400u32 {
        let object = TEST_OBJECTS[lot(4) as usize];
        let group = lot(6);
        let kind = [0u32, 0, 3, 0x1C][lot(4) as usize];
        let extra = lot(8);
        let flags = [0u32, 1, 4, 5, 8, 0xC][lot(6) as usize];
        // Seen from the front, from behind, or no vertices at all.
        let shape = lot(4);
        let mut face = [0u32; 4 + 3 * 4];
        face[0] = if shape == 3 { 0 } else { 3 };
        face[1] = kind << 16 | group << 24 | 0xE000_0000;
        face[2] = flags | extra << 24;
        let order = if shape == 1 { [0, 2, 1] } else { [0, 1, 2] };
        for (n, point) in order.into_iter().enumerate() {
            face[4 + n * 4] = 0x4050 + point;
        }
        let object_kind = [0x7777_7777, known[3], known[3].wrapping_add(1)][lot(3) as usize];
        let set: Vec<(usize, u32)> = vec![
            (IN_RACE_RVA, lot(2)),
            (OBJECT_RVA, object),
            (OBJECT_KINDS_RVA + object as usize * 4, object_kind),
            (KIND_GROUPS_RVA + 3 * KIND_LEN, lot(6)),
            (KIND_GROUPS_RVA + 3 * KIND_LEN + 4, lot(6)),
            (OTHER_GROUPS_RVA, lot(6)),
            (OTHER_GROUPS_RVA + 4, lot(6)),
            (LONE_GROUP_RVA, lot(6)),
            (LONE_GROUP_ON_RVA, lot(2)),
            (GROUPS_WHICH_RVA, lot(3)),
            (GROUP_MASK_RVA, [u32::MAX, u32::MAX, 0x2A][lot(3) as usize]),
            (GROUP_MASK_2_RVA, [u32::MAX, 0x15, 0][lot(3) as usize]),
            (MASK_KIND_RVA, [0u32, 5][lot(2) as usize]),
            (NO_MASK_2_RVA, lot(2)),
            (GROUP_TABLE_RVA + group as usize * 4, lot(2)),
            (SEE_THROUGH_ON_RVA, lot(2)),
            (SEE_THROUGH_VALUE_RVA, [0u32, 0x80][lot(2) as usize]),
            (BLOCK_RVA, 7),
            (BLOCKS_RVA + 7 * 4, block.as_ptr() as u32),
            (KIND_BYTES_RVA + kind as usize * 4, lot(8)),
            (FACE_NUMBER_RVA, 0x100 + case),
            (ENTRY_THIRD_RVA, 0x3333_0000 + case),
            (RECORDS_RVA, records.as_ptr() as u32),
            (rva(0x5C6950) + 3 * 4, f(0.33333334)),
            (LISTED_RVA, lot(3)),
            // Left by an earlier face, for the faces the first test does not reach.
            (HIDDEN_GROUPS_RVA, lot(6)),
            (HIDDEN_GROUPS_RVA + 4, lot(6)),
        ];
        let face_at = face.as_ptr() as usize;
        let run = |rust: bool| unsafe {
            for &(at, value) in &set {
                game_write(at, value);
            }
            let regions = regions();
            let blotted: Vec<Region> = regions
                .iter()
                .copied()
                .filter(|&(at, _)| {
                    at != game_addr(LISTED_RVA) as u32 && at != game_addr(HIDDEN_GROUPS_RVA) as u32
                })
                .collect();
            restore(&blotted, &vec![0xAA; blotted.iter().map(|r| r.1).sum()]);
            let done = if rust {
                list_object_face(face_at).is_ok()
            } else {
                let mut ro = Regs {
                    ebp: face_at as u32,
                    ..Regs::default()
                };
                call_original(&mut ro);
                true
            };
            (done, capture(&regions))
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine, || {
            format!(
                "case {case}: face {face:x?}, set {set:x?}: original {theirs:x?}; rust {mine:x?}"
            )
        });
    }
    restore(&inputs, &saved_inputs);
    t.finish()
}
