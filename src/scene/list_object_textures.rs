//! `scene::list_object_textures` (0x426ACA): the different textures the faces of the
//! file just read use.
//!
//! In:  nothing (the file in `byte_A2BB54`, the slot in `dword_634FEC`)
//! Out: every register preserved
//!
//! Goes through the file's faces (after its points; a face is 0x10 bytes and as many
//! again for each vertex) and lists each face's texture (its word +4) once, in the
//! order met, in `dword_631B6C`; their number to `dword_631B68`. For a slot under
//! 0x40 the number and the list also go to the slot's record of 0x40 bytes at
//! `unk_5DCEBC`; more than fifteen textures run on into the next slots' records, as
//! in the original, while that stays inside the 0x40 records.
//! One caller, `scene::load_object`, for slots up to 0x40.
//! Refuses a file of no faces (the original goes through 2^32), faces that run past
//! the file's room, more than 0x100 textures, and a list that would leave the records.
//! Self-test: 30 files by lot in the game's buffer, put back afterwards.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::scene::objects::{FILE_ROOM, FILE_RVA, SLOT_RVA};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::list_object_textures",
    rva(0x426ACA),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const COUNT_RVA: usize = rva(0x631B68); // dword_631B68, then the list dword_631B6C
const MOST: usize = 0x100;
const RECORDS_RVA: usize = rva(0x5DCEBC); // unk_5DCEBC
const RECORDS_LEN: usize = 0x40 * 0x40;

// ---- Rust version ----

/// The file's bytes at an offset, if they lie in its room.
pub unsafe fn file_dword(offset: usize) -> Result<u32, BadInput> {
    if offset + 4 > FILE_ROOM {
        return Err(BadInput("the object's file runs past its room"));
    }
    Ok(unsafe { (game_addr(FILE_RVA + offset) as *const u32).read_unaligned() })
}

pub unsafe fn list_object_textures() -> Result<(), BadInput> {
    unsafe {
        let (points, faces) = (file_dword(4)?, file_dword(8)?);
        if faces == 0 {
            return Err(BadInput("an object's file of no faces"));
        }
        let mut at = (points as usize)
            .checked_mul(12)
            .and_then(|bytes| bytes.checked_add(0x2C))
            .ok_or(BadInput("the object's file runs past its room"))?;
        let mut textures: Vec<u32> = Vec::new();
        for _ in 0..faces {
            let vertices = file_dword(at)?;
            let texture = file_dword(at + 4)? & 0xFFFF;
            if !textures.contains(&texture) {
                if textures.len() == MOST {
                    return Err(BadInput("more textures than the list holds"));
                }
                textures.push(texture);
            }
            at = (vertices as usize)
                .checked_mul(0x10)
                .and_then(|bytes| bytes.checked_add(at + 0x10))
                .ok_or(BadInput("the object's file runs past its room"))?;
        }
        let slot: u32 = game_read(SLOT_RVA);
        let record = slot as usize * 0x40;
        if slot < 0x40 && record + 4 + textures.len() * 4 > RECORDS_LEN {
            return Err(BadInput("a texture list that leaves the slots' records"));
        }
        game_write(COUNT_RVA, textures.len() as u32);
        for (n, &texture) in textures.iter().enumerate() {
            game_write(COUNT_RVA + 4 + n * 4, texture);
        }
        if slot < 0x40 {
            game_write(RECORDS_RVA + record, textures.len() as u32);
            for (n, &texture) in textures.iter().enumerate() {
                game_write(RECORDS_RVA + record + 4 + n * 4, texture);
            }
        }
        Ok(())
    }
}

// ---- hook ----

fn regions() -> [Region; 2] {
    [
        (game_addr(COUNT_RVA) as u32, 4 + MOST * 4),
        (game_addr(RECORDS_RVA) as u32, RECORDS_LEN),
    ]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { list_object_textures() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    const TEST_LEN: usize = 0x2000;
    let inputs = [(game_addr(FILE_RVA) as u32, TEST_LEN), at(SLOT_RVA)];
    let saved = (capture(&regions), capture(&inputs));
    let mut seed = 0x7E87_0006u32;
    let mut lot = |below: u32| {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (seed >> 8) % below
    };
    for case in 0..30u32 {
        let slot = [0u32, 5, 0x3E, 0x3F, 0x40, 0xF3][case as usize % 6];
        let points = lot(5);
        let faces = 1 + lot(if case % 5 == 4 { 40 } else { 9 });
        // The last slot's record has no next one to run into.
        let kinds = 1 + lot(if case % 5 == 4 && slot != 0x3F { 30 } else { 4 });
        let mut file = vec![0xAAAA_AAAAu32; TEST_LEN / 4];
        (file[1], file[2]) = (points, faces);
        let mut at = 11 + points as usize * 3;
        for _ in 0..faces {
            let vertices = lot(5);
            file[at] = vertices;
            // The texture is the low word only.
            file[at + 1] = 0x5A5A_0000 | (0x100 + lot(kinds));
            at += 4 + vertices as usize * 4;
        }
        check_state(
            &mut t,
            &format!(
                "case {case}: slot {slot:#x}, {points} points, {faces} faces of {kinds} textures"
            ),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&regions, &vec![0xAA; saved.0.len()]);
                std::ptr::copy_nonoverlapping(
                    file.as_ptr() as *const u8,
                    game_addr(FILE_RVA) as *mut u8,
                    TEST_LEN,
                );
                game_write(SLOT_RVA, slot);
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
