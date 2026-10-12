//! `menu::show_loading_picture` (0x47CEA8): shows the picture "loading".
//!
//! In:  nothing
//! Out: as `gfx::show_picture_file` leaves the registers (eax = 1, or the path if the
//!      file could not be read)
//! `gfx::show_picture_file` on `files\g\graphics\screens\loading.pcx`.
//! Fifteen callers, `menu::set_up_front_end` among them.
//! Shadow mode runs the original alone (it shows frames).
//! Self-test: a redirected copy whose call notes the path.

use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::gfx::show_picture_file::show_picture_file;
use crate::install::stand_ins::{set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::show_loading_picture",
    rva(0x47CEA8),
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

const PATH_RVA: usize = rva(0x731BBF); // "files\g\graphics\screens\loading.pcx"

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let esp = r.esp;
    r.eax = game_addr(PATH_RVA) as u32;
    unsafe {
        show_picture_file(r, &mut |at, regs| call_game(regs, game_addr(rva(at))));
    }
    r.esp = esp;
}

// ---- self-test ----

const LEN: usize = 0x0C;
const ENTRY_BYTES: [u8; 8] = [0x8D, 0x05, 0xBF, 0x1B, 0x73, 0x00, 0xE8, 0x78];
const CALLS: [usize; 1] = [0x06];

/// The path the stand-in was given.
static GIVEN: AtomicU32 = AtomicU32::new(0);

fn note_path(_: u32, r: &mut Regs) {
    GIVEN.store(r.eax, Ordering::SeqCst);
    r.eax = 0x77;
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(game_addr(rva(0x47CEA8)), LEN, &ENTRY_BYTES, &CALLS, &|_| {
            Some(stand_in(0))
        })
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    GIVEN.store(0, Ordering::SeqCst);
    set_handler(note_path);
    let before = Regs::SENTINEL;
    let mut ro = before;
    unsafe { call_game(&mut ro, copy.addr()) };
    let given = GIVEN.load(Ordering::SeqCst);
    let path = unsafe { std::ffi::CStr::from_ptr(game_addr(PATH_RVA) as *const i8) };
    t.check(
        given == game_addr(PATH_RVA) as u32
            && ro.eax == 0x77
            && ro.preserved_except(&before, crate::framework::EAX)
            && path.to_bytes().ends_with(b"loading.pcx"),
        || {
            format!(
                "the original gave {given:#x} and came back with eax {:#x}; the path is {path:?}",
                ro.eax
            )
        },
    );
    t.finish()
}
