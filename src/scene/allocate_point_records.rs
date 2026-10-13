//! `scene::allocate_point_records` (0x41F976): takes the memory of the points'
//! records.
//!
//! In:  nothing
//! Out: eax = the memory, ebx = the list it is taken on; everything else preserved
//!
//! 0x40000 bytes (0x2000 records of 0x20 bytes: 0x1000 for a scene's points and
//! 0x1000 for an object's) on the list `stru_5C6E50`, counted in `dword_5C6F90`, to
//! `g_pointRecords`.
//! One caller, the front end's 3D set-up.
//! Shadow mode runs the original alone.
//! Self-test: a redirected copy whose allocation is the test's.

use crate::errors::Refused;
use crate::framework::{
    EAX, EBX, RawHook, Regs, SelfTest, call_game, game_addr, game_ptr, game_read, game_write,
    hook_asm, rva, shadow,
};
use crate::install::stand_ins::{act, set_handler, stand_in};
use crate::memory::allocate::allocate;
use crate::redirect::RedirectedCopy;
use crate::scene::allocate_object_memory::TAKEN_RVA;
use crate::scene::project_point::RECORDS_RVA;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "scene::allocate_point_records",
    rva(0x41F976),
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

const LIST_RVA: usize = rva(0x5C6E50); // stru_5C6E50
const BYTES: u32 = 0x4_0000;

// ---- Rust version ----

/// `take`: what stands for `memory::allocate`, given the bytes and the list.
pub unsafe fn allocate_point_records(
    take: &mut dyn FnMut(u32, u32) -> Result<u32, Refused>,
) -> Result<Left, Refused> {
    unsafe {
        let list = game_addr(LIST_RVA) as u32;
        game_write(TAKEN_RVA, game_read::<u32>(TAKEN_RVA).wrapping_add(BYTES));
        let block = take(BYTES, list)?;
        game_write(RECORDS_RVA, block);
        Ok(Left::nothing().with(EAX, block).with(EBX, list))
    }
}

pub unsafe fn allocate_point_records_in_game() -> Result<Left, Refused> {
    unsafe {
        allocate_point_records(&mut |bytes, _| Ok(allocate(game_ptr(LIST_RVA), bytes)? as u32))
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    match unsafe { allocate_point_records_in_game() } {
        Ok(left) => left.apply(r),
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

// ---- self-test ----

const LEN: usize = 0x1C;
const ENTRY_BYTES: [u8; 5] = [0xB8, 0x00, 0x00, 0x04, 0x00];
const CALLS: [usize; 1] = [0x11];

static CALLED: Mutex<Vec<[u32; 2]>> = Mutex::new(Vec::new());

fn handler(_: u32, r: &mut Regs) {
    CALLED.lock().unwrap().push([r.eax, r.ebx]);
    r.eax = 0x0BAD_0000;
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x41F976)),
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
    let regions = [at(TAKEN_RVA), at(RECORDS_RVA)];
    let saved = capture(&regions);
    for taken in [0u32, 0x58_0000, 0xFFFF_0000] {
        let run = |rust: bool| unsafe {
            game_write(TAKEN_RVA, taken);
            game_write(RECORDS_RVA, 0xAAAA_AAAAu32);
            CALLED.lock().unwrap().clear();
            set_handler(handler);
            let mut r = Regs::SENTINEL;
            if rust {
                let left = allocate_point_records(&mut |bytes, list| {
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
            format!("taken {taken:#x}: original {theirs:x?}; rust {mine:x?}")
        });
    }
    restore(&regions, &saved);
    t.finish()
}
