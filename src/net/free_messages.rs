//! `net::free_messages` (0x4185BB): the network's memory given back (`memory::free_list`
//! on `stru_52A928`) and four of its counts (`dword_52AC34`, `dword_52AC38`,
//! `dword_52ACF4`, `dword_52ACF8`) made 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! See `net/mod.rs`. One caller, `net::destroy_connection`. Its call through `part` (the
//! Rust `free_list`, which refuses the game's own heap before anything). Shadow mode runs
//! the original alone. Self-test: a copy whose list and counts are the test's and whose
//! call goes to a noting stand-in, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::input::effects::compare;
use crate::net::{FREE_LIST, Part, a, dispatch_net, net_entry};
use crate::system::api_stand_ins::copy_with;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::free_messages", rva(0x4185BB), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const LIST: usize = 0x52A928; // stru_52A928
const COUNTS: [usize; 4] = [0x52AC34, 0x52AC38, 0x52ACF4, 0x52ACF8];

/// The Rust version on the list at `list` and the counts at `counts`.
pub fn free_messages_with(
    r: &mut Regs,
    list: u32,
    counts: [u32; 4],
    part: Part,
) -> Result<(), Refused> {
    let mut g = Regs { ebx: list, ..*r };
    part(FREE_LIST, &mut g)?;
    for at in counts {
        unsafe { (at as *mut u32).write_volatile(0) };
    }
    Ok(())
}

/// The Rust version.
pub fn free_messages(r: &mut Regs, part: Part) -> Result<(), Refused> {
    free_messages_with(r, a(LIST), COUNTS.map(a), part)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_net(&HOOK, r, call_original, free_messages);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, free_messages)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut mem = [0u32; 8];
    let base = mem.as_mut_ptr() as u32;
    let counts = [base + 0x10, base + 0x14, base + 0x18, base + 0x1C];
    // pusha; lea ebx, stru_52A928
    let entry = [0x60, 0x8D, 0x1D, 0x28, 0xA9, 0x52, 0x00];
    let copy = match copy_with(
        0x4185BB,
        0x24,
        &entry,
        &[7],
        &[FREE_LIST],
        &[],
        &[
            (LIST, 0x10, base),
            (COUNTS[0], 8, counts[0]),
            (COUNTS[2], 8, counts[2]),
        ],
        5,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    compare(
        &mut t,
        "the list and the counts",
        &copy,
        Regs::SENTINEL,
        &|| unsafe {
            for k in 0..8u32 {
                ((base + 4 * k) as *mut u32).write_volatile(0x1111_1111 * (k + 1));
            }
        },
        &[(base, 0x20)],
        &|r, part| free_messages_with(r, base, counts, part),
        false,
    );
    let _ = &mut mem;
    t.finish()
}
