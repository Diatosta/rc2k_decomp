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
//! (0x80600010), 0xC000 faces or more (0x80600012).
//! One caller, `scene::load_front_end_objects`.
//! Shadow mode runs the original alone (its parts are compared in their own hooks).
//! Self-test: a redirected copy whose calls note themselves, 200 cases by lot.

use crate::errors::{FatalError, Refused};
use crate::files::read_whole_file::{BYTES_READ_RVA, read_whole_file};
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::install::stand_ins::{WENT_FATAL, act, fatal_landing, set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::scene::add_object_shape::{MASK_RVA, PART_RVA, add_object_shape};
use crate::scene::list_object_textures::list_object_textures;
use crate::scene::measure_object::{MEASURES_RVA, measure_object};
use crate::scene::objects::{
    FACE_COUNT_RVA, FACE_CURSOR_RVA, FACE_ROOM_RVA, FACES, FILE_ROOM, FILE_RVA, KIND_RVA,
    OBJECT_KINDS_RVA, POINT_COUNT_RVA, POINTS_ROOM_RVA, SLOT_RVA, SLOTS,
};
use crate::scene::set_object_file_names::{NAMES_RVA, set_object_file_names};
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::load_object", rva(0x425D87), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const KNOWN_KINDS_RVA: usize = rva(0x5C5790); // g_knownObjectKinds, 0x22 kinds
const KNOWN_KINDS: u32 = 0x22;
const KNOWN_RVA: usize = rva(0x5E9EC8); // dword_5E9EC8
/// The kinds' records of 0x134 bytes, as far as this function reads them: from
/// `dword_5C90C4`.
const GROUPS_RVA: usize = rva(0x5C90C4);
const GROUPS_LEN: usize = 0x134;
/// `dword_5CCE1C` .. `dword_5CCE4C`: the masks.
const MASKS_RVA: usize = rva(0x5CCE1C);
const MASKS_LEN: usize = 0x34;
const REST: usize = 0x10; // dword_5CCE2C
/// `dword_634FF0` the car, `dword_634FF4` the part, `dword_634FF8` the first slot.
const CAR_RVA: usize = rva(0x634FF0);
/// Six tables of a dword a slot: the middles' x, y, z and the half extents' x, y, z.
const MIDDLES_RVA: usize = rva(0x633370);
const MIDDLES_LEN: usize = 6 * SLOTS as usize * 4;

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

/// What the function calls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Call {
    /// `scene::set_object_file_names`, with the kind.
    Names(u32),
    /// `files::read_whole_file` of the `.ro3` file.
    Read,
    Textures,
    Shape,
    Measure,
}

// ---- Rust version ----

/// `scene::find_object_kind` (0x41F4BD): the place of a kind, or of the kind before
/// it, among the known ones; 0x22 if it is none of them.
unsafe fn find_object_kind(kind: u32) -> u32 {
    (0..KNOWN_KINDS)
        .find(|&n| {
            let known: u32 = unsafe { game_read(KNOWN_KINDS_RVA + n as usize * 4) };
            kind == known || kind == known.wrapping_add(1)
        })
        .unwrap_or(KNOWN_KINDS)
}

unsafe fn mask(at: usize) -> u32 {
    unsafe { game_read(MASKS_RVA + at) }
}

pub unsafe fn load_object(
    r: &mut Regs,
    call: &mut dyn FnMut(Call) -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        let kind: u32 = game_read(KIND_RVA);
        if kind == 0 {
            return Ok(());
        }
        let slot: u32 = game_read(SLOT_RVA);
        if slot >= SLOTS {
            return Err(crate::errors::BadInput("an object's slot past the tables").into());
        }
        game_write(OBJECT_KINDS_RVA + slot as usize * 4, kind);
        r.eax = kind;
        r.ebx = slot;
        call(Call::Names(kind))?;
        let name = |n: usize| game_read::<u32>(NAMES_RVA + n * 4);
        // The file read is the `.ro3` one, whatever the name's last character was.
        ((name(0) + 7) as *mut u8).write(b'3');
        game_write(BYTES_READ_RVA, 0u32);
        call(Call::Read)?;
        if game_read::<u32>(BYTES_READ_RVA) == 0 {
            return Ok(());
        }
        ((name(0) + 7) as *mut u8).write(b'3');
        if slot <= 0x40 {
            call(Call::Textures)?;
        }
        if slot < 0x50 {
            shape(call, u32::MAX)?;
            return check();
        }
        game_write(MASK_RVA, u32::MAX);
        let known = find_object_kind(kind);
        game_write(KNOWN_RVA, known);
        let record = GROUPS_RVA + known as usize * GROUPS_LEN;
        for part in &PARTS {
            let group: u32 = game_read(record + part.group);
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
            game_write(MASKS_RVA + part.mask, bits);
        }
        let rest = FIRST_FIVE[1..]
            .iter()
            .fold(u32::MAX, |rest, &at| rest.wrapping_sub(mask(at)));
        game_write(MASKS_RVA + REST, rest);
        let (car, part) = ((slot - 0x50) / 0x14, (slot - 0x50) % 0x14);
        let first = car * 0x14 + 0x50;
        game_write(CAR_RVA, [car, part, first]);
        match part {
            0 => {
                for (n, &at) in FIRST_FIVE.iter().enumerate() {
                    game_write(SLOT_RVA, first + n as u32);
                    shape(call, mask(at))?;
                }
                game_write(SLOT_RVA, first + 5);
                for &(offset, at) in &MEASURED {
                    let slot = first + offset;
                    game_write(SLOT_RVA, slot);
                    shape(call, mask(at))?;
                    call(Call::Measure)?;
                    let measures: [u32; 6] = game_read(MEASURES_RVA);
                    for (table, &measure) in measures.iter().enumerate() {
                        game_write(
                            MIDDLES_RVA + (table * SLOTS as usize + slot as usize) * 4,
                            measure,
                        );
                    }
                }
            }
            0xA => shape(call, rest)?,
            0xB => shape(call, u32::MAX)?,
            _ => {}
        }
        check()
    }
}

/// One `scene::add_object_shape` with a mask.
unsafe fn shape(
    call: &mut dyn FnMut(Call) -> Result<(), Refused>,
    mask: u32,
) -> Result<(), Refused> {
    unsafe { game_write(MASK_RVA, mask) };
    call(Call::Shape)
}

/// The three fatal errors at the end, in the original's order.
unsafe fn check() -> Result<(), Refused> {
    unsafe {
        if game_read::<u32>(FACE_CURSOR_RVA) > game_read::<u32>(FACE_ROOM_RVA) {
            return Err(NO_FACE_MEMORY.into());
        }
        if game_read::<u32>(POINT_COUNT_RVA).wrapping_mul(12) > game_read::<u32>(POINTS_ROOM_RVA) {
            return Err(NO_POINT_MEMORY.into());
        }
        if game_read::<u32>(FACE_COUNT_RVA) >= FACES {
            return Err(NO_FACE_ROOM.into());
        }
        Ok(())
    }
}

/// With the Rust versions of what it calls.
pub unsafe fn load_object_in_game(r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        load_object(r, &mut |call| {
            match call {
                Call::Names(kind) => {
                    set_object_file_names(kind);
                }
                Call::Read => {
                    read_whole_file(
                        game_read::<u32>(NAMES_RVA + 4) as *const u8,
                        game_addr(FILE_RVA) as *mut u8,
                        FILE_ROOM as u32,
                        0,
                    )?;
                }
                Call::Textures => list_object_textures()?,
                Call::Shape => add_object_shape(game_addr(FILE_RVA))?,
                Call::Measure => {
                    measure_object()?;
                }
            }
            Ok(())
        })
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let entry = *r;
    if let Err(e) = unsafe { load_object_in_game(r) } {
        *r = entry;
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let entry = *r;
    if let Err(e) = unsafe { load_object_in_game(r) } {
        *r = entry;
        return Err(e);
    }
    Ok(())
}

// ---- self-test ----

const START: usize = 0x425D87;
const LEN: usize = 0x90E;
const ENTRY_BYTES: [u8; 7] = [0x83, 0x3D, 0xE8, 0x4F, 0x63, 0x00, 0x00];
/// Every call of the function, by its address.
const CALL_SITES: [usize; 33] = [
    0x425DAC, 0x425DDD, 0x425E03, 0x425E21, 0x425E3A, 0x42607D, 0x426098, 0x4260B3, 0x4260CE,
    0x4260E9, 0x426111, 0x426116, 0x42618C, 0x426191, 0x426207, 0x42620C, 0x426282, 0x426287,
    0x4262FD, 0x426302, 0x426378, 0x42637D, 0x4263F3, 0x4263F8, 0x42646E, 0x426473, 0x4264E9,
    0x4264EE, 0x426564, 0x426569, 0x4265DF, 0x4265E4, 0x42664F,
];
/// The operands of the three jumps to the fatal stubs.
const FATAL_JUMPS: [usize; 3] = [0x900, 0x905, 0x90A];

/// Each call: its place, the slot and the mask by then, eax.
static CALLED: Mutex<Vec<[u32; 4]>> = Mutex::new(Vec::new());
/// What the reading stand-in says was read, and where the names' stand-in points the
/// `.ro3` name's number.
static READS: Mutex<(u32, u32)> = Mutex::new((0, 0));

fn handler(place: u32, r: &mut Regs) {
    unsafe {
        CALLED
            .lock()
            .unwrap()
            .push([place, game_read(SLOT_RVA), game_read(MASK_RVA), r.eax]);
        let (read, name) = *READS.lock().unwrap();
        match place {
            0 => game_write(NAMES_RVA, name),
            1 => game_write(BYTES_READ_RVA, read),
            4 => {
                // The measures are the slot's own, so that the tables can be told apart.
                let slot: u32 = game_read(SLOT_RVA);
                game_write(
                    MEASURES_RVA,
                    std::array::from_fn::<u32, 6, _>(|n| slot * 16 + n as u32),
                );
            }
            _ => {}
        }
    }
}

fn place_of(call: Call) -> u32 {
    match call {
        Call::Names(_) => 0,
        Call::Read => 1,
        Call::Textures => 2,
        Call::Shape => 3,
        Call::Measure => 4,
    }
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let calls: Vec<usize> = CALL_SITES.iter().map(|site| site - START).collect();
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(START)),
            LEN,
            &ENTRY_BYTES,
            &calls,
            &|callee| {
                [
                    rva(0x43964A),
                    rva(0x421B54),
                    rva(0x426ACA),
                    rva(0x426695),
                    rva(0x4222B8),
                ]
                .iter()
                .position(|&known| known == callee)
                .map(|place| stand_in(place as u32))
            },
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    for operand in FATAL_JUMPS {
        unsafe { copy.retarget(operand, fatal_landing as *const () as usize) };
    }
    let regions: Vec<Region> = vec![
        at(SLOT_RVA),
        at(MASK_RVA),
        at(KNOWN_RVA),
        at(BYTES_READ_RVA),
        (game_addr(NAMES_RVA) as u32, 16),
        (game_addr(MASKS_RVA) as u32, MASKS_LEN),
        (game_addr(CAR_RVA) as u32, 12),
        (game_addr(MIDDLES_RVA) as u32, MIDDLES_LEN),
        (game_addr(OBJECT_KINDS_RVA) as u32, SLOTS as usize * 4),
        (game_addr(MEASURES_RVA) as u32, 0x30),
    ];
    let inputs: Vec<Region> = vec![
        at(KIND_RVA),
        at(FACE_CURSOR_RVA),
        at(FACE_ROOM_RVA),
        at(POINT_COUNT_RVA),
        at(POINTS_ROOM_RVA),
        at(FACE_COUNT_RVA),
        at(PART_RVA),
        (
            game_addr(GROUPS_RVA) as u32,
            (KNOWN_KINDS as usize + 1) * GROUPS_LEN,
        ),
    ];
    let saved = (capture(&regions), capture(&inputs));
    let known: Vec<u32> = (0..KNOWN_KINDS as usize)
        .map(|n| unsafe { game_read(KNOWN_KINDS_RVA + n * 4) })
        .collect();
    let mut seed = 0x10AD_0B1Eu32;
    let mut lot = |below: u32| {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (seed >> 8) % below
    };
    let name = std::cell::Cell::new([0xAAu8; 12]);
    let mut fatal = [0usize; 4];
    for case in 0..200u32 {
        let slot = match lot(8) {
            0 => lot(0x50),
            1 => [0x40u32, 0x41, 0x4F][lot(3) as usize],
            // A car's first slot, its parts 0xA and 0xB, and any other.
            2 | 3 => 0x50 + lot(8) * 0x14,
            4 => 0x50 + lot(8) * 0x14 + 0xA,
            5 => 0x50 + lot(8) * 0x14 + 0xB,
            _ => 0x50 + lot(0xA0),
        };
        let kind = match lot(6) {
            0 => 0,
            1 => 0x1234,
            n => known[lot(KNOWN_KINDS) as usize].wrapping_add(n & 1),
        };
        let read = [0u32, 0x100, 0x40000][lot(3).max(lot(3)) as usize];
        // Mostly room enough; now and then one of the three is used up.
        let short = lot(12);
        let groups: Vec<u32> = (0..(KNOWN_KINDS as usize + 1) * GROUPS_LEN / 4)
            .map(|_| match lot(5) {
                0 => u32::MAX,
                _ => lot(40),
            })
            .collect();
        let run = |rust: bool| unsafe {
            restore(&regions, &vec![0xAA; saved.0.len()]);
            name.set([0xAA; 12]);
            *READS.lock().unwrap() = (read, name.as_ptr() as u32);
            std::ptr::copy_nonoverlapping(
                groups.as_ptr() as *const u8,
                game_addr(GROUPS_RVA) as *mut u8,
                groups.len() * 4,
            );
            game_write(SLOT_RVA, slot);
            game_write(KIND_RVA, kind);
            game_write(NAMES_RVA, name.as_ptr() as u32);
            game_write(FACE_CURSOR_RVA, if short == 1 { 0x1001u32 } else { 0x1000 });
            game_write(FACE_ROOM_RVA, 0x1000u32);
            game_write(POINT_COUNT_RVA, if short == 2 { 101u32 } else { 100 });
            game_write(POINTS_ROOM_RVA, 1200u32);
            game_write(FACE_COUNT_RVA, if short == 3 { FACES } else { FACES - 1 });
            CALLED.lock().unwrap().clear();
            set_handler(handler);
            let before = Regs::SENTINEL;
            let mut r = before;
            let failed = if rust {
                match load_object(&mut r, &mut |call| {
                    let mut regs = Regs {
                        eax: match call {
                            Call::Names(kind) => kind,
                            _ => 0,
                        },
                        ..Regs::default()
                    };
                    act(place_of(call), &mut regs);
                    Ok(())
                }) {
                    Ok(()) => 0,
                    Err(Refused::Fatal(e)) => e.code,
                    Err(_) => 1,
                }
            } else {
                call_game(&mut r, copy.addr());
                WENT_FATAL.load(Ordering::SeqCst)
            };
            // Only the names' call is given a register that the test looks at.
            let called: Vec<[u32; 4]> = CALLED
                .lock()
                .unwrap()
                .iter()
                .map(|call| {
                    [
                        call[0],
                        call[1],
                        call[2],
                        if call[0] == 0 { call[3] } else { 0 },
                    ]
                })
                .collect();
            (
                [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp],
                capture(&regions),
                name.get(),
                called,
                failed,
            )
        };
        let (theirs, mine) = (run(false), run(true));
        let same_end = (theirs.4 != 0) == (mine.4 != 0);
        fatal[match mine.4 {
            0 => 0,
            0x8060_0010 => 1,
            0x8060_0011 => 2,
            0x8060_0012 => 3,
            _ => 0,
        }] += 1;
        t.check(
            same_end && (theirs.0, &theirs.1, theirs.2, &theirs.3) == (mine.0, &mine.1, mine.2, &mine.3),
            || {
                let first = theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b);
                format!(
                    "case {case}: slot {slot:#x}, kind {kind:#x}, read {read:#x}, short {short}: original regs {:x?} name {:x?} fatal {:#x} calls {:x?}; rust regs {:x?} name {:x?} fatal {:#x} calls {:x?}; the first difference in memory at byte {first:x?}",
                    theirs.0, theirs.2, theirs.4, theirs.3, mine.0, mine.2, mine.4, mine.3
                )
            },
        );
    }
    t.check(fatal.iter().all(|&n| n > 0), || {
        format!("the cases ended well and in the three fatal errors {fatal:?} times")
    });
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
