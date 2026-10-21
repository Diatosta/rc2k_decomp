//! `scene::allocate_object_memory` (0x41F814): takes the memory the objects' points
//! and faces are loaded into.
//!
//! In:  nothing
//! Out: eax = the memory's end, ebx = the list it is taken on; everything else
//!      preserved
//!
//! One block on the list `stru_5C6DD0`, counted in `dword_5C6F90`: in a race 0x300000
//! bytes, 0xC0000 of them for the points and 0x240000 for the faces; else 0x580000,
//! 0x180000 and 0x400000. `g_objectPoints` = the block, `dword_5C6FA8` (the faces'
//! memory) = what follows the points' part, `dword_5C6FAC` and `dword_5C6FB0` the two
//! rooms.
//! One caller, `scene::set_up_object_memory`.
//! Shadow mode runs the original alone and checks what it left.
//! Self-test: a redirected copy whose allocation is the test's.

use crate::errors::Refused;
use crate::framework::{
    EAX, EBX, RawHook, Regs, SelfTest, call_game, game_addr, game_ptr, game_read, game_write,
    hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::memory::allocate::allocate;
use crate::redirect::RedirectedCopy;
use crate::scene::objects::{FACE_MEMORY_RVA, FACE_ROOM_RVA, POINTS_ROOM_RVA, POINTS_RVA};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::allocate_object_memory",
    rva(0x41F814),
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

const IN_RACE_RVA: usize = rva(0x5C5CEC); // g_inRace
const LIST_RVA: usize = rva(0x5C6DD0); // stru_5C6DD0
pub const TAKEN_RVA: usize = rva(0x5C6F90); // dword_5C6F90

use rc2k_core::scene::objects::object_rooms as rooms;

// ---- Rust version ----

/// `take`: what stands for `memory::allocate`, given the bytes and the list.
// Converted: `rc2k_core::scene::objects::object_rooms`
pub unsafe fn allocate_object_memory(
    take: &mut dyn FnMut(u32, u32) -> Result<u32, Refused>,
) -> Result<Left, Refused> {
    unsafe {
        let (points, faces) = rooms(game_read::<u32>(IN_RACE_RVA) != 0);
        let list = game_addr(LIST_RVA) as u32;
        game_write(
            TAKEN_RVA,
            game_read::<u32>(TAKEN_RVA).wrapping_add(points + faces),
        );
        let block = take(points + faces, list)?;
        game_write(POINTS_RVA, block);
        game_write(FACE_MEMORY_RVA, block.wrapping_add(points));
        game_write(POINTS_ROOM_RVA, points);
        game_write(FACE_ROOM_RVA, faces);
        Ok(Left::nothing()
            .with(EAX, block.wrapping_add(points + faces))
            .with(EBX, list))
    }
}

pub unsafe fn allocate_object_memory_in_game() -> Result<Left, Refused> {
    unsafe {
        allocate_object_memory(&mut |bytes, _| Ok(allocate(game_ptr(LIST_RVA), bytes)? as u32))
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        let (points, faces) = rooms(unsafe { game_read::<u32>(IN_RACE_RVA) } != 0);
        unsafe { call_original(r) };
        let left: [u32; 4] = unsafe {
            [
                game_read::<u32>(FACE_MEMORY_RVA).wrapping_sub(game_read(POINTS_RVA)),
                game_read(POINTS_ROOM_RVA),
                game_read(FACE_ROOM_RVA),
                r.eax.wrapping_sub(game_read(POINTS_RVA)),
            ]
        };
        if left != [points, points, faces, points + faces] {
            HOOK.mismatch(|| {
                format!("the original left {left:x?}; rust rooms {points:#x}, {faces:#x}")
            });
        }
        return;
    }
    match unsafe { allocate_object_memory_in_game() } {
        Ok(left) => left.apply(r),
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

// ---- self-test ----

const LEN: usize = 0x87;
const ENTRY_BYTES: [u8; 7] = [0x83, 0x3D, 0xEC, 0x5C, 0x5C, 0x00, 0x00];
const CALLS: [usize; 2] = [0x1A, 0x59];
const BLOCK: u32 = 0x1234_0000;

/// Each allocation: the bytes and the list.
static CALLED: Mutex<Vec<[u32; 2]>> = Mutex::new(Vec::new());

fn handler(_: u32, r: &mut Regs) {
    CALLED.lock().unwrap().push([r.eax, r.ebx]);
    r.eax = BLOCK;
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x41F814)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| (callee == rva(0x4028A7)).then(|| stand_in(0)),
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let regions: [Region; 5] = [
        at(TAKEN_RVA),
        at(POINTS_RVA),
        at(FACE_MEMORY_RVA),
        at(POINTS_ROOM_RVA),
        at(FACE_ROOM_RVA),
    ];
    let inputs = [at(IN_RACE_RVA)];
    let saved = (capture(&regions), capture(&inputs));
    for (in_race, taken) in [
        (0u32, 0u32),
        (1, 0x100),
        (0x80, 0xFFFF_0000),
        (0, 0x7000_0000),
    ] {
        let run = |rust: bool| unsafe {
            restore(&regions, &vec![0xAA; saved.0.len()]);
            game_write(IN_RACE_RVA, in_race);
            game_write(TAKEN_RVA, taken);
            CALLED.lock().unwrap().clear();
            set_handler(handler);
            let mut r = Regs::SENTINEL;
            if rust {
                let left = allocate_object_memory(&mut |bytes, list| {
                    let mut regs = Regs {
                        eax: bytes,
                        ebx: list,
                        ..Regs::default()
                    };
                    act(0, &mut regs);
                    Ok(regs.eax)
                });
                match left {
                    Ok(left) => left.apply(&mut r),
                    Err(_) => r = Regs::default(),
                }
            } else {
                call_game(&mut r, copy.addr());
            }
            (
                [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp],
                capture(&regions),
                CALLED.lock().unwrap().clone(),
            )
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.2.len() == 1, || {
            format!(
                "in a race {in_race:#x}, taken {taken:#x}: original {theirs:x?}; rust {mine:x?}"
            )
        });
    }
    restore(&regions, &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
