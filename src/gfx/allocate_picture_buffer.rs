//! `gfx::allocate_picture_buffer` (0x40608F): takes the memory pictures are decoded in.
//!
//! In:  nothing
//! Out: every register preserved
//! `dword_4F49E0` = 0x1D4C00 (800 x 600 dwords); that and 0xC80 bytes more from
//! `memory::allocate` on the list `stru_4F49E4`; the address to `g_pictureBuffer`
//! (0x4F49DC). A failed allocation would be fatal error 0x80A19090, but the allocator
//! reports its own and does not come back. One caller, `start`. `sub_4060B7`, after it
//! in the listing, gives the memory back at the game's end.
//! Shadow mode runs the original alone (memory is taken).
//! Self-test: a redirected copy whose three variables are the test's.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva, shadow};
use crate::memory::MemoryList;
use crate::memory::allocate::allocate;
use crate::memory::free_list::free_list;
use crate::redirect::RedirectedCopy;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "gfx::allocate_picture_buffer",
    rva(0x40608F),
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

/// dword_4F49E0, stru_4F49E4, g_pictureBuffer.
const PLACES: [usize; 3] = [0x4F49E0, 0x4F49E4, 0x4F49DC];
const BUFFER_LEN: u32 = 0x1D_4C00;
const SPARE: u32 = 0xC80;

// ---- Rust version ----

/// `size`, `list` and `buffer` are the three variables' addresses.
pub unsafe fn allocate_picture_buffer(
    size: *mut u32,
    list: *mut MemoryList,
    buffer: *mut u32,
) -> Result<(), Refused> {
    unsafe {
        size.write_unaligned(BUFFER_LEN);
        let memory = allocate(list, BUFFER_LEN + SPARE)?;
        buffer.write_unaligned(memory as u32);
    }
    Ok(())
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let [size, list, buffer] = PLACES.map(|at| game_addr(rva(at)));
    let done = unsafe { allocate_picture_buffer(size as _, list as _, buffer as _) };
    if let Err(e) = done {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
    }
}

// ---- self-test ----

const LEN: usize = 0x28;
const ENTRY_BYTES: [u8; 8] = [0x60, 0xB8, 0x00, 0x4C, 0x1D, 0x00, 0xA3, 0xE0];
/// `memory::allocate`. The `jb` after it, to the fatal stub, is never taken.
const CALLS: [usize; 1] = [0x16];

#[derive(Default)]
#[repr(C)]
struct Test {
    size: u32,
    list: [u32; 4],
    buffer: u32,
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = unsafe {
        RedirectedCopy::new(game_addr(rva(0x40608F)), LEN, &ENTRY_BYTES, &CALLS, &|_| {
            None
        })
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let mut test = Box::new(Test::default());
    let places = [
        &raw mut test.size as u32,
        &raw mut test.list as u32,
        &raw mut test.buffer as u32,
    ];
    unsafe {
        let code = std::slice::from_raw_parts(copy.addr() as *const u8, LEN);
        let real = PLACES.map(|at| game_addr(rva(at)) as u32);
        let mut patches = Vec::new();
        let mut at = 0;
        while at + 4 <= LEN {
            let named = u32::from_le_bytes(code[at..at + 4].try_into().unwrap());
            match real.iter().position(|&address| address == named) {
                Some(n) => {
                    patches.push((at, places[n]));
                    at += 4;
                }
                None => at += 1,
            }
        }
        if patches.len() != 3 {
            return t.skip("the function does not name its variables as the test expects");
        }
        for (at, value) in patches {
            copy.patch_u32(at, value);
        }
    }
    let mut run = |rust: bool| unsafe {
        *test = Test::default();
        let list = &raw mut test.list as *mut MemoryList;
        let preserved = if rust {
            allocate_picture_buffer(&raw mut test.size, list, &raw mut test.buffer).is_ok()
        } else {
            let before = Regs::SENTINEL;
            let mut ro = before;
            call_game(&mut ro, copy.addr());
            ro.preserved_except(&before, 0)
        };
        // What was left, with the addresses made relative: the buffer lies 0x40 bytes
        // after the list's newest block.
        let left = (
            preserved,
            test.size,
            test.list[1],
            test.buffer != 0 && test.buffer.wrapping_sub(test.list[0]) == 0x40,
        );
        let _ = free_list(list);
        left
    };
    let (theirs, mine) = (run(false), run(true));
    t.check(
        theirs == mine && theirs == (true, BUFFER_LEN, BUFFER_LEN + SPARE, true),
        || format!("original {theirs:x?}; rust {mine:x?}"),
    );
    t.finish()
}
