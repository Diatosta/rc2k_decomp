//! `net::shut_down` (0x4184D6): the network at the game's end: `net::stop`, then the
//! received messages' mutex (`dword_52AB6C`) and the receiving thread's event
//! (`dword_52AB68`) taken out (`xchg` with 0) and closed (`sub_411092`, `sub_41104F`:
//! `CloseHandle` if not 0).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! See `net/mod.rs`. Two callers: `start`'s way out (0x4010EE) and `sub_4104F2`. Its calls
//! through `part` (in the game `stop`'s Rust version and `CloseHandle`). A call in its first
//! five bytes: private copy (0x22 bytes, three calls relocated), manual patch. It may wait
//! on the receiving thread: the thunk does not wait for self-tests. Shadow mode runs the
//! original alone (handles are closed). Self-test: a copy whose two handles are the
//! test's and whose calls go to noting stand-ins, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::input::effects::compare;
use crate::net::{CLOSE_EVENT, CLOSE_MUTEX, EVENT, MUTEX, Part, STOP, a, dispatch_net, net_entry};
use crate::system::api_stand_ins::copy_with;
use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("net::shut_down", rva(0x4184D6), thunk, &ORIGINAL)
    .private_copy(0x22)
    .relocate_calls(&[1, 0xE, 0x1B])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

/// The Rust version on the handles at `mutex` and `event`.
pub fn shut_down_with(r: &mut Regs, mutex: u32, event: u32, part: Part) -> Result<(), Refused> {
    let mut g = *r;
    part(STOP, &mut g)?;
    for (at, place) in [(mutex, CLOSE_MUTEX), (event, CLOSE_EVENT)] {
        g.eax = unsafe { &*(at as *const AtomicU32) }.swap(0, Ordering::SeqCst);
        part(place, &mut g)?;
    }
    Ok(())
}

/// The Rust version.
pub fn shut_down(r: &mut Regs, part: Part) -> Result<(), Refused> {
    shut_down_with(r, a(MUTEX), a(EVENT), part)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_net(&HOOK, r, call_original, shut_down);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, shut_down)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut mem = [0u32; 2];
    let (mutex, event) = (mem.as_mut_ptr() as u32, mem.as_mut_ptr() as u32 + 4);
    // pusha; call sub_41838C
    let entry = [0x60, 0xE8, 0xB0, 0xFE, 0xFF, 0xFF];
    let copy = match copy_with(
        0x4184D6,
        0x22,
        &entry,
        &[1, 0xE, 0x1B],
        &[STOP, CLOSE_MUTEX, CLOSE_EVENT],
        &[],
        &[(MUTEX, 4, mutex), (EVENT, 4, event)],
        2,
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    for handles in [[0u32, 0], [0x1234, 0], [0x88, 0x99]] {
        compare(
            &mut t,
            &format!("handles {handles:x?}"),
            &copy,
            Regs::SENTINEL,
            &|| unsafe { (mutex as *mut [u32; 2]).write_volatile(handles) },
            &[(mutex, 8)],
            &|r, part| shut_down_with(r, mutex, event, part),
            false,
        );
    }
    let _ = &mut mem;
    t.finish()
}
