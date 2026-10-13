//! `scene::reset_objects` (0x425CFF): empties the objects' tables before they are
//! loaded.
//!
//! In:  nothing
//! Out: eax = 0xFFFFFFFF, ecx = 0, edi = the end of the kinds' table
//!
//! The points' memory zeroed (its room `dword_5C6FAC`, in dwords); the slots' first
//! points, numbers of points and normals, first faces and numbers of faces zeroed;
//! `g_objectPointCount` = 1 (point 0 is never an object's), the faces' number and the
//! faces' memory used = 0; every slot's kind = -1.
//! One caller, `scene::load_front_end_objects`.
//! Refuses points' room without memory.
//! Self-test: on memory of the test's; the tables are put back afterwards.

use crate::errors::{BadInput, Refused};
use crate::framework::{
    EAX, ECX, EDI, RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva,
};
use crate::scene::objects::{
    FACE_COUNT_RVA, FACE_CURSOR_RVA, FIRST_POINT_RVA, OBJECT_KINDS_RVA, POINT_COUNT_RVA,
    POINTS_ROOM_RVA, POINTS_RVA, SLOTS,
};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("scene::reset_objects", rva(0x425CFF), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The five tables of 0x100 dwords from `g_objectFirstPoint` on.
const TABLES_LEN: usize = SLOTS as usize * 4 * 5;

// ---- Rust version ----

pub unsafe fn reset_objects() -> Result<Left, BadInput> {
    unsafe {
        let (memory, room): (u32, u32) = (game_read(POINTS_RVA), game_read(POINTS_ROOM_RVA));
        if memory == 0 && room >= 4 {
            return Err(BadInput("room for the objects' points without memory"));
        }
        std::ptr::write_bytes(memory as *mut u32, 0, (room >> 2) as usize);
        std::ptr::write_bytes(game_addr(FIRST_POINT_RVA) as *mut u8, 0, TABLES_LEN);
        game_write(POINT_COUNT_RVA, 1u32);
        game_write(FACE_COUNT_RVA, 0u32);
        game_write(FACE_CURSOR_RVA, 0u32);
        game_write(OBJECT_KINDS_RVA, [u32::MAX; SLOTS as usize]);
        Ok(Left::nothing()
            .with(EAX, u32::MAX)
            .with(ECX, 0)
            .with(EDI, game_addr(OBJECT_KINDS_RVA) as u32 + SLOTS * 4))
    }
}

// ---- hook ----

fn regions() -> Vec<Region> {
    let mut regions = vec![
        (game_addr(FIRST_POINT_RVA) as u32, TABLES_LEN),
        (game_addr(OBJECT_KINDS_RVA) as u32, SLOTS as usize * 4),
        at(POINT_COUNT_RVA),
        at(FACE_COUNT_RVA),
        at(FACE_CURSOR_RVA),
    ];
    let (memory, room): (u32, u32) = unsafe { (game_read(POINTS_RVA), game_read(POINTS_ROOM_RVA)) };
    if memory != 0 {
        regions.push((memory, (room & !3) as usize));
    }
    regions
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { reset_objects() }?)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let inputs = [at(POINTS_RVA), at(POINTS_ROOM_RVA)];
    let mut tables = regions();
    tables.truncate(5);
    let saved = (capture(&inputs), capture(&tables));
    let memory = vec![0u8; 0x110];
    for room in [0u32, 3, 4, 0x47, 0x100] {
        let mut regions = tables.clone();
        regions.push((memory.as_ptr() as u32, memory.len()));
        check_state(
            &mut t,
            &format!("room {room:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&regions, &vec![0xAA; saved.1.len() + memory.len()]);
                game_write(POINTS_RVA, memory.as_ptr() as u32);
                game_write(POINTS_ROOM_RVA, room);
            },
            &regions,
            &rust,
        );
    }
    restore(&tables, &saved.1);
    restore(&inputs, &saved.0);
    t.finish()
}
