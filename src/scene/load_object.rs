//! `scene::load_object` (0x425D87): loads the shape of one object slot from its
//! kind's file.
//!
//! In:  nothing (`dword_634FEC` the slot, `dword_634FE8` its kind)
//! Out: nothing changed for kind 0; else eax = the kind, ebx = the slot, everything
//!      else preserved
//!
//! For a kind that is not 0: the slot's kind is noted (`g_objectKinds`); the file's
//! names are made (`scene::set_object_file_names`) and the `.ro3` one read into
//! `g_objectFile` (`files::read_whole_file`, at most 0x40000 bytes); nothing more if
//! nothing was read. For a slot up to 0x40 the file's textures are listed
//! (`scene::list_object_textures`). Then:
//!
//! - a slot under 0x50: the whole file (`scene::add_object_shape` with every group);
//! - a car's slot (0x50 on, 0x14 slots a car: `dword_634FF0` the car, `dword_634FF4`
//!   the part, `dword_634FF8` the car's first slot). The kind's place among the known
//!   kinds (`scene::find_object_kind`, to `dword_5E9EC8`) picks a record of 0x134
//!   bytes whose eleven group numbers become eleven masks of one bit (`PARTS`; -1 = no
//!   such group = mask 0); `dword_5CCE2C` = every group but the first four.
//!   - part 0, the car's first slot, loads the whole car: slots +0 .. +4 take the
//!     rest and the first four groups, and eleven more slots a group each, every one
//!     of those measured (`scene::measure_object`) and its middle and half extents
//!     kept in six tables by slot (`dword_633370` ..). The slot's number is left at
//!     the car's first + 0xF.
//!   - part 0xA takes the rest again, part 0xB the whole file; the other parts
//!     nothing (part 0 loaded them).
//!
//! At the end three fatal errors: the faces' memory used up (0x80600011), the points'
//! (0x80600010), 0xC000 faces or more (0x80600012). One caller,
//! `scene::load_front_end_objects`.

use crate::errors::{FatalError, Refused};
use crate::files::read_whole_file::read_whole_file;
use crate::files::state::BYTES_READ;
use crate::pointers::GamePtr;
use crate::race::course_lists_store::COURSE_LISTS;
use crate::race::course_store::COURSE;
use crate::race::kind_store::KIND_TABLES;
use crate::race::light_records_store::LIGHT_RECORDS;
use crate::race::matrices_store::MATRICES;
use crate::scene::add_object_shape::add_object_shape;
use crate::scene::draw_switches_store::DRAW_SWITCHES;
use crate::scene::file_numbers_store::FILE_NUMBERS;
use crate::scene::list_object_textures::list_object_textures;
use crate::scene::measure_object::measure_object;
use crate::scene::object_slots_store::OBJECT_SLOTS;
use crate::scene::object_store::OBJECT_WORK;
use crate::scene::object_tables_store::OBJECT_TABLES;
use crate::scene::objects::{FACES, FILE_ROOM, SLOTS};
use crate::scene::set_object_file_names::set_object_file_names;

// ---- game data ----

const KNOWN_KINDS: u32 = 0x22;
/// `dword_5CCE1C` .. `dword_5CCE4C`: the masks (`COURSE.group_masks`).
const REST: usize = 0x10; // dword_5CCE2C
/// `dword_634FF0` the car, `dword_634FF4` the part, `dword_634FF8` the first slot.
/// Six tables of a dword a slot: the middles' x, y, z and the half extents' x, y, z
/// (`OBJECT_SLOTS.part_extents`).
/// A part's mask: where it is kept (from `dword_5CCE1C`) and where its group's number
/// is in the kind's record (from `dword_5C90C4`).
struct Part {
    mask: usize,
    group: usize,
}
const PARTS: [Part; 11] = [
    Part {
        mask: 0x00,
        group: 0x00,
    },
    Part {
        mask: 0x04,
        group: 0x04,
    },
    Part {
        mask: 0x08,
        group: 0x08,
    },
    Part {
        mask: 0x0C,
        group: 0x0C,
    },
    Part {
        mask: 0x18,
        group: 0x14,
    },
    Part {
        mask: 0x24,
        group: 0x34,
    },
    Part {
        mask: 0x28,
        group: 0x38,
    },
    Part {
        mask: 0x1C,
        group: 0x2C,
    },
    Part {
        mask: 0x20,
        group: 0x30,
    },
    Part {
        mask: 0x2C,
        group: 0x28,
    },
    Part {
        mask: 0x30,
        group: 0x3C,
    },
];
/// The mask at 0x24 gets more bits for three of the known kinds.
const WIDER: usize = 0x24;
/// Part 0's first five slots: the rest, then the first four groups.
const FIRST_FIVE: [usize; 5] = [REST, 0x00, 0x04, 0x08, 0x0C];
/// Part 0's measured slots: the slot from the car's first, and its mask.
const MEASURED: [(u32, usize); 11] = [
    (0x05, 0x00),
    (0x06, 0x04),
    (0x07, 0x08),
    (0x08, 0x0C),
    (0x09, 0x18),
    (0x10, 0x1C),
    (0x11, 0x20),
    (0x0D, 0x24),
    (0x0E, 0x28),
    (0x0C, 0x2C),
    (0x0F, 0x30),
];

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

// ---- Rust version ----

/// `scene::find_object_kind` (0x41F4BD): the place of a kind, or of the kind before
/// it, among the known ones; 0x22 if it is none of them.
fn find_object_kind(kind: u32) -> u32 {
    (0..KNOWN_KINDS)
        .find(|&n| {
            let known: u32 = KIND_TABLES.known_object_kinds[n as usize].get();
            kind == known || kind == known.wrapping_add(1)
        })
        .unwrap_or(KNOWN_KINDS)
}

fn mask(at: usize) -> u32 {
    COURSE.group_masks[at / 4].get()
}

/// The Rust version. The registers the original leaves (eax the kind, ebx the slot) are read
/// by no caller (liveness; its one caller is `load_front_end_objects`).
pub fn load_object() -> Result<(), Refused> {
    let kind: u32 = OBJECT_WORK.loading_kind.bits();
    if kind == 0 {
        return Ok(());
    }
    let slot: u32 = OBJECT_WORK.loading_slot.bits();
    if slot >= SLOTS {
        return Err(crate::errors::BadInput("an object's slot past the tables").into());
    }
    OBJECT_TABLES.object_kinds[slot as usize].set(kind);
    set_object_file_names(kind)?;
    let name = FILE_NUMBERS.object_ro3_number.bits();
    // The file read is the `.ro3` one, whatever the name's last character was.
    GamePtr::new(name.wrapping_add(7)).write(b'3', "an object's file name")?;
    BYTES_READ.store(0, std::sync::atomic::Ordering::Relaxed);
    read_whole_file(
        crate::pointers::host(FILE_NUMBERS.object_ro3_path.bits()) as *const u8,
        crate::pointers::host_at(crate::scene::objects::file_addr()),
        FILE_ROOM as u32,
    )?;
    if BYTES_READ.load(std::sync::atomic::Ordering::Relaxed) == 0 {
        return Ok(());
    }
    GamePtr::new(name.wrapping_add(7)).write(b'3', "an object's file name")?;
    if slot <= 0x40 {
        list_object_textures()?;
    }
    if slot < 0x50 {
        shape(u32::MAX)?;
        return check();
    }
    DRAW_SWITCHES.loading_group_mask.set_bits(u32::MAX);
    let known = find_object_kind(kind);
    MATRICES.known_object.set_bits(known);
    // The kind's record from `dword_5C90C4` (its `groups`); the kind is not bounded.
    let record = LIGHT_RECORDS.record_addr(known) as usize + 0x58;
    for part in &PARTS {
        let group: u32 = GamePtr::new((record + part.group) as u32).read("a kind's record")?;
        let mut bits = 0;
        if group != u32::MAX {
            bits = 1u32 << (group & 31);
            if part.mask == WIDER {
                bits |= match known {
                    4 | 7 => 0x0600_0000,
                    0xD => 0x0300_0000,
                    _ => 0,
                };
            }
        }
        COURSE.group_masks[part.mask / 4].set_bits(bits);
    }
    let rest = FIRST_FIVE[1..]
        .iter()
        .fold(u32::MAX, |rest, &at| rest.wrapping_sub(mask(at)));
    COURSE.group_masks[REST / 4].set_bits(rest);
    let (car, part) = ((slot - 0x50) / 0x14, (slot - 0x50) % 0x14);
    let first = car * 0x14 + 0x50;
    crate::session::write_all(&OBJECT_WORK.loading_car, [car, part, first]);
    match part {
        0 => {
            for (n, &at) in FIRST_FIVE.iter().enumerate() {
                OBJECT_WORK.loading_slot.set_bits(first + n as u32);
                shape(mask(at))?;
            }
            OBJECT_WORK.loading_slot.set_bits(first + 5);
            for &(offset, at) in &MEASURED {
                let slot = first + offset;
                OBJECT_WORK.loading_slot.set_bits(slot);
                shape(mask(at))?;
                measure_object()?;
                let measures: [u32; 6] = crate::session::read_first(&OBJECT_SLOTS.measures);
                for (table, &measure) in measures.iter().enumerate() {
                    OBJECT_SLOTS.part_extents[table][slot as usize].set(measure);
                }
            }
        }
        0xA => shape(rest)?,
        0xB => shape(u32::MAX)?,
        _ => {}
    }
    check()
}

/// One `scene::add_object_shape` with a mask.
fn shape(mask: u32) -> Result<(), Refused> {
    DRAW_SWITCHES.loading_group_mask.set_bits(mask);
    Ok({ add_object_shape(crate::scene::objects::file_addr()) }?)
}

/// The three fatal errors at the end, in the original's order.
fn check() -> Result<(), Refused> {
    if OBJECT_WORK.face_cursor.bits() > COURSE_LISTS.face_memory_room.bits() {
        return Err(NO_FACE_MEMORY.into());
    }
    if OBJECT_TABLES.point_count.get().wrapping_mul(12) > COURSE_LISTS.object_points_room.bits() {
        return Err(NO_POINT_MEMORY.into());
    }
    if OBJECT_TABLES.face_count.get() >= FACES {
        return Err(NO_FACE_ROOM.into());
    }
    Ok(())
}
