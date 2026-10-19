//! `memory::free_own_heap` (0x4029DB): at the game's end, with the game's own heap in use
//! (`altmemenable`, `dword_4E0584` 1), its memory (`dword_4E058C`) given back
//! (`memory::virtual_free`) and it and `dword_4E0590` made 0. The other half of
//! `sub_40298A` (agent A's: the heap's making).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! One caller, `start`'s way out. Its call through `part` (in the game `VirtualFree` with
//! `MEM_RELEASE`, as `virtual_free`). Shadow mode runs the original alone (memory is given
//! back). Self-test: a copy whose three variables are the test's and whose call goes to a
//! noting stand-in, against the Rust version, with the heap on and off.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::system::api_stand_ins::{Api, copy_with};
use crate::system::exit_parts::{Part, VIRTUAL_FREE, compare_exit, dispatch_exit};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("memory::free_own_heap", rva(0x4029DB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const OWN_HEAP: usize = 0x4E0584; // g_ownHeap (altmemenable)
const HEAP: usize = 0x4E058C; // dword_4E058C: the heap's memory
const HEAP_MORE: usize = 0x4E0590; // dword_4E0590

/// The Rust version on the switch at `on` and the heap's two dwords at `heap`, `more`.
pub fn free_own_heap_with(
    r: &mut Regs,
    on: u32,
    heap: u32,
    more: u32,
    part: Part,
) -> Result<(), Refused> {
    let rd = |at: u32| unsafe { (at as *const u32).read_volatile() };
    if rd(on) != 1 {
        return Ok(());
    }
    let mut g = Regs {
        eax: rd(heap),
        ..*r
    };
    part(VIRTUAL_FREE, &mut g)?;
    unsafe {
        (heap as *mut u32).write_volatile(0);
        (more as *mut u32).write_volatile(0);
    }
    Ok(())
}

fn in_game(r: &mut Regs, part: Part, _api: Api) -> Result<(), Refused> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    free_own_heap_with(r, a(OWN_HEAP), a(HEAP), a(HEAP_MORE), part)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_exit(&HOOK, r, call_original, in_game);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut mem = [0u32; 4];
    let base = mem.as_mut_ptr() as u32;
    let (on, heap, more) = (base, base + 4, base + 8);
    // pusha; cmp g_ownHeap, 1
    let entry = [0x60, 0x83, 0x3D, 0x84, 0x05, 0x4E, 0x00, 0x01];
    let copy = match copy_with(
        0x4029DB,
        0x2A,
        &entry,
        &[0xF],
        &[VIRTUAL_FREE],
        &[],
        &[(OWN_HEAP, 4, on), (HEAP, 4, heap), (HEAP_MORE, 4, more)],
        4,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    for switch in [0u32, 1, 2] {
        compare_exit(
            &mut t,
            &format!("switch {switch}"),
            &copy,
            Regs::SENTINEL,
            &|| unsafe { (base as *mut [u32; 4]).write_volatile([switch, 0x0123_0000, 9, 0x77]) },
            &[(base, 0x10)],
            &|r, part, _| free_own_heap_with(r, on, heap, more, part),
            false,
        );
    }
    let _ = &mut mem;
    t.finish()
}
