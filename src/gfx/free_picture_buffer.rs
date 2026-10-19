//! `gfx::free_picture_buffer` (0x4060B7): the picture buffer given back at the game's end:
//! `g_pictureFlags` and `g_pictureBuffer` 0, its list (`stru_4F49E4`) freed
//! (`memory::free_list`). The other half of `gfx::allocate_picture_buffer`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! One caller, `start`'s way out. Its call through `part` (the Rust `free_list`). Refuses
//! (before anything) what `free_list` refuses: the game's own heap with blocks on the list.
//! Shadow mode runs the original alone (memory is given back). Self-test: a copy whose
//! three variables are the test's and whose call goes to a noting stand-in, against the
//! Rust version.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, hook_asm, rva};
use crate::memory::OWN_HEAP_RVA;
use crate::system::api_stand_ins::{Api, copy_with};
use crate::system::exit_parts::{FREE_LIST, Part, compare_exit, dispatch_exit};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("gfx::free_picture_buffer", rva(0x4060B7), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const FLAGS: usize = 0x4F49CC; // g_pictureFlags
const BUFFER: usize = 0x4F49DC; // g_pictureBuffer
const LIST: usize = 0x4F49E4; // stru_4F49E4

/// The Rust version on the variables at `flags`, `buffer` and the list at `list`.
pub fn free_picture_buffer_with(
    r: &mut Regs,
    flags: u32,
    buffer: u32,
    list: u32,
    part: Part,
) -> Result<(), Refused> {
    let head = unsafe { (list as *const u32).read_volatile() };
    if head != 0 && unsafe { game_read::<u32>(OWN_HEAP_RVA) } == 1 {
        return Err(BadInput("the game's own heap is in use (altmemenable)").into());
    }
    let mut g = Regs {
        eax: 0,
        ebx: list,
        ..*r
    };
    unsafe {
        (flags as *mut u32).write_volatile(0);
        (buffer as *mut u32).write_volatile(0);
    }
    part(FREE_LIST, &mut g)
}

fn in_game(r: &mut Regs, part: Part, _api: Api) -> Result<(), Refused> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    free_picture_buffer_with(r, a(FLAGS), a(BUFFER), a(LIST), part)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_exit(&HOOK, r, call_original, in_game);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut mem = [0u32; 8];
    let base = mem.as_mut_ptr() as u32;
    let (flags, buffer, list) = (base, base + 4, base + 0x10);
    // pusha; xor eax, eax; mov g_pictureFlags, eax
    let entry = [0x60, 0x33, 0xC0, 0xA3, 0xCC, 0x49, 0x4F, 0x00];
    let copy = match copy_with(
        0x4060B7,
        0x1A,
        &entry,
        &[0x13],
        &[FREE_LIST],
        &[],
        &[(FLAGS, 4, flags), (BUFFER, 4, buffer), (LIST, 0x10, list)],
        3,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    let own_heap = unsafe { game_read::<u32>(OWN_HEAP_RVA) } == 1;
    for head in [0u32, 0x1234_0000] {
        compare_exit(
            &mut t,
            &format!("head {head:#x}"),
            &copy,
            Regs::SENTINEL,
            &|| unsafe {
                (base as *mut [u32; 8]).write_volatile([7, 0x0BAD_F00D, 1, 2, head, 5, 6, 7])
            },
            &[(base, 0x20)],
            &|r, part, _| free_picture_buffer_with(r, flags, buffer, list, part),
            own_heap && head != 0,
        );
    }
    let _ = &mut mem;
    t.finish()
}
